use model::{api::ClaimApi, ClaimAvailabilityView, TokensAmount, UnixTimestamp};
use near_sdk::{env, json_types::U128, near_bindgen, require, store::Vector, AccountId, PromiseOrValue};

use crate::{common::now_seconds, Contract, ContractExt, StorageKey::AccrualsEntry};

#[near_bindgen]
impl ClaimApi for Contract {
    fn get_claimable_balance_for_account(&self, account_id: AccountId) -> U128 {
        let Some(account_data) = self.accounts.get(&account_id) else {
            return U128(0);
        };

        let mut total_accrual = 0;
        let now = now_seconds();

        for (datetime, index) in &account_data.accruals {
            if now - datetime > self.burn_period {
                continue;
            }

            let Some((accruals, _)) = self.accruals.get(datetime) else {
                continue;
            };

            if let Some(amount) = accruals.get(*index) {
                total_accrual += *amount;
            }
        }

        U128(total_accrual)
    }

    fn is_claim_available(&self, account_id: AccountId) -> ClaimAvailabilityView {
        let Some(account_data) = self.accounts.get(&account_id) else {
            return ClaimAvailabilityView::Unregistered;
        };

        let Some(last_claim_at) = account_data.last_claim_at else {
            return ClaimAvailabilityView::Available;
        };

        if now_seconds() - last_claim_at > self.claim_period {
            ClaimAvailabilityView::Available
        } else {
            ClaimAvailabilityView::Unavailable((last_claim_at, self.claim_period))
        }
    }

    fn claim(&mut self) -> PromiseOrValue<U128> {
        let account_id = env::predecessor_account_id();

        require!(
            self.is_claim_available(account_id.clone()) == ClaimAvailabilityView::Available,
            "Claim is not available at the moment"
        );

        let account_data = self.accounts.get_mut(&account_id).expect("Account data is not found");

        let now = now_seconds();
        let mut total_accrual = 0;
        let mut details = vec![];

        for (datetime, index) in &account_data.accruals {
            if now - datetime > self.burn_period {
                continue;
            }

            let Some((accruals, total)) = self.accruals.get_mut(datetime) else {
                continue;
            };

            let Some(amount) = accruals.get_mut(*index) else {
                continue;
            };

            details.push((*datetime, *amount));

            total_accrual += *amount;
            *total -= *amount;
            *amount = 0;
        }

        account_data.accruals.clear();

        if total_accrual > 0 {
            self.transfer_external(account_id, total_accrual, details)
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }
}

impl Contract {
    fn on_transfer_internal(
        &mut self,
        account_id: AccountId,
        total_accrual: TokensAmount,
        details: Vec<(UnixTimestamp, TokensAmount)>,
        is_success: bool,
    ) -> U128 {
        let account = self.accounts.get_mut(&account_id).expect("Account not found");

        if is_success {
            account.last_claim_at = Some(now_seconds());

            U128(total_accrual)
        } else {
            for (timestamp, amount) in details {
                if !self.accruals.contains_key(&timestamp) {
                    self.accruals
                        .insert(timestamp, (Vector::new(AccrualsEntry(timestamp)), 0));
                }

                let daily_accruals = self.accruals.get_mut(&timestamp).unwrap();
                daily_accruals.0.push(amount);
                daily_accruals.1 += amount;

                account.accruals.push((timestamp, daily_accruals.0.len() - 1));
            }

            U128(0)
        }
    }
}

#[cfg(not(test))]
mod not_test {
    use model::{TokensAmount, UnixTimestamp};
    use near_sdk::{
        env, ext_contract, is_promise_success, json_types::U128, near_bindgen, serde_json::json, AccountId, Gas,
        Promise, PromiseOrValue,
    };

    use crate::{Contract, ContractExt};

    #[ext_contract(ext_self)]
    pub trait SelfCallback {
        fn on_transfer(
            &mut self,
            account_id: AccountId,
            total_accrual: TokensAmount,
            details: Vec<(UnixTimestamp, TokensAmount)>,
        ) -> U128;
    }

    #[near_bindgen]
    impl SelfCallback for Contract {
        fn on_transfer(
            &mut self,
            account_id: AccountId,
            total_accrual: TokensAmount,
            details: Vec<(UnixTimestamp, TokensAmount)>,
        ) -> U128 {
            self.on_transfer_internal(account_id, total_accrual, details, is_promise_success())
        }
    }

    impl Contract {
        pub(crate) fn transfer_external(
            &mut self,
            account_id: AccountId,
            total_accrual: TokensAmount,
            details: Vec<(UnixTimestamp, TokensAmount)>,
        ) -> PromiseOrValue<U128> {
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
                .then(
                    ext_self::ext(env::current_account_id())
                        .with_static_gas(Gas(5 * Gas::ONE_TERA.0))
                        .on_transfer(account_id, total_accrual, details),
                )
                .into()
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use model::{TokensAmount, UnixTimestamp};
    use near_sdk::{json_types::U128, AccountId, PromiseOrValue};

    use crate::{common::tests::data::get_test_future_success, Contract};

    pub(crate) const EXT_TRANSFER_FUTURE: &str = "ext_transfer";

    impl Contract {
        pub(crate) fn transfer_external(
            &mut self,
            account_id: AccountId,
            total_accrual: TokensAmount,
            details: Vec<(UnixTimestamp, TokensAmount)>,
        ) -> PromiseOrValue<U128> {
            PromiseOrValue::Value(self.on_transfer_internal(
                account_id,
                total_accrual,
                details,
                get_test_future_success(EXT_TRANSFER_FUTURE),
            ))
        }
    }
}
