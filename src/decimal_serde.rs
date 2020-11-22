use serde::Serializer;
use rust_decimal::{Decimal, RoundingStrategy};

/// Serializes decimal with rounded down precision 4,
/// as requested from spec document.
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
