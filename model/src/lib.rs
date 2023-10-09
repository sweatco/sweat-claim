use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
};

pub type UnixTimestamp = u32;
pub type AccrualIndex = u32;
pub type TokensAmount = u128;
pub type Duration = u32; // Period in seconds

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde", tag = "type", content = "data", rename_all = "snake_case")]
pub enum ClaimAvailabilityView {
    Available(),
    Unavailable((UnixTimestamp, Duration)),
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountRecord {
    pub accruals: Vec<(UnixTimestamp, AccrualIndex)>,
    pub is_enabled: bool,
    pub last_claim_at: Option<UnixTimestamp>,
}

impl Default for AccountRecord {
    fn default() -> Self {
        Self {
            accruals: Vec::new(),
            is_enabled: true,
            last_claim_at: None,
        }
    }
}
