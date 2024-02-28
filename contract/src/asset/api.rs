use std::collections::HashMap;

use claim_model::{api::AssetsApi, Asset};
use near_sdk::{near_bindgen, AccountId};

use crate::{Contract, ContractExt};

#[near_bindgen]
impl AssetsApi for Contract {
    fn get_assets(&self) -> HashMap<Asset, AccountId> {
        self.assets.into_iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    fn register_asset(&mut self, asset: Asset, contract_id: AccountId) {
        self.assert_oracle();
        self.assets.insert(asset, contract_id);
    }
}
