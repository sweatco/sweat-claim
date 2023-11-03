use model::UnixTimestamp;
use near_sdk::env::{block_timestamp_ms, panic_str};

mod asserts;
pub(crate) mod tests;

fn ms_timestamp_to_seconds(ms: u64) -> UnixTimestamp {
    u32::try_from(ms / 1000)
        .unwrap_or_else(|err| panic_str(&format!("Failed to get convert milliseconds to Unix timestamp: {err}")))
}

pub(crate) fn now_seconds() -> UnixTimestamp {
    ms_timestamp_to_seconds(block_timestamp_ms())
}
