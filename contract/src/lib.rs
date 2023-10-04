use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::Serialize;
use near_sdk::store::{LookupMap, UnorderedSet, Vector};
use near_sdk::{env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault};

use crate::StorageKey::AccrualsEntry;

const INITIAL_CLAIM_PERIOD_MS: u32 = 24 * 60 * 60;
const INITIAL_BURN_PERIOD_MS: u32 = 30 * 24 * 60 * 60;

type UnixTimestamp = u32;
type AccrualIndex = u32;
type TokensAmount = u128;
type Duration = u32; // Period in seconds

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token_account_id: AccountId,

    oracles: UnorderedSet<AccountId>,
    claim_period: Duration, // Period in milliseconds during which tokens are locked after claim
    burn_period: Duration,  // Time in milliseconds after that unclaimed tokens are burnt

    accruals: LookupMap<UnixTimestamp, (Vector<TokensAmount>, TokensAmount)>,
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
            accruals: LookupMap::new(StorageKey::Accruals),
            oracles: UnorderedSet::new(StorageKey::Oracles),

            claim_period: INITIAL_CLAIM_PERIOD_MS,
            burn_period: INITIAL_BURN_PERIOD_MS,
        }
    }

    #[private]
    pub fn add_oracle(&mut self, account_id: AccountId) {
        require!(self.oracles.insert(account_id.clone()), "Already exists");
        env::log_str(&format!("Oracle {} was added", account_id));
    }

    #[private]
    pub fn remove_oracle(&mut self, account_id: AccountId) {
        require!(self.oracles.remove(&account_id), "No such oracle");
        env::log_str(&format!("Oracle {} was removed", account_id));
    }

    pub fn get_oracles(&self) -> Vec<AccountId> {
        self.oracles.iter().cloned().collect()
    }

    pub fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>) {
        require!(
            self.oracles.contains(&env::predecessor_account_id()),
            "Unauthorized access! Only oracle can do this!"
        );

        let now_seconds: UnixTimestamp = (env::block_timestamp_ms() / 1_000) as u32;
        let mut balances: Vector<TokensAmount> = Vector::new(AccrualsEntry(now_seconds));
        let mut total_balance: TokensAmount = 0;

        for (account_id, amount) in amounts {
            let amount = amount.0;

            total_balance += amount;
            balances.push(amount);
            let index = balances.len() - 1;

            if let Some(record) = self.accounts.get_mut(&account_id) {
                record.accruals.push((now_seconds, index));
            } else {
                let mut record = AccountRecord::default();
                record.accruals.push((now_seconds, index));
                self.accounts.insert(account_id, record);
            }
        }

        self.accruals.insert(now_seconds, (balances, total_balance));
    }

    pub fn get_balance_for_account(&self, account_id: AccountId) -> U128 {
        let account_data = self
            .accounts
            .get(&account_id)
            .expect("Account data is not found");

        let result = account_data
            .accruals
            .iter()
            .map(|entry| {
                let data = self.accruals.get(&entry.0).expect("No data for date");
                data.0.get(entry.1).expect("No record for accrual")
            })
            .sum();

        U128(result)
    }

    pub fn is_claim_available(&self, account_id: AccountId) -> ClaimAvailabilityView {
        let account_data = self
            .accounts
            .get(&account_id)
            .expect("Account data is not found");

        if let Some(last_claim_at) = account_data.last_claim_at {
            let now_seconds = (env::block_timestamp_ms() / 1_000) as u32;

            if now_seconds - last_claim_at > self.claim_period {
                ClaimAvailabilityView::Available()
            } else {
                ClaimAvailabilityView::Unavailable((last_claim_at, self.claim_period))
            }
        } else {
            ClaimAvailabilityView::Available()
        }
    }
}

#[derive(Serialize)]
#[serde(
    crate = "near_sdk::serde",
    tag = "type",
    content = "data",
    rename_all = "snake_case"
)]
pub enum ClaimAvailabilityView {
    Available(),
    Unavailable((UnixTimestamp, Duration)),
}

#[derive(BorshDeserialize, BorshSerialize)]
struct AccountRecord {
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

#[cfg(test)]
mod tests {}
