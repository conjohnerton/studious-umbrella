use std::{collections::HashMap, env, fs::File};

use anyhow::Result;
use csv::{self, Trim};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,

    #[serde(rename = "withdrawal")]
    Withdrawal,

    #[serde(rename = "dispute")]
    Dispute,

    #[serde(rename = "resolve")]
    Resolve,

    #[serde(rename = "chargeback")]
    Chargeback,
}
type Client = u16;
type TransactionId = u32;
type Amount = Decimal;
struct Account {
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}

#[derive(Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    tx_type: TransactionType,
    client: Client,
    tx: TransactionId,
    amount: Option<Amount>,
}
impl Transaction {
    pub fn tx_type(&self) -> TransactionType {
        self.tx_type.clone()
    }

    pub fn client(&self) -> Client {
        self.client
    }

    pub fn tx(&self) -> TransactionId {
        self.tx
    }

    pub fn amount(&self) -> Option<Decimal> {
        self.amount
    }
}

fn try_main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let file_name: &str = match &args[..] {
        [_, file] => file,
        _ => "To use this tool, pass in a single filename.",
    };

    let file = File::open(file_name)?;
    let mut csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .from_reader(file);

    let mut client_accounts: HashMap<Client, Account> = HashMap::new();
    let mut transaction_amounts: HashMap<TransactionId, Amount> = HashMap::new();

    for group in csv_reader.deserialize() {
        let transaction: Transaction = group.unwrap();
        // println!("{:?}", transaction.tx_type());
        // println!("{:?}", transaction.client());
        // println!("{:?}", transaction.tx());
        // println!("{:?}", transaction.amount());

        match transaction.tx_type() {
            TransactionType::Deposit => {
                let amount = transaction.amount().unwrap();
                transaction_amounts.insert(transaction.tx(), amount);

                match client_accounts.get(&transaction.client()) {
                    Some(account) => client_accounts.insert(
                        transaction.client(),
                        Account {
                            available: account.available + amount,
                            held: account.held,
                            total: account.total + amount,
                            locked: account.locked,
                        },
                    ),
                    None => client_accounts.insert(
                        transaction.client(),
                        Account {
                            available: amount,
                            held: Amount::default(),
                            total: amount,
                            locked: false,
                        },
                    ),
                };
            }
            TransactionType::Withdrawal => {
                let amount = transaction.amount().unwrap();
                transaction_amounts.insert(transaction.tx(), amount);

                match client_accounts.get(&transaction.client()) {
                    Some(account) => {
                        let available_after_withdrawal = (account.available - amount).round_dp(4);
                        let total_after_withdrawal = (account.total - amount).round_dp(4);

                        if available_after_withdrawal < Amount::ZERO {
                            ()
                        } else {
                            client_accounts.insert(
                                transaction.client(),
                                Account {
                                    available: available_after_withdrawal,
                                    held: account.held,
                                    total: total_after_withdrawal,
                                    locked: account.locked,
                                },
                            );
                            ()
                        }
                    }
                    None => (),
                };
            }
            TransactionType::Dispute => {
                match transaction_amounts.get(&transaction.tx()) {
                    Some(dispute_amount) => match client_accounts.get(&transaction.client()) {
                        Some(account) => {
                            client_accounts.insert(
                                transaction.client(),
                                Account {
                                    available: account.available - dispute_amount,
                                    held: account.held + dispute_amount,
                                    total: account.total,
                                    locked: account.locked,
                                },
                            );
                            ()
                        }
                        None => (),
                    },
                    None => (),
                };
            }
            TransactionType::Resolve => {
                match transaction_amounts.get(&transaction.tx()) {
                    Some(dispute_amount) => match client_accounts.get(&transaction.client()) {
                        Some(account) => {
                            client_accounts.insert(
                                transaction.client(),
                                Account {
                                    available: account.available + dispute_amount,
                                    held: account.held - dispute_amount,
                                    total: account.total,
                                    locked: account.locked,
                                },
                            );
                            ()
                        }
                        None => (),
                    },
                    None => (),
                };
            }
            TransactionType::Chargeback => {
                
            }
        };
    }

    print_output(client_accounts);

    Ok(())
}

fn print_output(client_account: HashMap<Client, Account>) {
    println!("client, available, held, total, locked");

    for (client, account) in client_account {
        println!(
            "{}, {}, {}, {}, {}",
            client, account.available, account.held, account.total, account.locked
        );
    }
}

fn main() {
    if let Err(err) = try_main() {
        println!("ERROR: {}", err);
        // err.chain().skip(1).for_each(|cause| eprintln!("because: {}", cause));
        std::process::exit(1);
    }
}
