use std::fs::File;
use std::io;
use std::convert::TryFrom;

use crate::input_transaction::InputTransaction;
use crate::transaction::Transaction;
use crate::account::Account;
use crate::output_account::OutputAccount;

pub trait Bank: Default {
    type AccountsIter: Iterator<Item = Account>;

    /// Apply `Transaction` to the `Account` in `Bank`.
    fn apply_tx<T: Into<Transaction>>(&mut self, tx: T) -> Result<(), ()>;
    fn into_accounts_iter(self) -> Self::AccountsIter;

    /// Reads and deserializes input csv from file and applies
    /// transactions to the new/empty `Bank`. Returning `Bank`.
    fn from_input_transactions_csv_file(filename: &str) -> Self {
        let file = File::open(filename).unwrap();
        Self::from_input_transactions_csv(file)
    }

    /// Reads and deserializes input csv from reader and applies
    /// transactions to the new/empty `Bank`. Returning `Bank`.
    fn from_input_transactions_csv<R: io::Read>(reader: R) -> Self {
        Self::from_input_transactions(
            csv::Reader::from_reader(reader)
                .deserialize::<InputTransaction>()
                .filter_map(Result::ok)
        )
    }

    /// Applies `InputTransaction`-s to the new/empty `Bank`. Returning `Bank`.
    fn from_input_transactions<I>(iter: I) -> Self
    where I: Iterator<Item = InputTransaction>,
    {
        let iter = iter
            .map(Transaction::try_from)
            .filter_map(Result::ok);

        Self::from_transactions(iter)
    }

    /// Applies `Transaction`-s to the new/empty `Bank`. Returning `Bank`.
    fn from_transactions<I>(it: I) -> Self
    where I: Iterator<Item = Transaction>,
    {
        let mut bank = Self::default();
        it.for_each(|tx: Transaction| {
            // ignore result
            let _ = bank.apply_tx(tx);
        });
        bank
    }


    /// Extracts accounts data from the bank and serializes
    /// [OutputAccount](crate::output_account::OutputAccount) to writer.
    /// No need to create BufWriter since `csv::Writer` uses it's own buffer.
    fn accounts_to_csv<W>(self, writer: W) -> Result<(), csv::Error>
    where W: io::Write,
    {
        let mut wtr = csv::Writer::from_writer(writer);

        for account in self.into_accounts_iter() {
            let output: OutputAccount = account.into();
            wtr.serialize(output)?;
        }
        Ok(())
    }
}
