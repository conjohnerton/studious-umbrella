use std::{collections::HashMap, env, fs::File};

use anyhow::{bail, Context, Result};
use csv::{self, Trim};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
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
type Transaction = (TransactionType, Client, TransactionId, Amount);
struct Account {
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}

fn try_main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let file_name: &str = match &args[..] {
        [_, file] => file,
        _ => "To use this tool, pass in a single filename.",
    };

    let file = File::open(file_name)?;
    let mut csv_reader = csv::ReaderBuilder::new().trim(Trim::All).from_reader(file);

    let mut client_accounts: HashMap<Client, Account> = HashMap::new();
    let mut transaction_amounts: HashMap<TransactionId, Amount> = HashMap::new();

    for group in csv_reader.deserialize() {
        let (tx_type, client, tx, amount): Transaction =
            group.context("Could not get record from result group")?;
        // println!("{:?}", tx_type);
        // println!("{:?}", client);
        // println!("{:?}", tx);
        // println!("{:?}", amount);

        match tx_type {
            TransactionType::Deposit => {
                transaction_amounts.insert(tx, amount);

                match client_accounts.get(&client) {
                    None => client_accounts.insert(
                        client,
                        Account {
                            available: amount,
                            held: Amount::default(),
                            total: amount,
                            locked: false,
                        },
                    ),
                    Some(account) => client_accounts.insert(
                        client,
                        Account {
                            available: account.available + amount,
                            held: account.held,
                            total: account.total+ amount,
                            locked: account.locked,
                        },
                    ),
                };
            }
            TransactionType::Withdrawal => {
                transaction_amounts.insert(tx, amount);

                match client_accounts.get(&client) {
                    None => (),
                    Some(account) => {
                        let available_after_withdrawal = (account.available - amount).round_dp(4);
                        let total_after_withdrawal = (account.total - amount).round_dp(4);

                        if available_after_withdrawal < Amount::ZERO {
                            // bail!("Rejected")
                            ()
                        } else {
                            client_accounts.insert(
                                client,
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
                };
            }
            TransactionType::Dispute => {
                // transaction_amounts.get(tx)
            }
            TransactionType::Resolve => {}
            TransactionType::Chargeback => {}
        };
    }

    print_output(client_accounts);

    Ok(())
}

fn print_output(client_account: HashMap<Client, Account>) {
    println!("client, available, held, total, locked");

    for (client, account) in client_account {
        println!("{}, {}, {}, {}, {}", client, account.available, account.held, account.total, account.locked);
    }
}

fn main() {
    if let Err(err) = try_main() {
        println!("ERROR: {}", err);
        // err.chain().skip(1).for_each(|cause| eprintln!("because: {}", cause));
        std::process::exit(1);
    }
}
