use serde::Serialize;

use crate::types::{ClientID, Amount};
use crate::decimal_serde::serialize as serialize_decimal;

#[derive(Serialize)]
pub struct OutputAccount {
    #[serde(rename = "client")]
    pub client_id: ClientID,
    #[serde(serialize_with = "serialize_decimal")]
    pub available: Amount,
    #[serde(serialize_with = "serialize_decimal")]
    pub held: Amount,
    #[serde(serialize_with = "serialize_decimal")]
    pub total: Amount,
    pub locked: bool,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    fn serialize_to_string<I: IntoIterator<Item = T>, T: Serialize>(it: I) -> String {
        let mut wtr = csv::Writer::from_writer(vec![]);

        it.into_iter().for_each(|v| wtr.serialize(v).unwrap());

        wtr.flush().unwrap();
        String::from_utf8(wtr.into_inner().unwrap()).unwrap()
    }

    #[test]
    fn serialize_decimal_precision_4() {
        let output = {
            let available = Amount::from_str("10.5234").unwrap();
            let held = Amount::from_str("30.293853").unwrap();
            let total = available + held;
            serialize_to_string(vec![OutputAccount {
                client_id: 1,
                available,
                held,
                total,
                locked: false
            }])

        };
        let output = output.split('\n').collect::<Vec<_>>()[1];
        let vals: Vec<_> = output.split(',').collect();

        assert_eq!(vals[1], "10.5234");
        assert_eq!(vals[2], "30.2938");
    }
}
