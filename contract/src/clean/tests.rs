#![cfg(test)]

use claim_model::api::RecordApi;
use near_sdk::json_types::U128;

use crate::{clean::api::CleanApi, common::tests::Context, record::model::versioned::AccountRecord};

#[test]
fn test_clean_single_account_by_oracle() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    context.switch_account(&accounts.oracle);

    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(100_000_000))], None);

    let record: Option<AccountRecord> = contract.accounts.get(&accounts.alice).map(|value| value.clone().into());
    assert!(record.is_some());
    assert_eq!(1, record.unwrap().accruals.len());

    contract.clean(vec![accounts.alice.clone()]);

    let record: Option<AccountRecord> = contract.accounts.get(&accounts.alice).map(|value| value.clone().into());
    assert!(record.is_none());
}

#[test]
#[should_panic(expected = "Unauthorized access")]
fn test_clean_single_account_by_not_oracle() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    context.switch_account(&accounts.oracle);

    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(100_000_000))], None);

    let record: Option<AccountRecord> = contract.accounts.get(&accounts.alice).map(|value| value.clone().into());
    assert!(record.is_some());
    assert_eq!(1, record.unwrap().accruals.len());

    context.switch_account(&accounts.alice);
    contract.clean(vec![accounts.alice.clone()]);
}

#[test]
fn test_clean_multiple_accounts_by_oracle() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    context.switch_account(&accounts.oracle);

    contract.record_batch_for_hold(
        vec![
            (accounts.alice.clone(), U128(100_000_000)),
            (accounts.bob.clone(), U128(1_000_000_000)),
        ],
        None,
    );

    let alice_record: Option<AccountRecord> = contract.accounts.get(&accounts.alice).map(|value| value.clone().into());
    assert!(alice_record.is_some());
    assert_eq!(1, alice_record.unwrap().accruals.len());

    let bob_record: Option<AccountRecord> = contract.accounts.get(&accounts.bob).map(|value| value.clone().into());
    assert!(bob_record.is_some());
    assert_eq!(1, bob_record.unwrap().accruals.len());

    contract.clean(vec![accounts.alice.clone(), accounts.bob.clone()]);

    let alice_record: Option<AccountRecord> = contract.accounts.get(&accounts.alice).map(|value| value.clone().into());
    assert!(alice_record.is_none());

    let bob_record: Option<AccountRecord> = contract.accounts.get(&accounts.bob).map(|value| value.clone().into());
    assert!(bob_record.is_none());
}
