use model::{
    account_record::AccountRecord,
    api::RecordApi,
    event::{emit, EventKind::Record, RecordData},
};
use near_sdk::{json_types::U128, near_bindgen, store::Vector, AccountId};

use crate::{common::now_seconds, Contract, ContractExt, StorageKey::AccrualsEntry};

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

        emit(Record(event_data));
    }
}
