use model::{
    account_record::AccountRecord,
    api::RecordApi,
    event::{emit, EventKind, RecordData},
    is_near, AccrualsExt, Asset, BatchedAccruals,
};
use near_sdk::{env, json_types::U128, near_bindgen, require, store::Vector, AccountId};

use crate::{common::now_seconds, Contract, ContractExt, StorageKey::AccrualsEntry};

#[near_bindgen]
impl RecordApi for Contract {
    #[payable]
    fn record_batch_for_hold(&mut self, amounts: BatchedAccruals, asset: Option<Asset>) {
        self.assert_oracle();

        let asset = asset.unwrap_or("SWEAT".to_string());

        if is_near(&asset) {
            Self::assert_deposit(amounts.total_amount());
        }

        let now_seconds = now_seconds();
        let mut balances = Vector::new(AccrualsEntry(now_seconds));
        let mut total_balance = 0;

        let mut event_data = RecordData {
            timestamp: now_seconds,
            asset: asset.clone(),
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

        let existing = self.accruals.insert(now_seconds, (balances, total_balance, asset));

        emit(EventKind::Record(event_data));

        require!(
            existing.is_none(),
            format!("Record for this timestamp: {now_seconds} already existed. It was overwritten.")
        );
    }
}
