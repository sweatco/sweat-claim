use crate::{common::unix_timestamp, *};

pub trait ClaimApi {
    fn get_claimable_balance_for_account(&self, account_id: AccountId) -> U128;

    fn is_claim_available(&self, account_id: AccountId) -> ClaimAvailabilityView;

    fn claim(&mut self) -> PromiseOrValue<U128>;
}

#[near_bindgen]
impl ClaimApi for Contract {
    fn get_claimable_balance_for_account(&self, account_id: AccountId) -> U128 {
        // TODO: return value when no account data
        let account_data = self.accounts.get(&account_id).expect("Account data is not found");

        let mut total_accrual: TokensAmount = 0;
        let now: UnixTimestamp = unix_timestamp(env::block_timestamp_ms());

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

    fn is_claim_available(&self, account_id: AccountId) -> ClaimAvailabilityView {
        // TODO: return value when no account data
        let account_data = self.accounts.get(&account_id).expect("Account data is not found");

        let Some(last_claim_at) = account_data.last_claim_at else {
            return ClaimAvailabilityView::Available;
        };

        let now_seconds = (env::block_timestamp_ms() / 1_000) as u32;

        if now_seconds - last_claim_at > self.claim_period {
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

        let now: UnixTimestamp = (env::block_timestamp_ms() / 1000) as u32;
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
