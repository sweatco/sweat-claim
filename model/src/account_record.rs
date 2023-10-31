use near_sdk::{
    borsh,
    borsh::{BorshDeserialize, BorshSerialize},
};

use crate::{AccrualIndex, UnixTimestamp};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountRecord {
    pub accruals: Vec<(UnixTimestamp, AccrualIndex)>,
    pub is_enabled: bool,
    pub last_claim_at: Option<UnixTimestamp>,
    pub is_locked: bool,
}

impl Default for AccountRecord {
    fn default() -> Self {
        Self {
            accruals: Vec::new(),
            is_enabled: true,
            last_claim_at: None,
            is_locked: false,
        }
    }
}
