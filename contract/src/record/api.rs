use model::{account_record::AccountRecord, api::RecordApi, TokensAmount};
use near_sdk::{env, json_types::U128, near_bindgen, store::Vector, AccountId};

use crate::{common::unix_timestamp, Contract, ContractExt, StorageKey::AccrualsEntry};

#[near_bindgen]
impl RecordApi for Contract {
    fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>) {
        self.assert_oracle();

        let now_seconds = unix_timestamp(env::block_timestamp_ms());
        let mut balances: Vector<TokensAmount> = Vector::new(AccrualsEntry(now_seconds));
        let mut total_balance: TokensAmount = 0;

        for (account_id, amount) in amounts {
            let amount = amount.0;

            total_balance += amount;
            balances.push(amount);
            let index = balances.len() - 1;

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

        self.accruals.insert(now_seconds, (balances, total_balance));
    }
}
