use claim_model::{
    api::RecordApi,
    event::{emit, EventKind::Record, RecordData},
};
use near_sdk::{AccountId, json_types::U128, near_bindgen, store::Vector};

use crate::{common::now_seconds, Contract, ContractExt, StorageKey::AccrualsEntry};
use crate::record::model::legacy::AccountRecordLegacy;

#[near_bindgen]
impl RecordApi for Contract {
    fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>) {
        self.assert_oracle();

        let now_seconds = now_seconds();
        let mut event_data = RecordData::new(now_seconds);

        let balances = self
            .accruals
            .entry(now_seconds)
            .or_insert_with(|| (Vector::new(AccrualsEntry(now_seconds)), 0));

        for (account_id, amount) in amounts {
            event_data.amounts.push((account_id.clone(), amount));

            let amount = amount.0;
            let index = balances.0.len();

            balances.1 += amount;
            balances.0.push(amount);

            if let Some(record) = self.accounts_legacy.get_mut(&account_id) {
                record.accruals.push((now_seconds, index));
            } else {
                let record = AccountRecordLegacy {
                    accruals: vec![(now_seconds, index)],
                    ..AccountRecordLegacy::new(now_seconds)
                };

                self.accounts_legacy.insert(account_id, record);
            }
        }

        emit(Record(event_data));
    }
}
