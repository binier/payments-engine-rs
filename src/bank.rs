use crate::transaction::Transaction;
use crate::account::Account;

pub trait Bank: Default {
    type AccountsIter: Iterator<Item = Account>;
    /// Apply `Transaction` to the `Account` in `Bank`.
    fn apply_tx<T: Into<Transaction>>(&mut self, tx: T) -> Result<(), ()>;
    fn into_accounts_iter(self) -> Self::AccountsIter;
}
