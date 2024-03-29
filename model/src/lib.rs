pub mod account_record;
pub mod api;
pub mod event;

use near_sdk::{
    json_types::U128,
    serde::{Deserialize, Serialize},
};

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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct ClaimResultView {
    pub total: U128,
}

impl ClaimResultView {
    pub fn new(total: u128) -> Self {
        Self { total: U128(total) }
    }
}
