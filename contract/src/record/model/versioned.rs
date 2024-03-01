use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use crate::record::model::v1::AccountRecordV1;

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub(crate) enum AccountRecordVersioned {
    V1(AccountRecordV1),
}

impl Into<AccountRecord> for AccountRecordVersioned {
    fn into(self) -> AccountRecord {
        match self {
            AccountRecordVersioned::V1(record) => record,
        }
    }
}

pub(crate) type AccountRecord = AccountRecordV1;

impl AccountRecord {
    pub fn into_versioned(self) -> AccountRecordVersioned {
        AccountRecordVersioned::V1(self)
    }
}
