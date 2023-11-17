use model::UnixTimestamp;
use near_sdk::env::panic_str;

mod asserts;
pub(crate) mod tests;

pub(crate) fn unix_timestamp(ms: u64) -> UnixTimestamp {
    u32::try_from(ms / 1000).unwrap_or_else(|_| panic_str("Failed to get convert milliseconds to Unix timestamp"))
}
