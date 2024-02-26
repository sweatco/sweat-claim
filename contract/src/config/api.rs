use claim_model::{api::ConfigApi, Duration};
use near_sdk::near_bindgen;

use crate::{Contract, ContractExt};

#[near_bindgen]
impl ConfigApi for Contract {
    fn set_claim_period(&mut self, period: Duration) {
        self.assert_oracle();

        self.claim_period = period;
    }

    fn set_burn_period(&mut self, period: Duration) {
        self.assert_oracle();

        self.burn_period = period;
    }
}
