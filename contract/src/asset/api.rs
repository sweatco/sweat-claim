use std::collections::HashMap;

use claim_model::{api::AssetsApi, Asset};
use near_sdk::{env::log_str, near_bindgen, store::UnorderedMap, AccountId};

use crate::{common::AssetExt, get_default_asset, Contract, ContractExt, StorageKey};

#[near_bindgen]
impl AssetsApi for Contract {
    fn get_assets(&self) -> HashMap<Asset, AccountId> {
        self.assets.into_iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    fn register_asset(&mut self, asset: Asset, contract_id: AccountId) {
        let asset = asset.normalize();

        self.assert_oracle();
        self.assets.insert(asset.clone(), contract_id);
        self.extra_accruals
            .insert(asset.clone(), UnorderedMap::new(StorageKey::ExtraAccrualsEntry(asset)));
    }
}

impl Contract {
    pub(crate) fn get_token_account_id(&self, asset: &Asset) -> AccountId {
        let asset = asset.normalize();
        if asset.is_default() {
            self.token_account_id.clone()
        } else {
            self.assets.get(&asset).expect("Asset not found").clone()
        }
    }
}
