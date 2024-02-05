use std::collections::HashMap;

use model::{
    api::ClaimApi,
    event::{emit, ClaimData, EventKind},
    ClaimAllResultView, ClaimAvailabilityView, ClaimResultView,
};
use near_sdk::{
    env, env::log_str, ext_contract, is_promise_success, json_types::U128, require, serde_json::json, Gas, Promise,
    PromiseOrValue,
};

use crate::{claim::api::AssetDetails, common::now_seconds, Contract, StorageKey::AccrualsEntry, *};

type AssetClaim = (AssetAbbreviation, AssetDetails);

#[near_bindgen]
impl Contract {
    pub fn claim_all(&mut self) -> PromiseOrValue<ClaimAllResultView> {
        let account_id = env::predecessor_account_id();

        require!(
            self.is_claim_available(account_id.clone()) == ClaimAvailabilityView::Available,
            "Claim is not available at the moment",
        );

        let details = self.collect_accruals(&account_id);
        self.accounts
            .get_mut(&account_id)
            .expect("Account data is not found")
            .accruals
            .clear();

        if details.is_empty() {
            PromiseOrValue::Value(ClaimAllResultView::default())
        } else {
            let (head, tail) = details.split_first().unwrap();
            self.transfer(
                now_seconds(),
                account_id.clone(),
                head.clone(),
                tail.to_vec(),
                ClaimAllResultView::default(),
            )
        }
    }

    fn collect_accruals(&mut self, account_id: &AccountId) -> Vec<AssetClaim> {
        let account_data = self.accounts.get_mut(account_id).expect("Account data is not found");

        let now = now_seconds();
        let mut details: HashMap<AssetAbbreviation, AssetDetails> = HashMap::new();

        for (datetime, index) in &account_data.accruals {
            if now - *datetime > self.burn_period {
                continue;
            }

            let Some((accruals, total, asset)) = self.accruals.get_mut(datetime) else {
                continue;
            };

            let Some(amount) = accruals.get_mut(*index) else {
                continue;
            };

            if !details.contains_key(asset) {
                details.insert(asset.clone(), AssetDetails::default());
            }

            log_str(format!("Add {amount:?} for {asset}").as_str());

            let details = details.get_mut(asset).unwrap();
            details.0.push((*datetime, *amount));
            details.1 += *amount;
            *total -= *amount;
            *amount = 0;
        }

        details
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }

    fn transfer(
        &mut self,
        now: UnixTimestamp,
        account_id: AccountId,
        head: AssetClaim,
        tail: Vec<AssetClaim>,
        result: ClaimAllResultView,
    ) -> PromiseOrValue<ClaimAllResultView> {
        let args = Self::compose_transfer_arguments(&account_id, head.1 .1);
        let token_account_id = self.get_token_account_id(&head.0);

        Promise::new(token_account_id)
            .function_call("ft_transfer".to_string(), args, 1, Gas(5 * Gas::ONE_TERA.0))
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * Gas::ONE_TERA.0))
                    .on_asset_transfer(now, account_id, head, tail, result),
            )
            .into()
    }

    fn compose_transfer_arguments(account_id: &AccountId, amount: TokensAmount) -> Vec<u8> {
        json!({
            "receiver_id": account_id.clone(),
            "amount": amount.to_string(),
            "memo": "",
        })
        .to_string()
        .as_bytes()
        .to_vec()
    }

    fn handle_transfer_result(
        &mut self,
        is_success: bool,
        now: UnixTimestamp,
        account_id: AccountId,
        head: AssetClaim,
    ) -> (AssetAbbreviation, bool, Option<ClaimResultView>) {
        let account = self.accounts.get_mut(&account_id).expect("Account not found");

        let asset = &head.0;
        let details = &head.1 .0;
        let amount = head.1 .1;

        let result = if is_success {
            account.claim_period_refreshed_at = now;

            let event_data = ClaimData {
                account_id,
                asset: asset.clone(),
                details: details
                    .iter()
                    .map(|(timestamp, amount)| (*timestamp, U128(*amount)))
                    .collect(),
                total_claimed: U128(amount),
            };
            emit(EventKind::Claim(event_data));

            Some(ClaimResultView::new(head.1 .1))
        } else {
            for (timestamp, amount) in details {
                let daily_accruals = self
                    .accruals
                    .entry(*timestamp)
                    .or_insert_with(|| (Vector::new(AccrualsEntry(*timestamp)), 0, asset.clone()));

                daily_accruals.0.push(*amount);
                daily_accruals.1 += amount;

                account.accruals.push((*timestamp, daily_accruals.0.len() - 1));
            }

            None
        };

        (asset.clone(), is_success, result)
    }
}

#[ext_contract(ext_self)]
pub trait Callbacks {
    fn on_asset_transfer(
        &mut self,
        now: UnixTimestamp,
        account_id: AccountId,
        head: AssetClaim,
        tail: Vec<AssetClaim>,
        result: ClaimAllResultView,
    ) -> PromiseOrValue<ClaimAllResultView>;
}

#[near_bindgen]
impl Callbacks for Contract {
    fn on_asset_transfer(
        &mut self,
        now: UnixTimestamp,
        account_id: AccountId,
        head: AssetClaim,
        tail: Vec<AssetClaim>,
        result: ClaimAllResultView,
    ) -> PromiseOrValue<ClaimAllResultView> {
        let is_success = is_promise_success();

        let step_result = self.handle_transfer_result(is_success, now, account_id.clone(), head);

        let mut result = result.0;
        result.push(step_result);
        let result_view = ClaimAllResultView(result);

        if tail.is_empty() {
            PromiseOrValue::Value(result_view)
        } else {
            let (head, tail) = tail.split_first().unwrap();
            self.transfer(now, account_id, head.clone(), tail.to_vec(), result_view)
        }
    }
}
