use claim_model::{AccrualIndex, Asset, UnixTimestamp};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use crate::{common::AssetExt, record::model::v1::AccountRecordV1};

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub(crate) enum AccountRecordVersioned {
    V1(AccountRecordV1),
}

impl AccountRecordVersioned {
    pub(crate) fn push(&mut self, accrual: (UnixTimestamp, AccrualIndex), asset: &Asset) {
        match self {
            AccountRecordVersioned::V1(record) => if asset.is_default() {
                &mut record.accruals
            } else {
                record.extra_accruals
                    .get_mut(asset)
                    .expect(format!("Asset {asset} is not registered").as_str())
            }.push(accrual),
        }
    }
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
