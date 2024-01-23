use model::{AssetAbbreviation, UnixTimestamp};
use near_sdk::{
    env::{block_timestamp_ms, panic_str},
    AccountId,
};

use crate::Contract;

mod asserts;
pub(crate) mod tests;

fn ms_timestamp_to_seconds(ms: u64) -> UnixTimestamp {
    u32::try_from(ms / 1000)
        .unwrap_or_else(|err| panic_str(&format!("Failed to get convert milliseconds to Unix timestamp: {err}")))
}

pub(crate) fn now_seconds() -> UnixTimestamp {
    ms_timestamp_to_seconds(block_timestamp_ms())
}

impl Contract {
    pub(crate) fn get_token_account_id(&self, asset: &AssetAbbreviation) -> AccountId {
        self.token_account_ids
            .get(asset.as_str())
            .unwrap_or_else(|| panic_str(&format!("Token {asset} is not registered")))
            .clone()
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
