use model::{AccountRecord, ClaimAvailabilityView, Duration, TokensAmount, UnixTimestamp};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    env::log_str,
    ext_contract, is_promise_success,
    json_types::U128,
    near_bindgen, require,
    serde_json::json,
    store::{LookupMap, UnorderedMap, UnorderedSet, Vector},
    AccountId, BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseOrValue,
};

use crate::StorageKey::AccrualsEntry;

mod auth;
mod burn;
mod claim;
mod clean;
mod common;
mod record;

const INITIAL_CLAIM_PERIOD_MS: u32 = 24 * 60 * 60;
const INITIAL_BURN_PERIOD_MS: u32 = 30 * 24 * 60 * 60;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token_account_id: AccountId,

    oracles: UnorderedSet<AccountId>,
    claim_period: Duration, // Period in milliseconds during which tokens are locked after claim
    burn_period: Duration,  // Time in milliseconds after that unclaimed tokens are burnt

    accruals: UnorderedMap<UnixTimestamp, (Vector<TokensAmount>, TokensAmount)>,
    accounts: LookupMap<AccountId, AccountRecord>,
}

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKey {
    Accounts,
    Accruals,
    AccrualsEntry(u32),
    Oracles,
}

#[near_bindgen]
impl Contract {
    #[init]
    #[private]
    pub fn init(token_account_id: AccountId) -> Self {
        Self {
            token_account_id,

            accounts: LookupMap::new(StorageKey::Accounts),
            accruals: UnorderedMap::new(StorageKey::Accruals),
            oracles: UnorderedSet::new(StorageKey::Oracles),

            claim_period: INITIAL_CLAIM_PERIOD_MS,
            burn_period: INITIAL_BURN_PERIOD_MS,
        }
    }

    pub fn set_claim_period(&mut self, period: Duration) {
        self.claim_period = period;
    }

    pub fn set_burn_period(&mut self, period: Duration) {
        self.burn_period = period;
    }
}
