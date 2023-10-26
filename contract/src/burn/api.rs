use model::{BurnApi, TokensAmount, UnixTimestamp};
use near_sdk::{
    env, ext_contract, is_promise_success, json_types::U128, near_bindgen, serde_json::json, Gas, Promise,
    PromiseOrValue,
};

use crate::{common::unix_timestamp, Contract, ContractExt};

#[near_bindgen]
impl BurnApi for Contract {
    fn burn(&mut self) -> PromiseOrValue<U128> {
        self.assert_oracle();

        let mut total_to_burn = 0;
        let mut keys_to_remove: Vec<UnixTimestamp> = vec![];
        let now: UnixTimestamp = unix_timestamp(env::block_timestamp_ms());

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
