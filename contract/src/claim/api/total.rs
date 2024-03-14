use std::collections::HashMap;

use claim_model::{
    api::ClaimApi,
    event::{emit, ClaimData, EventKind},
    AccrualsReference, ClaimAllResultView, ClaimAvailabilityView, ClaimResultView, TokensAmount, UnixTimestamp,
};
use near_sdk::{
    env, env::log_str, ext_contract, json_types::U128, require, serde_json::json, store::Vector, Gas, Promise,
    PromiseOrValue,
};

use crate::{
    claim::model::{Claim, ClaimDetails},
    common::{is_promise_success, now_seconds},
    record::model::versioned::AccountRecord,
    Contract,
    StorageKey::AccrualsEntry,
    *,
};

const EXT_TRANSFER_ALL_FUTURE: &str = "ext_transfer_all";

#[near_bindgen]
impl Contract {
    pub fn claim_all(&mut self) -> PromiseOrValue<ClaimAllResultView> {
        let account_id = env::predecessor_account_id();

        require!(
            self.is_claim_available(account_id.clone()) == ClaimAvailabilityView::Available,
            "Claim is not available at the moment",
        );

        let details = self.collect_accruals(&account_id);

        self.clear_all_accruals(&account_id);

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

    fn collect_accruals(&mut self, account_id: &AccountId) -> Vec<Claim> {
        let AccountRecordVersioned::V1(account_data) = self
            .accounts
            .get(account_id)
            .expect("Account data is not found")
            .clone();

        let now = now_seconds();
        let details = &mut HashMap::<Asset, ClaimDetails>::new();

        self.collect_claim_details(details, now, &get_default_asset(), &account_data.accruals);
        for (asset, accruals) in &account_data.extra_accruals {
            self.collect_claim_details(details, now, asset, accruals);
        }

        details
            .iter()
            .map(|(key, value)| Claim::new(key.clone(), value.clone()))
            .collect()
    }

    #[cfg(not(test))]
    fn transfer(
        &mut self,
        now: UnixTimestamp,
        account_id: AccountId,
        head: Claim,
        tail: Vec<Claim>,
        result: ClaimAllResultView,
    ) -> PromiseOrValue<ClaimAllResultView> {
        let args = Self::compose_transfer_arguments(&account_id, head.details.total);
        let token_account_id = self.get_token_account_id(&head.asset);

        Promise::new(token_account_id)
            .function_call("ft_transfer".to_string(), args, 1, Gas(5 * Gas::ONE_TERA.0))
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * Gas::ONE_TERA.0))
                    .on_asset_transfer(now, account_id, head, tail, result),
            )
            .into()
    }

    #[cfg(test)]
    fn transfer(
        &mut self,
        now: UnixTimestamp,
        account_id: AccountId,
        head: Claim,
        tail: Vec<Claim>,
        result: ClaimAllResultView,
    ) -> PromiseOrValue<ClaimAllResultView> {
        self.on_asset_transfer(now, account_id, head, tail, result)
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
        head: Claim,
    ) -> ClaimResultView {
        let asset = &head.asset;
        let details = &head.details.accruals;
        let amount = head.details.total;

        let result = if is_success {
            self.get_account_mut(&account_id).claim_period_refreshed_at = now;

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

            Some(amount)
        } else {
            for (timestamp, amount) in details {
                self.push_accrual(asset, timestamp, amount);

                let accrual_index = self.get_daily_accruals(asset, timestamp).0.len() - 1;
                self.get_account_mut(&account_id)
                    .accruals
                    .push((*timestamp, accrual_index));
            }

            None
        };

        ClaimResultView::new(asset.clone(), is_success, result)
    }

    fn push_accrual(&mut self, asset: &Asset, timestamp: &UnixTimestamp, amount: &TokensAmount) {
        let daily_accruals = self.get_daily_accruals_mut(asset, timestamp);
        daily_accruals.0.push(*amount);
        daily_accruals.1 += amount;
    }
}

impl Contract {
    fn get_account_mut(&mut self, account_id: &AccountId) -> &mut AccountRecord {
        let AccountRecordVersioned::V1(account) = self.accounts.get_mut(&account_id).expect("Account not found");

        account
    }

    fn get_daily_accruals(&self, asset: &Asset, timestamp: &UnixTimestamp) -> &(Vector<TokensAmount>, TokensAmount) {
        self.get_accruals(asset)
            .get(timestamp)
            .expect("Daily accruals not found")
    }

    fn get_daily_accruals_mut(
        &mut self,
        asset: &Asset,
        timestamp: &UnixTimestamp,
    ) -> &mut (Vector<TokensAmount>, TokensAmount) {
        self.get_accruals_mut(asset)
            .entry(*timestamp)
            .or_insert_with(|| (Vector::new(AccrualsEntry(*timestamp)), 0))
    }

    fn get_accruals(&self, asset: &Asset) -> &AccrualsMap {
        if *asset == get_default_asset() {
            &self.accruals
        } else {
            self.extra_accruals.get(asset).expect("Asset not found")
        }
    }

    fn get_accruals_mut(&mut self, asset: &Asset) -> &mut AccrualsMap {
        if *asset == get_default_asset() {
            &mut self.accruals
        } else {
            self.extra_accruals.get_mut(asset).expect("Asset not found")
        }
    }

    fn clear_all_accruals(&mut self, account_id: &AccountId) {
        let AccountRecordVersioned::V1(account_data) =
            self.accounts.get_mut(account_id).expect("Account data is not found");

        account_data.accruals.clear();
        account_data.extra_accruals.clear();
    }

    fn collect_claim_details(
        &mut self,
        acc: &mut HashMap<Asset, ClaimDetails>,
        now: UnixTimestamp,
        asset: &Asset,
        accruals: &AccrualsReference,
    ) {
        for (datetime, index) in accruals {
            if now - *datetime > self.burn_period {
                continue;
            }

            let Some((accruals, total)) = self.accruals.get_mut(datetime) else {
                continue;
            };

            let Some(amount) = accruals.get_mut(*index) else {
                continue;
            };

            if !acc.contains_key(asset) {
                acc.insert(asset.clone(), ClaimDetails::default());
            }

            log_str(format!("Add {amount:?} for {asset}").as_str());

            let details = acc.get_mut(asset).unwrap();
            details.accruals.push((*datetime, *amount));
            details.total += *amount;
            *total -= *amount;
            *amount = 0;
        }
    }
}

#[ext_contract(ext_self)]
pub trait Callbacks {
    fn on_asset_transfer(
        &mut self,
        now: UnixTimestamp,
        account_id: AccountId,
        head: Claim,
        tail: Vec<Claim>,
        result: ClaimAllResultView,
    ) -> PromiseOrValue<ClaimAllResultView>;
}

#[near_bindgen]
impl Callbacks for Contract {
    fn on_asset_transfer(
        &mut self,
        now: UnixTimestamp,
        account_id: AccountId,
        head: Claim,
        tail: Vec<Claim>,
        mut result: ClaimAllResultView,
    ) -> PromiseOrValue<ClaimAllResultView> {
        let is_success = is_promise_success(EXT_TRANSFER_ALL_FUTURE);

        let step_result = self.handle_transfer_result(is_success, now, account_id.clone(), head);

        result.push(step_result);

        if tail.is_empty() {
            PromiseOrValue::Value(result)
        } else {
            let (head, tail) = tail.split_first().unwrap();
            self.transfer(now, account_id, head.clone(), tail.to_vec(), result)
        }
    }
}
