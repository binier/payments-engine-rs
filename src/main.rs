use std::convert::TryFrom;
use std::io;
use clap::{App, Arg};

mod types;
mod decimal_serde;
mod account;

mod transaction;
use transaction::Transaction;

mod output_account;
use output_account::OutputAccount;

mod input_transaction;
use input_transaction::InputTransaction;

mod bank;
use bank::Bank;

mod basic_bank;
use basic_bank::BasicBank;

mod concurrent_bank;
use concurrent_bank::ConcurrentBank;

/// Parses input csv and applies transactions to the new/empty `Bank`.
fn bank_from_transactions_csv<B: Bank>(filename: &str) -> B {
    let mut bank = B::default();
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
/// No need to create BufWriter since `csv::Writer` uses it's own buffer.
fn bank_accounts_to_csv<B, W>(bank: B, writer: W)
where B: Bank,
      W: io::Write,
{
    let mut wtr = csv::Writer::from_writer(writer);

    for account in bank.into_accounts_iter() {
        let output: OutputAccount = account.into();
        wtr.serialize(output).unwrap();
    }
}

fn main() {
    // parse cli args
    let matches = App::new("simple payments engine")
        .version("0.1")
        .arg(Arg::with_name("INPUT")
             .help("input file")
             .required(true)
             .index(1))
        .arg(Arg::with_name("concurrent")
             .help("concurrent mode")
             .short("c")
             .long("concurrent")
             .takes_value(false))
        .get_matches();

    let filename = matches.value_of("INPUT").unwrap();
    let is_concurrent = matches.is_present("concurrent");

    if !is_concurrent {
        let bank = bank_from_transactions_csv::<BasicBank>(filename);
        bank_accounts_to_csv(bank, io::stdout().lock());
    } else {
        let bank = bank_from_transactions_csv::<ConcurrentBank>(filename);
        bank_accounts_to_csv(bank, io::stdout().lock());
    }
}
