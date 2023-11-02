pub mod account_record;
pub mod api;

use near_sdk::serde::{Deserialize, Serialize};

pub type UnixTimestamp = u32;
pub type AccrualIndex = u32;
pub type TokensAmount = u128;
pub type Duration = u32; // Period in seconds

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde", tag = "type", content = "data", rename_all = "snake_case")]
pub enum ClaimAvailabilityView {
    Available,
    Unavailable((UnixTimestamp, Duration)),
    Unregistered,
}
