use model::{AccountRecord, ClaimAvailabilityView, Duration, TokensAmount, UnixTimestamp};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    env::log_str,
    ext_contract, is_promise_success,
    json_types::U128,
    near_bindgen, require,
    serde::Serialize,
    serde_json::json,
    store::{LookupMap, UnorderedMap, UnorderedSet, Vector},
    AccountId, BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseOrValue,
};

use crate::StorageKey::AccrualsEntry;

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
        log_str(format!("Record batch: {amounts:?}").as_str());

        require!(
            self.oracles.contains(&env::predecessor_account_id()),
            "Unauthorized access! Only oracle can do this!"
        );

        let now_seconds: UnixTimestamp = (env::block_timestamp_ms() / 1_000) as u32;
        let mut balances: Vector<TokensAmount> = Vector::new(AccrualsEntry(now_seconds));
        let mut total_balance: TokensAmount = 0;

        for (account_id, amount) in amounts {
            log_str(format!("Record {amount:?} for {account_id}").as_str());

            let amount = amount.0;

            total_balance += amount;
            balances.push(amount);
            let index = balances.len() - 1;

            if let Some(record) = self.accounts.get_mut(&account_id) {
                record.accruals.push((now_seconds, index));
            } else {
                let mut record = AccountRecord::default();
                record.last_claim_at = Some(now_seconds);
                record.accruals.push((now_seconds, index));
                self.accounts.insert(account_id, record);
            }
        }

        self.accruals.insert(now_seconds, (balances, total_balance));
    }

    pub fn get_balance_for_account(&self, account_id: AccountId) -> U128 {
        let account_data = self.accounts.get(&account_id).expect("Account data is not found");

        let mut total_accrual: TokensAmount = 0;
        let now: UnixTimestamp = Self::unix_timestamp(env::block_timestamp_ms());

        for (datetime, index) in account_data.accruals.iter() {
            if now - datetime > self.burn_period {
                continue;
            }

            if let Some((accruals, _)) = self.accruals.get(datetime) {
                if let Some(amount) = accruals.get(*index) {
                    total_accrual += *amount;
                }
            }
        }

        U128(total_accrual)
    }

    pub fn is_claim_available(&self, account_id: AccountId) -> ClaimAvailabilityView {
        let account_data = self.accounts.get(&account_id).expect("Account data is not found");

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

    pub fn claim(&mut self) -> PromiseOrValue<U128> {
        let account_id = env::predecessor_account_id();

        require!(
            self.is_claim_available(account_id.clone()) == ClaimAvailabilityView::Available(),
            "Claim is not available at the moment"
        );

        let account_data = self.accounts.get_mut(&account_id).expect("Account data is not found");

        let now: UnixTimestamp = (env::block_timestamp_ms() / 1000) as u32;
        let mut total_accrual: TokensAmount = 0;

        for (datetime, index) in account_data.accruals.iter() {
            if now - datetime > self.burn_period {
                continue;
            }

            if let Some((accruals, total)) = self.accruals.get_mut(datetime) {
                if let Some(amount) = accruals.get_mut(*index) {
                    total_accrual += *amount;
                    *total -= *amount;
                    *amount = 0;
                }
            }
        }

        account_data.accruals.clear();
        account_data.last_claim_at = Some(now);

        let args = json!({
            "receiver_id": account_id,
            "amount": total_accrual.to_string(),
            "memo": "",
        })
        .to_string()
        .as_bytes()
        .to_vec();

        Promise::new(self.token_account_id.clone())
            .function_call("ft_transfer".to_string(), args, 1, Gas(5 * Gas::ONE_TERA.0))
            .into()
    }

    pub fn burn(&mut self) -> PromiseOrValue<U128> {
        require!(
            self.oracles.contains(&env::predecessor_account_id()),
            "Unauthorized access"
        );

        let mut total_to_burn = 0;
        let mut keys_to_remove: Vec<UnixTimestamp> = vec![];
        let now: UnixTimestamp = Self::unix_timestamp(env::block_timestamp_ms());

        for (datetime, (_, total)) in self.accruals.iter() {
            if now - datetime >= self.burn_period {
                keys_to_remove.push(*datetime);
                total_to_burn += total;
            }
        }

        let args = json!({
            "amount": U128(total_to_burn),
        })
        .to_string()
        .as_bytes()
        .to_vec();

        Promise::new(self.token_account_id.clone())
            .function_call("burn".to_string(), args, 0, Gas(5 * Gas::ONE_TERA.0))
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * Gas::ONE_TERA.0))
                    .on_burn(total_to_burn, keys_to_remove),
            )
            .into()
    }

    fn unix_timestamp(ms: u64) -> UnixTimestamp {
        (ms / 1000) as u32
    }
}

#[ext_contract(ext_self)]
pub trait SelfCallback {
    fn on_burn(&mut self, total_to_burn: TokensAmount, keys_to_remove: Vec<UnixTimestamp>) -> U128;
}

#[near_bindgen]
impl SelfCallback for Contract {
    fn on_burn(&mut self, total_to_burn: TokensAmount, keys_to_remove: Vec<UnixTimestamp>) -> U128 {
        if is_promise_success() {
            for datetime in keys_to_remove {
                self.accruals.remove(&datetime);
            }

            U128(total_to_burn)
        } else {
            U128(0)
        }
    }
}

#[cfg(test)]
mod tests {}
