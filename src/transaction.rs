use std::convert::TryFrom;

use crate::types::{ClientID, TransactionID, Amount};
use crate::input_transaction::InputTransaction;

/// Ref to the existing transaction.
#[derive(Debug)]
pub struct TransactionRef {
    pub client_id: ClientID,
    pub tx_id: TransactionID,
}

/// Transaction info.
#[derive(Debug)]
pub struct TransactionInfo {
    pub client_id: ClientID,
    pub tx_id: TransactionID,
    pub amount: Amount,
    pub under_dispute: bool,
}

/// Different types of transactions that are supported.
#[derive(Debug)]
pub enum Transaction {
    /// Money deposited/added to the account.
    Deposit(TransactionInfo),
    /// Money withdrawn/removed from the account.
    Withdrawal(TransactionInfo),
    /// Start dispute for the existing `Transaction`.
    Dispute(TransactionRef),
    /// Resolve `Dispute` by basically undoing it.
    Resolve(TransactionRef),
    /// Chargeback `Dispute`. It freezes/locks the account.
    ChargeBack(TransactionRef),
}

impl Transaction {
    pub fn is_ref(&self) -> bool {
        match self {
            Self::Deposit(_) | Self::Withdrawal(_) => false,
            _ => true,
        }
    }

    pub fn get_client_id(&self) -> ClientID {
        match self {
            Transaction::Deposit(tx) => tx.client_id,
            Transaction::Withdrawal(tx) => tx.client_id,
            Transaction::Dispute(tx) => tx.client_id,
            Transaction::Resolve(tx) => tx.client_id,
            Transaction::ChargeBack(tx) => tx.client_id,
        }
    }

    /// Get `Transaction` ID.
    pub fn get_tx_id(&self) -> TransactionID {
        match self {
            Transaction::Deposit(tx) => tx.tx_id,
            Transaction::Withdrawal(tx) => tx.tx_id,
            Transaction::Dispute(tx) => tx.tx_id,
            Transaction::Resolve(tx) => tx.tx_id,
            Transaction::ChargeBack(tx) => tx.tx_id,
        }
    }

    /// Get string representation of the `Transaction` type.
    pub fn get_type(&self) -> &'static str {
        match self {
            Transaction::Deposit(_) => "deposit",
            Transaction::Withdrawal(_) => "withdrawal",
            Transaction::Dispute(_) => "dispute",
            Transaction::Resolve(_) => "resolve",
            Transaction::ChargeBack(_) => "chargeback"
        }
    }
}

impl TryFrom<InputTransaction> for Transaction {
    // TODO: implement errors enum
    type Error = &'static str;

    fn try_from(input: InputTransaction) -> Result<Self, Self::Error> {
        let InputTransaction { client_id, tx_id, tx_type, amount } = input;

        if let "deposit" | "withdrawal" = tx_type.as_str() {
            if amount.is_none() {
                return Err("for deposit and withdrawal, amount can't be none");
            }
            let tx_info = TransactionInfo {
                client_id,
                tx_id,
                amount: amount.unwrap(),
                under_dispute: false,
            };
            return Ok(match tx_type.as_str() {
                "deposit" => Transaction::Deposit(tx_info),
                "withdrawal" => Transaction::Withdrawal(tx_info),
                _ => unreachable!(),
            });
        }

        let tx_ref = TransactionRef { client_id, tx_id };

        match tx_type.as_str() {
            "dispute" => Ok(Transaction::Dispute(tx_ref)),
            "resolve" => Ok(Transaction::Resolve(tx_ref)),
            "chargeback" => Ok(Transaction::ChargeBack(tx_ref)),
            _ => Err("unknown transaction type"),
        }
    }
}
