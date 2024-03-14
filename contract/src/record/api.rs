use claim_model::{
    api::RecordApi, event::{emit, EventKind::Record, RecordData}, is_near, Asset
};
use near_sdk::{env::{self, panic_str}, json_types::U128, near_bindgen, store::Vector, AccountId};

use crate::{
    common::now_seconds, record::model::versioned::AccountRecord, Contract, ContractExt, StorageKey::AccrualsEntry,
};

#[near_bindgen]
impl RecordApi for Contract {
    #[payable]
    fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>, asset: Option<Asset>) {
        self.assert_oracle();

        let attached_deposit = env::attached_deposit();
        if asset.clone().map(|asset| is_near(&asset)) == Some(true) {
            let total_amount: u128 = amounts.iter().map(|(_, amount)| amount.0).sum();
            assert!(total_amount == attached_deposit, "Total amount does not match attached deposit");
        }

        for (account_id, _) in &amounts {
            self.migrate_record_if_needed(account_id);
        }

        let now_seconds = now_seconds();
        let mut event_data = RecordData::new(now_seconds);

        let balances = if let Some(asset) = asset {
            self.extra_accruals
                .get_mut(&asset)
                .unwrap_or_else(|| panic_str(&format!("Asset '{asset}' doesn't exist")))
                .entry(now_seconds)
        } else {
            self.accruals.entry(now_seconds)
        }
        .or_insert_with(|| (Vector::new(AccrualsEntry(now_seconds)), 0));

        for (account_id, amount) in &amounts {
            event_data.amounts.push((account_id.clone(), *amount));

            balances.1 += amount.0;
            balances.0.push(amount.0);

            if let Some(record) = self.accounts.get_mut(account_id) {
                let accrual = (now_seconds, balances.0.len() - 1);
                record.push(accrual);
            } else {
                let record = AccountRecord {
                    accruals: vec![(now_seconds, balances.0.len() - 1)],
                    ..AccountRecord::new(now_seconds)
                };
                self.accounts.insert(account_id.clone(), record.into_versioned());
            }
        }

        emit(Record(event_data));
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
}
