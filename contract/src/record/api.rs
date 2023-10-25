use crate::*;

trait RecordApi {
    fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>);
}

#[near_bindgen]
impl RecordApi for Contract {
    fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>) {
        log_str(&format!("Record batch: {amounts:?}"));

        require!(
            self.oracles.contains(&env::predecessor_account_id()),
            "Unauthorized access! Only oracle can do this!"
        );

        let now_seconds: UnixTimestamp = (env::block_timestamp_ms() / 1_000) as u32;
        let mut balances: Vector<TokensAmount> = Vector::new(AccrualsEntry(now_seconds));
        let mut total_balance: TokensAmount = 0;

        for (account_id, amount) in amounts {
            log_str(&format!("Record {amount:?} for {account_id}"));

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
