use serde::Serializer;
use rust_decimal::{Decimal, RoundingStrategy};

/// serializes decimal with rounded down precision 4.
pub fn serialize<S>(
    num: &Decimal,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let num_str = num
        .round_dp_with_strategy(4, RoundingStrategy::RoundDown)
        .normalize()
        .to_string();
    serializer.serialize_str(&num_str)
}
