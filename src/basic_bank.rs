use std::collections::HashMap;

use crate::types::ClientID;
use crate::transaction::Transaction;
use crate::account::Account;
use crate::bank::Bank;

/// Stores and manages accounts in the bank.
#[derive(Default)]
pub struct BasicBank {
    accounts: HashMap<ClientID, Account>,
}

impl Bank for BasicBank {
    type AccountsIter = Box<dyn Iterator<Item = Account>>;

    // TODO: propagate error from apply_tx.
    /// Apply `Transaction` to the `Account` in `BasicBank`.
    fn apply_tx<T: Into<Transaction>>(&mut self, tx: T) -> Result<(), ()> {
        let tx: Transaction = tx.into();
        let client_id = tx.get_client_id();

        self.accounts
            .entry(client_id)
            .or_insert(Account::new(client_id))
            .apply_tx(tx)
            .or(Err(())) // ignore error
    }

    /// Consumes `BasicBank` returning accounts iterator.
    fn into_accounts_iter(self) -> Self::AccountsIter {
        Box::new(
            self.accounts.into_iter().map(|(_, account)| account)
        )
    }
}

impl BasicBank {
    /// Create new empty `BasicBank`
    pub fn new() -> Self {
        Self { accounts: HashMap::new() }
    }
}
