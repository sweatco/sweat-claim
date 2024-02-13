use model::{AccrualIndex, UnixTimestamp};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    near_bindgen,
};

use crate::Contract;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountRecordOld {
    pub accruals: Vec<(UnixTimestamp, AccrualIndex)>,
    pub is_enabled: bool,
    pub claim_period_refreshed_at: UnixTimestamp,
    pub is_locked: bool,
}

#[near_bindgen]
impl Contract {
    fn migrate() {}
}
