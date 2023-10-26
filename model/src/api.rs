use integration_trait::make_integration_version;
use near_sdk::{json_types::U128, AccountId, PromiseOrValue};

use crate::{ClaimAvailabilityView, Duration};

#[make_integration_version]
pub trait InitApi {
    fn init(token_account_id: AccountId) -> Self;
    fn set_claim_period(&mut self, period: Duration);
    fn set_burn_period(&mut self, period: Duration);
}

#[make_integration_version]
pub trait AuthApi {
    fn add_oracle(&mut self, account_id: AccountId);
    fn remove_oracle(&mut self, account_id: AccountId);
    fn get_oracles(&self) -> Vec<AccountId>;
}

#[make_integration_version]
pub trait BurnApi {
    fn burn(&mut self) -> PromiseOrValue<U128>;
}

#[make_integration_version]
pub trait RecordApi {
    fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>);
}

#[make_integration_version]
pub trait ClaimApi {
    fn get_claimable_balance_for_account(&self, account_id: AccountId) -> U128;
    fn is_claim_available(&self, account_id: AccountId) -> ClaimAvailabilityView;
    fn claim(&mut self) -> PromiseOrValue<()>;
}
