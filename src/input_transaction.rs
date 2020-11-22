use serde::Deserialize;

use crate::types::{ClientID, TransactionID, Amount};

#[derive(Deserialize)]
pub struct InputTransaction {
    #[serde(rename = "type")]
    pub tx_type: String,
    #[serde(rename = "client")]
    pub client_id: ClientID,
    #[serde(rename = "tx")]
    pub tx_id: TransactionID,
    pub amount: Option<Amount>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_empty_amount() {
        let input = "\
client,tx,type,amount
1,1,deposit,
";
        let mut rdr = csv::Reader::from_reader(input.as_bytes());
        let res: InputTransaction = rdr.deserialize().next().unwrap().unwrap();
        assert!(res.amount.is_none());
    }

    #[test]
    fn deserialize_with_amount() {
        let input = "\
client,tx,type,amount
1,1,deposit,10.543
";
        let mut rdr = csv::Reader::from_reader(input.as_bytes());
        let res: InputTransaction = rdr.deserialize().next().unwrap().unwrap();
        assert!(res.amount.is_some());
        assert_eq!(res.amount.unwrap().to_string(), "10.543");
    }
}
