pub mod account_record;
pub mod api;
pub mod event;

use near_sdk::{
    json_types::U128,
    serde::{Deserialize, Serialize},
    store::Vector,
    AccountId,
};

pub type UnixTimestamp = u32;
pub type AccrualIndex = u32;
pub type TokensAmount = u128;
pub type Duration = u32;
// Period in seconds
pub type Asset = String;

pub fn is_near(asset: &Asset) -> bool {
    asset.as_str() == "NEAR"
}

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
    pub asset: Asset,
    pub is_success: bool,
    pub total: Option<U128>,
}

impl ClaimResultView {
    pub fn new(asset: Asset, is_success: bool, total: Option<u128>) -> Self {
        Self {
            asset,
            is_success,
            total: total.map(U128),
        }
    }
}

pub type ClaimAllResultView = Vec<ClaimResultView>;

pub type BatchedAccruals = Vec<(AccountId, U128)>;

pub trait AccrualsExt {
    fn total_amount(&self) -> TokensAmount;
}

impl AccrualsExt for BatchedAccruals {
    fn total_amount(&self) -> TokensAmount {
        self.iter().map(|(_, amount)| amount.0).sum()
    }
}
