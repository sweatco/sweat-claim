use claim_model::{
    api::RecordApi,
    event::{emit, EventKind::Record, RecordData},
    is_near, Asset,
};
use near_sdk::{
    env::{self, panic_str},
    json_types::U128,
    near_bindgen,
    store::Vector,
    AccountId,
};

use crate::{
    common::{now_seconds, AssetExt},
    get_default_asset,
    record::model::versioned::AccountRecord,
    Contract, ContractExt,
    StorageKey::AccrualsEntry,
};

#[near_bindgen]
impl RecordApi for Contract {
    #[payable]
    fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>, asset: Option<Asset>) {
        self.assert_oracle();

        let attached_deposit = env::attached_deposit();
        if asset.clone().map(|asset| is_near(&asset)) == Some(true) {
            let total_amount: u128 = amounts.iter().map(|(_, amount)| amount.0).sum();
            assert!(
                total_amount == attached_deposit,
                "Total amount does not match attached deposit"
            );
        }

        self.record_batch_for_hold_internal(amounts, asset);
    }
}

impl Contract {
    #[allow(deprecated)]
    pub(crate) fn migrate_record_if_needed(&mut self, account_id: &AccountId) {
        if let Some(record_legacy) = self.accounts_legacy.get(account_id) {
            let record: AccountRecord = record_legacy.clone().into();
            self.accounts.insert(account_id.clone(), record.into_versioned());
            self.accounts_legacy.remove(account_id);
        }
    }

    #[allow(deprecated)]
    pub(crate) fn get_account(&self, account_id: &AccountId) -> Option<AccountRecord> {
        if let Some(account) = self.accounts_legacy.get(account_id) {
            Some(account.clone().into())
        } else {
            self.accounts.get(account_id).map(|value| value.clone().into())
        }
    }

    pub(crate) fn record_batch_for_hold_internal(&mut self, amounts: Vec<(AccountId, U128)>, asset: Option<Asset>) {
        let asset = asset.unwrap_or(get_default_asset()).normalize();

        for (account_id, _) in &amounts {
            self.migrate_record_if_needed(account_id);
        }

        let now_seconds = now_seconds();
        let mut event_data = RecordData::new(now_seconds);

        let balances = if asset.is_default() {
            &mut self.accruals
        } else {
            self.extra_accruals
                .get_mut(&asset)
                .expect(format!("Asset {asset} not found").as_str())
        }
        .entry(now_seconds)
        .or_insert_with(|| (Vector::new(AccrualsEntry(now_seconds)), 0));

        for (account_id, amount) in &amounts {
            event_data.amounts.push((account_id.clone(), *amount));

            balances.1 += amount.0;
            balances.0.push(amount.0);

            if let Some(record) = self.accounts.get_mut(account_id) {
                let accrual = (now_seconds, balances.0.len() - 1);
                record.push(accrual, &asset);
            } else {
                let mut record = AccountRecord::new(now_seconds).into_versioned();
                record.push((now_seconds, balances.0.len() - 1), &asset);
                self.accounts.insert(account_id.clone(), record);
            }
        }

        emit(Record(event_data));
    }
}
