use claim_model::{Asset, TokensAmount, UnixTimestamp};
use near_sdk::serde::{self, Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub(crate) struct ClaimDetails {
    pub accruals: Vec<(UnixTimestamp, TokensAmount)>,
    pub total: TokensAmount,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub(crate) struct Claim {
    pub asset: Asset,
    pub details: ClaimDetails,
}

impl Claim {
    pub(crate) fn new(asset: Asset, details: ClaimDetails) -> Self {
        Self { asset, details }
    }
}
