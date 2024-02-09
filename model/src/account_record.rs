use std::collections::HashMap;

use near_sdk::{
    borsh,
    borsh::{BorshDeserialize, BorshSerialize},
};

use crate::{AccrualIndex, Asset, UnixTimestamp};

/// Represents the state of a registered account in the smart contract.
///
/// `AccountRecord` maintains the status and history of an individual user's account within
/// the smart contract. It tracks various aspects of the account, such as accrual references,
/// claim history, and operational states.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountRecord {
    /// A list of references to accrual entries in `Contract.accruals`.
    ///
    /// `accruals` contains pairs of timestamps and indices that link to specific accrual
    /// records in the contract's accruals ledger. These references are used to calculate
    /// and verify the user's accrued token amount.
    ///
    /// Here is an illustration of the connection:
    /// ```text
    ///        Contract.accruals:
    ///        ...
    ///        1705066289: ([0.1, 2.3, 5.3, 2.0, 4.3], 14)
    ///  ┌───> 1705066501: ([1.2, 3.4, 8.7, 9.6], 22.9)
    ///  │     ...                      ↑
    ///  │                              │
    ///  │     AccountRecord.accruals:  │
    ///  │     [(1705066501, 2)]        │
    ///  └────────────┘      └──────────┘
    /// ```
    pub accruals: HashMap<Asset, Vec<(UnixTimestamp, AccrualIndex)>>,

    /// Indicates whether the user is authorized to use the contract's features.
    ///
    /// Currently, `is_enabled` is not actively used but is prepared for future releases.
    /// It can be used to enable or disable access to contract functionalities for this
    /// particular account.
    pub is_enabled: bool,

    /// The timestamp of the last event that resets claim period.
    /// It can be either creation of the record or claim operation performed by the account.
    ///
    /// `claim_period_refreshed_at` holds an `UnixTimestamp` that records either the time when
    /// the record was created or when the user last claimed their tokens.
    /// It is used to determine eligibility for future claims.
    pub claim_period_refreshed_at: UnixTimestamp,

    /// Indicates whether there is an active operation on the user's balance.
    ///
    /// `is_locked` is used to signal if the account is currently engaged in an operation
    /// that affects its balance, such as a claim process. This is important for ensuring
    /// the integrity of account operations and preventing concurrent modifications.
    pub is_locked: bool,
}

impl AccountRecord {
    pub fn new(now: UnixTimestamp, asset: Option<Asset>) -> Self {
        let mut accruals = HashMap::new();
        accruals.insert(asset.unwrap_or("SWEAT".to_string()), Vec::new());

        Self {
            accruals,
            is_enabled: true,
            claim_period_refreshed_at: now,
            is_locked: false,
        }
    }

    pub fn get_sweat_accruals(&self) -> Option<&Vec<(UnixTimestamp, AccrualIndex)>> {
        self.accruals.get("SWEAT")
    }

    pub fn get_sweat_accruals_unsafe(&self) -> &Vec<(UnixTimestamp, AccrualIndex)> {
        self.get_sweat_accruals().unwrap()
    }

    pub fn get_sweat_accruals_unsafe_mut(&mut self) -> &mut Vec<(UnixTimestamp, AccrualIndex)> {
        self.accruals.get_mut("SWEAT").unwrap()
    }
}
