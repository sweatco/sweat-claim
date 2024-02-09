use model::{
    account_record::AccountRecord,
    api::RecordApi,
    event::{emit, EventKind, RecordData},
};
use near_sdk::{json_types::U128, near_bindgen, store::Vector, AccountId};

use crate::{common::now_seconds, Contract, ContractExt, StorageKey::AccrualsEntry};

#[near_bindgen]
impl RecordApi for Contract {
    fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>) {
        self.assert_oracle();

        let now_seconds = now_seconds();
        let mut balances = Vector::new(AccrualsEntry(now_seconds));
        let mut total_balance = 0;

        let mut event_data = RecordData {
            timestamp: now_seconds,
            amounts: vec![],
        };

        for (account_id, amount) in amounts {
            event_data.amounts.push((account_id.clone(), amount));

            let amount = amount.0;
            let index = balances.len();

            total_balance += amount;
            balances.push(amount);

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

        if self.accruals.contains_key(&now_seconds) {
            let current_accruals = self.accruals.get_mut(&now_seconds).unwrap();
            current_accruals.0.extend(balances.iter().cloned());
            current_accruals.1 += total_balance;
        } else {
            self.accruals.insert(now_seconds, (balances, total_balance));
        }

        emit(EventKind::Record(event_data));
    }
}
