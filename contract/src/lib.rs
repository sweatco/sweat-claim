use model::{account_record::AccountRecord, api::InitApi, Duration, TokensAmount, UnixTimestamp};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    near_bindgen,
    store::{LookupMap, UnorderedMap, UnorderedSet, Vector},
    AccountId, BorshStorageKey, PanicOnDefault,
};

mod auth;
mod burn;
mod claim;
mod clean;
mod common;
mod config;
mod record;

const INITIAL_CLAIM_PERIOD_MS: u32 = 24 * 60 * 60;
const INITIAL_BURN_PERIOD_MS: u32 = 30 * 24 * 60 * 60;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token_account_id: AccountId,

    oracles: UnorderedSet<AccountId>,
    claim_period: Duration, // Period in seconds during which tokens are locked after claim
    burn_period: Duration,  // Time in seconds after that unclaimed tokens are burnt

    accruals: UnorderedMap<UnixTimestamp, (Vector<TokensAmount>, TokensAmount)>,
    accounts: LookupMap<AccountId, AccountRecord>,

    is_service_call_running: bool,
}

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKey {
    Accounts,
    Accruals,
    AccrualsEntry(u32),
    Oracles,
}

#[near_bindgen]
impl InitApi for Contract {
    #[init]
    fn init(token_account_id: AccountId) -> Self {
        Self::assert_private();

        Self {
            token_account_id,

            accounts: LookupMap::new(StorageKey::Accounts),
            accruals: UnorderedMap::new(StorageKey::Accruals),
            oracles: UnorderedSet::new(StorageKey::Oracles),

            claim_period: INITIAL_CLAIM_PERIOD_MS,
            burn_period: INITIAL_BURN_PERIOD_MS,

            is_service_call_running: false,
        }
    }
}
