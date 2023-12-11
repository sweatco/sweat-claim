use model::{
    api::BurnApi,
    event::{emit, BurnData, EventKind},
    TokensAmount, UnixTimestamp,
};
use near_sdk::{json_types::U128, near_bindgen, require, PromiseOrValue};

use crate::{common::now_seconds, Contract, ContractExt};

#[near_bindgen]
impl BurnApi for Contract {
    fn burn(&mut self) -> PromiseOrValue<U128> {
        self.assert_oracle();

        require!(!self.is_service_call_running, "Another service call is running");

        self.is_service_call_running = true;

        let mut total_to_burn = 0;
        let mut keys_to_remove = vec![];
        let now = now_seconds();

        for (datetime, (_, total)) in self.accruals.iter() {
            if now - datetime >= self.burn_period {
                keys_to_remove.push(*datetime);
                total_to_burn += total;
            }
        }

        if total_to_burn > 0 {
            self.burn_external(total_to_burn, keys_to_remove)
        } else {
            self.is_service_call_running = false;

            PromiseOrValue::Value(U128(0))
        }
    }
}

impl Contract {
    fn on_burn_internal(
        &mut self,
        total_to_burn: TokensAmount,
        keys_to_remove: Vec<UnixTimestamp>,
        is_success: bool,
    ) -> U128 {
        self.is_service_call_running = false;

        if !is_success {
            return U128(0);
        }

        for datetime in keys_to_remove {
            self.accruals.remove(&datetime);
        }

        emit(EventKind::Burn(BurnData {
            burnt_amount: U128(total_to_burn),
        }));

        U128(total_to_burn)
    }
}

#[cfg(not(test))]
pub(crate) mod prod {
    use model::{TokensAmount, UnixTimestamp};
    use near_sdk::{
        env, ext_contract, is_promise_success, json_types::U128, near_bindgen, serde_json::json, Gas, Promise,
        PromiseOrValue,
    };

    use crate::{Contract, ContractExt};

    #[ext_contract(ext_self)]
    pub trait SelfCallback {
        fn on_burn(&mut self, total_to_burn: TokensAmount, keys_to_remove: Vec<UnixTimestamp>) -> U128;
    }

    #[near_bindgen]
    impl SelfCallback for Contract {
        #[private]
        fn on_burn(&mut self, total_to_burn: TokensAmount, keys_to_remove: Vec<UnixTimestamp>) -> U128 {
            self.on_burn_internal(total_to_burn, keys_to_remove, is_promise_success())
        }
    }

    impl Contract {
        pub(crate) fn burn_external(
            &mut self,
            total_to_burn: TokensAmount,
            keys_to_remove: Vec<UnixTimestamp>,
        ) -> PromiseOrValue<U128> {
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
}

#[cfg(test)]
pub(crate) mod test {
    use model::{TokensAmount, UnixTimestamp};
    use near_sdk::{json_types::U128, PromiseOrValue};

    use crate::{common::tests::data::get_test_future_success, Contract};

    pub(crate) const EXT_BURN_FUTURE: &str = "ext_burn";

    impl Contract {
        pub(crate) fn burn_external(
            &mut self,
            total_to_burn: TokensAmount,
            keys_to_remove: Vec<UnixTimestamp>,
        ) -> PromiseOrValue<U128> {
            PromiseOrValue::Value(self.on_burn_internal(
                total_to_burn,
                keys_to_remove,
                get_test_future_success(EXT_BURN_FUTURE),
            ))
        }
    }
}
