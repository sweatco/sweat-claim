use model::{
    account_record::AccountRecord,
    api::RecordApi,
    event::{emit, BatchedRecordData, EventKind, RecordData},
};
use near_sdk::{json_types::U128, near_bindgen, store::Vector, AccountId};

use crate::{common::now_seconds, Contract, ContractExt, StorageKey::AccrualsEntry};

const RECORD_EVENT_BATCH_SIZE: usize = 100;

#[near_bindgen]
impl RecordApi for Contract {
    fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>) {
        self.assert_oracle();

        let now_seconds = now_seconds();
        let mut batched_event_data = vec![RecordData::new(now_seconds)];

        if !self.accruals.contains_key(&now_seconds) {
            self.accruals
                .insert(now_seconds, (Vector::new(AccrualsEntry(now_seconds)), 0));
        }

        let balances = self.accruals.get_mut(&now_seconds).unwrap();

        for (account_id, amount) in amounts {
            batched_event_data.push_amount((account_id.clone(), amount), now_seconds, RECORD_EVENT_BATCH_SIZE);

            let amount = amount.0;
            let index = balances.0.len();

            balances.1 += amount;
            balances.0.push(amount);

            if let Some(record) = self.accounts.get_mut(&account_id) {
                record.accruals.push((now_seconds, index));
            } else {
                let record = AccountRecord {
                    accruals: vec![(now_seconds, index)],
                    ..AccountRecord::new(now_seconds)
                };

                self.accounts.insert(account_id, record);
            }
        }

        for event_data in batched_event_data {
            emit(EventKind::Record(event_data));
        }
    }
}
