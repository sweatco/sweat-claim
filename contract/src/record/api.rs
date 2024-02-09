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
                record.get_sweat_accruals_unsafe_mut().push((now_seconds, index));
            } else {
                let mut record = AccountRecord::new(now_seconds, None);
                record.get_sweat_accruals_unsafe_mut().push((now_seconds, index));

                self.accounts.insert(account_id, record);
            }
        }

        let sweat_accruals = self.get_sweat_accruals_unsafe_mut();
        if sweat_accruals.contains_key(&now_seconds) {
            let current_accruals = sweat_accruals.get_mut(&now_seconds).unwrap();
            current_accruals.0.extend(balances.iter().cloned());
            current_accruals.1 += total_balance;
        } else {
            sweat_accruals.insert(now_seconds, (balances, total_balance));
        }

        emit(EventKind::Record(event_data));
    }
}
