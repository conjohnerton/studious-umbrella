mod types;

use std::{collections::HashMap, env, fs::File};

use anyhow::Result;
use csv::{self, Trim};

use types::*;

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
                deposit(transaction, &mut client_accounts, &mut transaction_info)
            }
            TransactionType::Withdrawal => {
                withdrawal(transaction, &mut client_accounts, &mut transaction_info)
            }
            TransactionType::Dispute => {
                dispute(transaction, &mut client_accounts, &mut transaction_info)
            }
            TransactionType::Resolve => {
                resolve(transaction, &mut client_accounts, &mut transaction_info)
            }
            TransactionType::Chargeback => {
                chargeback(transaction, &mut client_accounts, &mut transaction_info)
            }
        };
    }

    print_output(client_accounts);

    Ok(())
}

fn chargeback(
    transaction: Transaction,
    client_accounts: &mut HashMap<Client, Account>,
    transaction_info: &mut HashMap<TransactionId, TransactionInfo>,
) {
    if let Some(&(dispute_amount, true)) = transaction_info.get(&transaction.tx()) {
        if let Some(account) = client_accounts.get(&transaction.client()) {
            if account.locked {
                return;
            }

            transaction_info.insert(transaction.tx(), (dispute_amount, false));
            client_accounts.insert(
                transaction.client(),
                Account {
                    available: account.available,
                    held: account.held - dispute_amount,
                    total: account.total - dispute_amount,
                    locked: true,
                },
            );
        }
    }
}

fn resolve(
    transaction: Transaction,
    client_accounts: &mut HashMap<Client, Account>,
    transaction_info: &mut HashMap<TransactionId, TransactionInfo>,
) {
    if let Some(&(dispute_amount, true)) = transaction_info.get(&transaction.tx()) {
        if let Some(account) = client_accounts.get(&transaction.client()) {
            if account.locked {
                return;
            }

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
        }
    }
}

fn dispute(
    transaction: Transaction,
    client_accounts: &mut HashMap<Client, Account>,
    transaction_info: &mut HashMap<TransactionId, TransactionInfo>,
) {
    if let Some(&(dispute_amount, false)) = transaction_info.get(&transaction.tx()) {
        // Reject disputes of withdrawal, since that's not something we can handle
        if dispute_amount < Amount::ZERO {
            return;
        }

        if let Some(account) = client_accounts.get(&transaction.client()) {
            if account.locked {
                return;
            }

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
        }
    }
}

fn withdrawal(
    transaction: Transaction,
    client_accounts: &mut HashMap<Client, Account>,
    transaction_info: &mut HashMap<TransactionId, TransactionInfo>,
) {
    let amount = transaction.amount().unwrap();
    transaction_info.insert(transaction.tx(), (-amount, false));

    if let Some(account) = client_accounts.get(&transaction.client()) {
        if account.locked {
            return;
        }

        let available_after_withdrawal = (account.available - amount).round_dp(4);
        let total_after_withdrawal = (account.total - amount).round_dp(4);

        if available_after_withdrawal >= Amount::ZERO {
            client_accounts.insert(
                transaction.client(),
                Account {
                    available: available_after_withdrawal,
                    held: account.held,
                    total: total_after_withdrawal,
                    locked: account.locked,
                },
            );
        };
    }
}

fn deposit(
    transaction: Transaction,
    client_accounts: &mut HashMap<Client, Account>,
    transaction_info: &mut HashMap<TransactionId, TransactionInfo>,
) {
    let amount = transaction.amount().unwrap();
    transaction_info.insert(transaction.tx(), (amount, false));

    match client_accounts.get(&transaction.client()) {
        Some(account) => {
            if account.locked {
                return;
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
        }
    };
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
