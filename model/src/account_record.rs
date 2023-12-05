use near_sdk::{
    borsh,
    borsh::{BorshDeserialize, BorshSerialize},
};

use crate::{AccrualIndex, UnixTimestamp};

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
    pub accruals: Vec<(UnixTimestamp, AccrualIndex)>,

    /// Indicates whether the user is authorized to use the contract's features.
    ///
    /// Currently, `is_enabled` is not actively used but is prepared for future releases.
    /// It can be used to enable or disable access to contract functionalities for this
    /// particular account.
    pub is_enabled: bool,

    /// The timestamp of the last claim operation performed by the account.
    ///
    /// `last_claim_at` holds an `Option<UnixTimestamp>` that records the time when the user
    /// last claimed their tokens. It is used to determine eligibility for future claims.
    pub last_claim_at: Option<UnixTimestamp>,

    /// Indicates whether there is an active operation on the user's balance.
    ///
    /// `is_locked` is used to signal if the account is currently engaged in an operation
    /// that affects its balance, such as a claim process. This is important for ensuring
    /// the integrity of account operations and preventing concurrent modifications.
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
