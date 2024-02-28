use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use crate::record::model::v1::AccountRecordV1;

#[derive(BorshDeserialize, BorshSerialize)]
pub(crate) enum AccountRecordVersioned {
    V1(AccountRecordV1),
}

type AccountRecord = AccountRecordV1;

impl AccountRecord {
    pub fn into_versioned(self) -> AccountRecordVersioned {
        AccountRecordVersioned::V1(self)
    }
}
