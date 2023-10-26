use integration_trait::make_integration_version;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
    AccountId, PromiseOrValue,
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
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountRecord {
    pub accruals: Vec<(UnixTimestamp, AccrualIndex)>,
    pub is_enabled: bool,
    pub last_claim_at: Option<UnixTimestamp>,
    pub is_locked: bool,
}

impl Default for AccountRecord {
    fn default() -> Self {
        Self {
            accruals: Vec::new(),
            is_enabled: true,
            last_claim_at: None,
            is_locked: false,
        }
    }
}

#[make_integration_version]
pub trait SweatClaimInterface {
    fn init(token_account_id: AccountId) -> Self;
    fn set_claim_period(&mut self, period: Duration);
    fn set_burn_period(&mut self, period: Duration);
    fn add_oracle(&mut self, account_id: AccountId);
    fn remove_oracle(&mut self, account_id: AccountId);
    fn get_oracles(&self) -> Vec<AccountId>;
    fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>);
    fn get_balance_for_account(&self, account_id: AccountId) -> U128;
    fn is_claim_available(&self, account_id: AccountId) -> ClaimAvailabilityView;
    fn claim(&mut self) -> PromiseOrValue<()>;
    fn burn(&mut self) -> PromiseOrValue<U128>;
}
