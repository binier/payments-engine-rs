use std::collections::HashMap;

use crate::types::{ClientID, TransactionID, Amount};
use crate::transaction::Transaction;
use crate::output_account::OutputAccount;

#[derive(Debug)]
pub struct Account {
    client_id: ClientID,
    /// Amount on the balance that.
    available: Amount,
    /// Amount that is `held` because of the ongoing dispute.
    held: Amount,
    /// Whether account is locked/frozen.
    /// Happens if we encounter `Transaction::Chargeback`
    locked: bool,
    transactions: HashMap<TransactionID, Transaction>,
}

impl Account {
    /// Creates an **unlocked** account with **zero** balance
    /// and with no transactions.
    pub fn new(client_id: ClientID) -> Self {
        Self {
            client_id,
            available: Default::default(),
            held: Default::default(),
            locked: false,
            transactions: HashMap::new(),
        }
    }

    /// Total amount that user has: **available + held**
    pub fn total(&self) -> Amount {
        self.available + self.held
    }

    /// Should be called cautiously outside `apply_tx`, since `apply_tx`
    /// does bunch of checks before calling this method, which we don't
    /// do here. Also transation won't be added to `Self::transactions`.
    fn dispute_tx_with_id(&mut self, tx_id: TransactionID) -> Result<(), &'static str> {
        let tx = match self.transactions.get_mut(&tx_id) {
            Some(tx) => tx,
            None => return Err("transaction for dispute not found"),
        };

        match tx {
            Transaction::Deposit(tx_info) => {
                if tx_info.under_dispute {
                    return Err("can't dispute transaction that's already under dispute");
                }

                if self.available < tx_info.amount {
                    return Err("insufficient funds for dispute");
                }

                tx_info.under_dispute = true;
                self.available -= tx_info.amount;
                self.held += tx_info.amount;
            },
            _ => return Err("only deposit transaction can be disputed"),
        };

        Ok(())
    }

    /// Should be called cautiously outside `apply_tx`, since `apply_tx`
    /// does bunch of checks before calling this method, which we don't
    /// do here. Also transation won't be added to `Self::transactions`.
    fn resolve_tx_with_id(&mut self, tx_id: TransactionID) -> Result<(), &'static str> {
        let tx = match self.transactions.get_mut(&tx_id) {
            Some(tx) => tx,
            None => return Err("transaction for dispute not found"),
        };

        match tx {
            Transaction::Deposit(tx_info) => {
                if !tx_info.under_dispute {
                    return Err("transaction is not under dispute");
                }

                if self.held < tx_info.amount {
                    unreachable!("held amount is less then disputed amount");
                }

                tx_info.under_dispute = false;
                self.available += tx_info.amount;
                self.held -= tx_info.amount;
            },
            _ => return Err("only deposit transaction can be disputed"),
        };

        Ok(())
    }

    /// Should be called cautiously outside `apply_tx`, since `apply_tx`
    /// does bunch of checks before calling this method, which we don't
    /// do here. Also transation won't be added to `Self::transactions`.
    fn chargeback_tx_with_id(&mut self, tx_id: TransactionID) -> Result<(), &'static str> {
        let tx = match self.transactions.get_mut(&tx_id) {
            Some(tx) => tx,
            None => return Err("transaction for dispute not found"),
        };

        match tx {
            Transaction::Deposit(tx_info) => {
                if !tx_info.under_dispute {
                    return Err("transaction is not under dispute");
                }

                if self.held < tx_info.amount {
                    unreachable!("held amount is less then disputed amount");
                }

                tx_info.under_dispute = false;
                self.held -= tx_info.amount;
            },
            _ => return Err("only deposit transaction can be disputed"),
        };

        // should lock account if chargeback occured.
        self.locked = true;
        Ok(())
    }

    // TODO: create errors enum.
    /// Apply transaction to the account.
    pub fn apply_tx(&mut self, tx: Transaction) -> Result<(), &'static str> {
        if self.locked {
            return Err("can't apply transaction to a locked account");
        }

        if !tx.is_ref() && self.transactions.contains_key(&tx.get_tx_id()) {
            return Err("transaction with same id already applied");
        }

        match &tx {
            Transaction::Deposit(tx_info) => self.available += tx_info.amount,
            Transaction::Withdrawal(tx_info) => {
                if tx_info.amount > self.available {
                    return Err("insufficient funds");
                }
                self.available -= tx_info.amount;
            },
            Transaction::Dispute(tx_ref) => {
                self.dispute_tx_with_id(tx_ref.tx_id)?;
            },
            Transaction::Resolve(tx_ref) => {
                self.resolve_tx_with_id(tx_ref.tx_id)?;
            },
            Transaction::ChargeBack(tx_ref) => {
                self.chargeback_tx_with_id(tx_ref.tx_id)?;
            }
        };

        self.transactions.insert(tx.get_tx_id(), tx);
        Ok(())
    }
}

impl Into<OutputAccount> for Account {
    fn into(self) -> OutputAccount {
        OutputAccount {
            client_id: self.client_id,
            available: self.available,
            held: self.held,
            total: self.available + self.held,
            locked: self.locked,
        }
    }
}
