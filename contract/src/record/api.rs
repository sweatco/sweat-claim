use model::{account_record::AccountRecord, api::RecordApi};
use near_sdk::{env::log_str, json_types::U128, near_bindgen, require, store::Vector, AccountId};

use crate::{common::now_seconds, Contract, ContractExt, StorageKey::AccrualsEntry};

#[near_bindgen]
impl RecordApi for Contract {
    fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>) {
        self.assert_oracle();

        self.assert_oracle();

        let now_seconds = now_seconds();
        let mut balances = Vector::new(AccrualsEntry(now_seconds));
        let mut total_balance = 0;

        for (account_id, amount) in amounts {
            let amount = amount.0;
            let index = balances.len();

            total_balance += amount;
            balances.push(amount);

            if let Some(record) = self.accounts.get_mut(&account_id) {
                record.accruals.push((now_seconds, index));
            } else {
                let record = AccountRecord {
                    last_claim_at: Some(now_seconds),
                    accruals: vec![(now_seconds, index)],
                    ..Default::default()
                };

                self.accounts.insert(account_id, record);
            }
        }

        let existing = self.accruals.insert(now_seconds, (balances, total_balance));

        require!(
            existing.is_none(),
            "Record for this timestamp: {now_seconds} already existed. It was owerwritten."
        );
    }
}
