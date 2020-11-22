use std::convert::TryFrom;
use std::io;
use clap::{App, Arg};

mod types;
mod decimal_serde;
mod transaction;
use transaction::Transaction;

mod output_account;
use output_account::OutputAccount;

mod account;
use account::Account;

mod input_transaction;
use input_transaction::InputTransaction;

mod bank;
use bank::Bank;

/// Parses input csv and applies transactions to the new/empty `Bank`.
fn bank_from_transactions_csv(filename: &str) -> Bank {
    let mut bank = Bank::new();
    let mut rdr = csv::Reader::from_path(filename).unwrap();

    rdr.deserialize::<InputTransaction>()
        .filter_map(Result::ok)
        .map(Transaction::try_from)
        .filter_map(Result::ok)
        .for_each(|tx: Transaction| {
            // ignore result
            let _ = bank.apply_tx(tx);
        });

    bank
}

/// Extracts accounts data from the bunk and serializes to writer.
fn bank_accounts_to_csv<W: io::Write>(bank: Bank, writer: W) {
    let mut wtr = csv::Writer::from_writer(writer);

    for account in bank.into_accounts_iter() {
        let output: OutputAccount = account.into();
        wtr.serialize(output).unwrap();
    }
}

fn main() {
    let matches = App::new("simple payments engine")
        .version("0.1")
        .arg(Arg::with_name("INPUT")
             .help("input file")
             .required(true)
             .index(1))
        .get_matches();

    let bank = bank_from_transactions_csv(
        matches.value_of("INPUT").unwrap()
    );

    // No need to create BufWriter since `csv::Writer` uses it's own buffer.
    bank_accounts_to_csv(bank, io::stdout().lock());
}
