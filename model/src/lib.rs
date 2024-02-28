pub mod api;
pub mod event;

use near_sdk::{
    json_types::U128,
    serde::{Deserialize, Serialize},
    store::{UnorderedMap, Vector},
};

pub type UnixTimestamp = u32;
pub type AccrualIndex = u32;
pub type TokensAmount = u128;
// Period in seconds
pub type Duration = u32;
pub type Asset = String;
pub type AccrualsMap = UnorderedMap<UnixTimestamp, (Vector<TokensAmount>, TokensAmount)>;

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
