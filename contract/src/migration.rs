use claim_model::{AccrualsMap, Duration};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, near_bindgen,
    store::{LookupMap, UnorderedMap, UnorderedSet},
    AccountId, PanicOnDefault,
};

use crate::{record::model::legacy::AccountRecordLegacy, Contract, ContractExt, StorageKey};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ContractLegacy {
    token_account_id: AccountId,
    oracles: UnorderedSet<AccountId>,
    claim_period: Duration,
    burn_period: Duration,
    accruals: AccrualsMap,
    accounts: LookupMap<AccountId, AccountRecordLegacy>,
    is_service_call_running: bool,
}

#[near_bindgen]
impl Contract {
    #[allow(deprecated)]
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let old_state: ContractLegacy = env::state_read().expect("Failed to read state");

        Self {
            token_account_id: old_state.token_account_id,
            assets: UnorderedMap::new(StorageKey::Assets),
            oracles: old_state.oracles,
            claim_period: old_state.claim_period,
            burn_period: old_state.burn_period,
            accruals: old_state.accruals,
            extra_accruals: LookupMap::new(StorageKey::ExtraAccruals),
            accounts_legacy: old_state.accounts,
            accounts: LookupMap::new(StorageKey::Accounts),
            is_service_call_running: old_state.is_service_call_running,
        }
    }
}
