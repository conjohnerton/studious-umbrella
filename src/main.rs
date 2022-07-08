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
type TransactionInfo = (Amount, bool);
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
    let mut transaction_info: HashMap<TransactionId, TransactionInfo> = HashMap::new();

    for group in csv_reader.deserialize() {
        let transaction: Transaction = group.unwrap();

        match transaction.tx_type() {
            TransactionType::Deposit => {
                let amount = transaction.amount().unwrap();
                transaction_info.insert(transaction.tx(), (amount, false));

                match client_accounts.get(&transaction.client()) {
                    Some(account) => {
                        if account.locked {
                            continue
                        }

                        client_accounts.insert(
                            transaction.client(),
                            Account {
                                available: account.available + amount,
                                held: account.held,
                                total: account.total + amount,
                                locked: account.locked,
                            },
                        );
                        ()
                    }
                    None => {
                        client_accounts.insert(
                            transaction.client(),
                            Account {
                                available: amount,
                                held: Amount::default(),
                                total: amount,
                                locked: false,
                            },
                        );
                        ()
                    }
                };
            }
            TransactionType::Withdrawal => {
                let amount = transaction.amount().unwrap();
                transaction_info.insert(transaction.tx(), (-amount, false));

                match client_accounts.get(&transaction.client()) {
                    Some(account) => {
                        if account.locked {
                            continue
                        }

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
                match transaction_info.get(&transaction.tx()) {
                    Some(&(dispute_amount, false)) => {
                        match client_accounts.get(&transaction.client()) {
                            Some(account) => {
                                transaction_info.insert(transaction.tx(), (dispute_amount, true));
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
                        }
                    }
                    _ => (),
                };
            }
            TransactionType::Resolve => {
                match transaction_info.get(&transaction.tx()) {
                    Some(&(dispute_amount, true)) => {
                        match client_accounts.get(&transaction.client()) {
                            Some(account) => {
                                transaction_info.insert(transaction.tx(), (dispute_amount, false));
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
                        }
                    }
                    _ => (),
                };
            }
            TransactionType::Chargeback => {
                match transaction_info.get(&transaction.tx()) {
                    Some((dispute_amount, true)) => {
                        match client_accounts.get(&transaction.client()) {
                            Some(account) => {
                                client_accounts.insert(
                                    transaction.client(),
                                    Account {
                                        available: account.available,
                                        held: account.held - dispute_amount,
                                        total: account.total - dispute_amount,
                                        locked: true,
                                    },
                                );
                                ()
                            }
                            None => (),
                        }
                    }
                    _ => (),
                };
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
