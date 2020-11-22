use std::collections::HashMap;

use crate::types::ClientID;
use crate::transaction::Transaction;
use crate::account::Account;

/// Stores and manages accounts in the bank.
#[derive(Default)]
pub struct Bank {
    accounts: HashMap<ClientID, Account>,
}

impl Bank {
    /// Create new empty bank
    pub fn new() -> Self {
        Self { accounts: HashMap::new() }
    }

    // TODO: propagate error from apply_tx.
    /// Apply `Transaction` to the `Account` in `Bank`.
    pub fn apply_tx<T: Into<Transaction>>(&mut self, tx: T) -> Result<(), ()> {
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

    /// Consumes `Bank` returning accounts iterator.
    pub fn into_accounts_iter(self) -> impl Iterator<Item = Account> {
        self.accounts.into_iter().map(|(_, account)| account)
    }

    /// Consumes `Bank` returning accounts vector.
    pub fn into_accounts(self) -> Vec<Account> {
        self.into_accounts_iter().collect()
    }
}
