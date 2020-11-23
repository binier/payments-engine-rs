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

        if !tx.is_ref() {
            self.transactions.insert(tx.get_tx_id(), tx);
        }
        Ok(())
    }
}

impl Into<OutputAccount> for Account {
    fn into(self) -> OutputAccount {
        OutputAccount {
            client_id: self.client_id,
            available: self.available,
            held: self.held,
            total: self.total(),
            locked: self.locked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use rust_decimal::prelude::*;
    use crate::transaction::{TransactionInfo, TransactionRef};

    fn dec(val: &str) -> Amount {
        Amount::from_str(val).unwrap()
    }

    fn zero() -> Amount {
        Amount::zero()
    }

    #[test]
    fn deposit_withdraw() {
        let mut acc = Account::new(1);

        assert!(acc.apply_tx(Transaction::Deposit(TransactionInfo {
            client_id: 1,
            tx_id: 1,
            amount: dec("1.05"),
            under_dispute: false,
        })).is_ok());

        assert_eq!(acc.available, dec("1.05"));
        assert_eq!(acc.held, zero());

        assert!(acc.apply_tx(Transaction::Withdrawal(TransactionInfo {
            client_id: 1,
            tx_id: 2,
            amount: dec("1.05"),
            under_dispute: false,
        })).is_ok());

        assert_eq!(acc.available, zero());
        assert_eq!(acc.held, zero());
    }

    #[test]
    fn withdraw_from_empty_account() {
        let mut acc = Account::new(1);

        assert!(acc.apply_tx(Transaction::Withdrawal(TransactionInfo {
            client_id: 1,
            tx_id: 2,
            amount: dec("1.05"),
            under_dispute: false,
        })).is_err());

        assert_eq!(acc.available, zero());
        assert_eq!(acc.held, zero());
    }


    #[test]
    fn withdraw_more_than_have() {
        let mut acc = Account::new(1);

        assert!(acc.apply_tx(Transaction::Deposit(TransactionInfo {
            client_id: 1,
            tx_id: 1,
            amount: dec("1.05"),
            under_dispute: false,
        })).is_ok());

        assert!(acc.apply_tx(Transaction::Withdrawal(TransactionInfo {
            client_id: 1,
            tx_id: 2,
            amount: dec("1.06"),
            under_dispute: false,
        })).is_err());

        assert_eq!(acc.available, dec("1.05"));
        assert_eq!(acc.held, zero());
    }

    #[test]
    fn withdraw_held_funds() {
        let mut acc = Account::new(1);

        assert!(acc.apply_tx(Transaction::Deposit(TransactionInfo {
            client_id: 1,
            tx_id: 1,
            amount: dec("1.05"),
            under_dispute: false,
        })).is_ok());

        assert!(acc.apply_tx(Transaction::Dispute(TransactionRef {
            client_id: 1,
            tx_id: 1,
        })).is_ok());

        assert!(acc.apply_tx(Transaction::Withdrawal(TransactionInfo {
            client_id: 1,
            tx_id: 2,
            amount: dec("1.04"),
            under_dispute: false,
        })).is_err());

        assert_eq!(acc.available, zero());
        assert_eq!(acc.held, dec("1.05"));
    }

    /// for now only deposit disputes are allowed, it should error if
    /// we try to dispute "withdrawal" transaction.
    #[test]
    fn dispute_withdrawal() {
        let mut acc = Account::new(1);

        assert!(acc.apply_tx(Transaction::Deposit(TransactionInfo {
            client_id: 1,
            tx_id: 1,
            amount: dec("1.05"),
            under_dispute: false,
        })).is_ok());

        assert!(acc.apply_tx(Transaction::Withdrawal(TransactionInfo {
            client_id: 1,
            tx_id: 2,
            amount: dec("1.05"),
            under_dispute: false,
        })).is_ok());

        assert_eq!(acc.available, zero());
        assert_eq!(acc.held, zero());

        assert!(acc.apply_tx(Transaction::Dispute(TransactionRef {
            client_id: 1,
            tx_id: 2,
        })).is_err());

        assert_eq!(acc.available, zero());
        assert_eq!(acc.held, zero());
    }

    #[test]
    fn dispute_resolve() {
        let mut acc = Account::new(1);

        assert!(acc.apply_tx(Transaction::Deposit(TransactionInfo {
            client_id: 1,
            tx_id: 1,
            amount: dec("1.05"),
            under_dispute: false,
        })).is_ok());

        assert!(acc.apply_tx(Transaction::Dispute(TransactionRef {
            client_id: 1,
            tx_id: 1,
        })).is_ok());

        assert_eq!(acc.available, zero());
        assert_eq!(acc.held, dec("1.05"));

        assert!(acc.apply_tx(Transaction::Resolve(TransactionRef {
            client_id: 1,
            tx_id: 1,
        })).is_ok());

        assert_eq!(acc.available, dec("1.05"));
        assert_eq!(acc.held, zero());
    }

    #[test]
    fn dispute_chargeback() {
        let mut acc = Account::new(1);

        assert!(acc.apply_tx(Transaction::Deposit(TransactionInfo {
            client_id: 1,
            tx_id: 1,
            amount: dec("1.05"),
            under_dispute: false,
        })).is_ok());

        assert!(acc.apply_tx(Transaction::Dispute(TransactionRef {
            client_id: 1,
            tx_id: 1,
        })).is_ok());

        assert_eq!(acc.available, zero());
        assert_eq!(acc.held, dec("1.05"));

        assert!(acc.apply_tx(Transaction::ChargeBack(TransactionRef {
            client_id: 1,
            tx_id: 1,
        })).is_ok());

        assert_eq!(acc.available, zero());
        assert_eq!(acc.held, zero());
        assert!(acc.locked);
    }

    #[test]
    fn resolve_not_existant_tx() {
        let mut acc = Account::new(1);

        assert!(acc.apply_tx(Transaction::Resolve(TransactionRef {
            client_id: 1,
            tx_id: 1,
        })).is_err());

        assert_eq!(acc.available, zero());
        assert_eq!(acc.held, zero());
    }

    #[test]
    fn resolve_not_under_dispute() {
        let mut acc = Account::new(1);

        assert!(acc.apply_tx(Transaction::Deposit(TransactionInfo {
            client_id: 1,
            tx_id: 1,
            amount: dec("1.05"),
            under_dispute: false,
        })).is_ok());

        assert!(acc.apply_tx(Transaction::Resolve(TransactionRef {
            client_id: 1,
            tx_id: 1,
        })).is_err());

        assert_eq!(acc.available, dec("1.05"));
        assert_eq!(acc.held, zero());
    }

    #[test]
    fn chargeback_not_existant_tx() {
        let mut acc = Account::new(1);

        assert!(acc.apply_tx(Transaction::ChargeBack(TransactionRef {
            client_id: 1,
            tx_id: 1,
        })).is_err());

        assert_eq!(acc.available, zero());
        assert_eq!(acc.held, zero());
        assert!(!acc.locked);
    }

    #[test]
    fn chargeback_not_under_dispute() {
        let mut acc = Account::new(1);

        assert!(acc.apply_tx(Transaction::Deposit(TransactionInfo {
            client_id: 1,
            tx_id: 1,
            amount: dec("1.05"),
            under_dispute: false,
        })).is_ok());

        assert!(acc.apply_tx(Transaction::ChargeBack(TransactionRef {
            client_id: 1,
            tx_id: 1,
        })).is_err());


        assert_eq!(acc.available, dec("1.05"));
        assert_eq!(acc.held, zero());
        assert!(!acc.locked);
    }
}
