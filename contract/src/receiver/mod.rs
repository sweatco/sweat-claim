use claim_model::{api::RecordApi, Asset};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{
    env,
    json_types::U128,
    near_bindgen,
    serde::{Deserialize, Serialize},
    serde_json, AccountId, PromiseOrValue,
};

use crate::{Contract, ContractExt};

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde", tag = "type", content = "data", rename_all = "snake_case")]
pub enum FtMessage {
    BatchRecord(Vec<(AccountId, U128)>),
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        let ft_message: FtMessage = serde_json::from_str(&msg).expect("Unable to deserialize msg");

        match ft_message {
            FtMessage::BatchRecord(batch) => {
                let total = batch.iter().fold(0, |acc, (_, amount)| acc + amount.0);
                assert_eq!(total, amount.0, "Total amount doesn't match");

                let asset = self
                    .get_asset_for_account(env::predecessor_account_id())
                    .expect("Unknown asset");
                self.record_batch_for_hold_internal(batch, Some(asset));
            }
        }

        PromiseOrValue::Value(U128(0))
    }
}

impl Contract {
    fn get_asset_for_account(&self, account_id: AccountId) -> Option<Asset> {
        self.assets
            .iter()
            .find(|(_, id)| **id == account_id)
            .map(|(asset, _)| asset.clone())
    }
}
