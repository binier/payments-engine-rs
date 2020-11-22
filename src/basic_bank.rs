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
        let account: &mut Account = loop {
            let client_id = tx.get_client_id();

            if let Some(account) = self.accounts.get_mut(&client_id) {
                break account;
            }
            self.accounts.insert(client_id, Account::new(client_id));
        };

        match account.apply_tx(tx) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
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
