use model::{api::ClaimApi, ClaimAvailabilityView, TokensAmount, UnixTimestamp};
use near_sdk::{
    env, json_types::U128, near_bindgen, require, serde_json::json, AccountId, Gas, Promise, PromiseOrValue,
};

use crate::{common::unix_timestamp, Contract, ContractExt};

#[near_bindgen]
impl ClaimApi for Contract {
    fn get_claimable_balance_for_account(&self, account_id: AccountId) -> U128 {
        if let Some(account_data) = self.accounts.get(&account_id) {
            let mut total_accrual: TokensAmount = 0;
            let now: UnixTimestamp = unix_timestamp(env::block_timestamp_ms());

            for (datetime, index) in &account_data.accruals {
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
        } else {
            U128(0)
        }
    }

    fn is_claim_available(&self, account_id: AccountId) -> ClaimAvailabilityView {
        if let Some(account_data) = self.accounts.get(&account_id) {
            let Some(last_claim_at) = account_data.last_claim_at else {
                return ClaimAvailabilityView::Available;
            };

            let now_seconds = unix_timestamp(env::block_timestamp_ms());

            if now_seconds - last_claim_at > self.claim_period {
                ClaimAvailabilityView::Available
            } else {
                ClaimAvailabilityView::Unavailable((last_claim_at, self.claim_period))
            }
        } else {
            ClaimAvailabilityView::Unregistered
        }
    }

    fn claim(&mut self) -> PromiseOrValue<()> {
        let account_id = env::predecessor_account_id();

        require!(
            self.is_claim_available(account_id.clone()) == ClaimAvailabilityView::Available,
            "Claim is not available at the moment"
        );

        let account_data = self.accounts.get_mut(&account_id).expect("Account data is not found");

        let now = unix_timestamp(env::block_timestamp_ms());
        let mut total_accrual: TokensAmount = 0;

        for (datetime, index) in &account_data.accruals {
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

        // TODO: add error handling in callback
        Promise::new(self.token_account_id.clone())
            .function_call("ft_transfer".to_string(), args, 1, Gas(5 * Gas::ONE_TERA.0))
            .into()
    }
}
