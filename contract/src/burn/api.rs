use claim_model::{
    api::BurnApi,
    event::{emit, BurnData, EventKind},
    Asset, TokensAmount, UnixTimestamp,
};
use near_sdk::{json_types::U128, near_bindgen, require, PromiseOrValue};

use crate::{
    common::{now_seconds, UnixTimestampExtension},
    get_default_asset, Contract, ContractExt,
};

#[near_bindgen]
impl BurnApi for Contract {
    fn burn(&mut self, asset: Option<Asset>) -> PromiseOrValue<U128> {
        self.assert_oracle();

        require!(!self.is_service_call_running, "Another service call is running");

        self.is_service_call_running = true;

        let asset = asset.unwrap_or(get_default_asset());
        let mut total_to_burn = 0;
        let mut keys_to_remove = vec![];
        let now = now_seconds();

        let accruals = self.accruals.get_accruals(&asset);

        for (datetime, (_, total)) in accruals {
            if !datetime.is_within_period(now, self.burn_period) {
                keys_to_remove.push(*datetime);
                total_to_burn += total;
            }
        }

        if total_to_burn > 0 {
            self.burn_external(asset, total_to_burn, keys_to_remove)
        } else {
            self.is_service_call_running = false;

            PromiseOrValue::Value(U128(0))
        }
    }
}

impl Contract {
    fn on_burn_internal(
        &mut self,
        asset: Asset,
        total_to_burn: TokensAmount,
        keys_to_remove: Vec<UnixTimestamp>,
        is_success: bool,
    ) -> U128 {
        self.is_service_call_running = false;

        if !is_success {
            return U128(0);
        }

        let accruals = self.accruals.get_accruals_mut(&asset);
        for datetime in keys_to_remove {
            accruals.remove(&datetime);
        }

        emit(EventKind::Burn(BurnData {
            burnt_amount: U128(total_to_burn),
        }));

        U128(total_to_burn)
    }
}

#[cfg(not(test))]
pub(crate) mod prod {
    use claim_model::{is_near, Asset, TokensAmount, UnixTimestamp};
    use near_contract_standards::fungible_token::{
        core::{
            ext_ft_core::{self, FungibleTokenCoreExt},
            FungibleTokenCore,
        },
        FungibleToken,
    };
    use near_sdk::{
        env, ext_contract, is_promise_success, json_types::U128, near_bindgen, serde_json::json, AccountId, Gas,
        Promise, PromiseOrValue,
    };

    use crate::{common::AssetExt, Contract, ContractExt};

    #[ext_contract(ext_self)]
    pub trait SelfCallback {
        fn on_burn(&mut self, asset: Asset, total_to_burn: TokensAmount, keys_to_remove: Vec<UnixTimestamp>) -> U128;
    }

    #[near_bindgen]
    impl SelfCallback for Contract {
        #[private]
        fn on_burn(&mut self, asset: Asset, total_to_burn: TokensAmount, keys_to_remove: Vec<UnixTimestamp>) -> U128 {
            self.on_burn_internal(asset, total_to_burn, keys_to_remove, is_promise_success())
        }
    }

    impl Contract {
        pub(crate) fn burn_external(
            &mut self,
            asset: Asset,
            total_to_burn: TokensAmount,
            keys_to_remove: Vec<UnixTimestamp>,
        ) -> PromiseOrValue<U128> {
            self.burn_promise(&asset, total_to_burn)
                .then(
                    ext_self::ext(env::current_account_id())
                        .with_static_gas(Gas(5 * Gas::ONE_TERA.0))
                        .on_burn(asset, total_to_burn, keys_to_remove),
                )
                .into()
        }

        fn burn_promise(&mut self, asset: &Asset, total_to_burn: TokensAmount) -> Promise {
            if is_near(asset) {
                self.burn_near(total_to_burn)
            } else if asset.is_default() {
                self.burn_sweat(total_to_burn)
            } else {
                self.burn_ft(asset, total_to_burn)
            }
        }

        fn burn_sweat(&mut self, total_to_burn: TokensAmount) -> Promise {
            let args = json!({
                "amount": U128(total_to_burn),
            })
            .to_string()
            .as_bytes()
            .to_vec();

            Promise::new(self.token_account_id.clone()).function_call(
                "burn".to_string(),
                args,
                0,
                Gas(5 * Gas::ONE_TERA.0),
            )
        }

        fn burn_ft(&mut self, asset: &Asset, total_to_burn: TokensAmount) -> Promise {
            let contract_account_id = self.get_token_account_id(&asset);
            ext_ft_core::ext(contract_account_id).ft_transfer(
                AccountId::new_unchecked("system".to_string()),
                U128(total_to_burn),
                None,
            )
        }

        fn burn_near(&mut self, total_to_burn: TokensAmount) -> Promise {
            Promise::new(AccountId::new_unchecked("system".to_string())).transfer(total_to_burn)
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use claim_model::{Asset, TokensAmount, UnixTimestamp};
    use near_sdk::{json_types::U128, PromiseOrValue};

    use crate::{common::tests::data::get_test_future_success, Contract};

    pub(crate) const EXT_BURN_FUTURE: &str = "ext_burn";

    impl Contract {
        pub(crate) fn burn_external(
            &mut self,
            asset: Asset,
            total_to_burn: TokensAmount,
            keys_to_remove: Vec<UnixTimestamp>,
        ) -> PromiseOrValue<U128> {
            PromiseOrValue::Value(self.on_burn_internal(
                asset,
                total_to_burn,
                keys_to_remove,
                get_test_future_success(EXT_BURN_FUTURE),
            ))
        }
    }
}
