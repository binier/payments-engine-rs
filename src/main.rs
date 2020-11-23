use std::io;
use clap::{App, Arg};

mod types;
mod decimal_serde;
mod input_transaction;
mod transaction;
mod account;
mod output_account;

mod bank;
use bank::Bank;

mod basic_bank;
use basic_bank::BasicBank;

mod concurrent_bank;
use concurrent_bank::ConcurrentBank;

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
        let bank = BasicBank::from_input_transactions_csv_file(filename);
        bank.accounts_to_csv(io::stdout().lock()).unwrap();
    } else {
        let bank = ConcurrentBank::from_input_transactions_csv_file(filename);
        bank.accounts_to_csv(io::stdout().lock()).unwrap();
    }
}
