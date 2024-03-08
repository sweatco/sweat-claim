use claim_model::{Duration, UnixTimestamp};
use near_sdk::env::{block_timestamp_ms, panic_str};

use crate::common::tests::data::get_test_future_success;

mod asserts;
pub(crate) mod tests;

pub(crate) fn is_promise_success(tag: &str) -> bool {
    #[cfg(test)]
    {
        get_test_future_success(tag)
    }

    #[cfg(not(test))]
    {
        near_sdk::is_promise_success()
    }
}

fn ms_timestamp_to_seconds(ms: u64) -> UnixTimestamp {
    u32::try_from(ms / 1000)
        .unwrap_or_else(|err| panic_str(&format!("Failed to get convert milliseconds to Unix timestamp: {err}")))
}

pub(crate) fn now_seconds() -> UnixTimestamp {
    ms_timestamp_to_seconds(block_timestamp_ms())
}

pub(crate) trait UnixTimestampExtension {
    fn is_within_period(&self, now: UnixTimestamp, period: Duration) -> bool;
}

impl UnixTimestampExtension for UnixTimestamp {
    fn is_within_period(&self, now: UnixTimestamp, period: Duration) -> bool {
        now - self < period
    }
}

#[test]
fn convert_milliseconds_to_unix_timestamp_successfully() {
    let millis: u64 = 1_699_038_575_819;
    let timestamp = ms_timestamp_to_seconds(millis);

    assert_eq!(1_699_038_575, timestamp);
}

#[test]
#[should_panic(expected = "Failed to get convert milliseconds to Unix timestamp")]
fn convert_milliseconds_to_unix_timestamp_with_unsuccessfully() {
    let millis: u64 = u64::MAX;
    let _timestamp = ms_timestamp_to_seconds(millis);
}
