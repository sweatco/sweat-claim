#![cfg(test)]

use model::api::{ClaimApi, RecordApi};
use near_sdk::{json_types::U128, AccountId};

use crate::common::tests::Context;

#[test]
fn record_by_oracle() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance_1 = 1_000_000;

    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance_1))]);

    let alice_actual_balance = contract.get_claimable_balance_for_account(accounts.alice.clone());
    assert_eq!(alice_balance_1, alice_actual_balance.0);

    context.set_block_timestamp_in_seconds(1_000);

    let alice_balance_2 = 500_000;
    let bob_balance = 200_000;

    contract.record_batch_for_hold(vec![
        (accounts.alice.clone(), U128(alice_balance_2)),
        (accounts.bob.clone(), U128(bob_balance)),
    ]);

    let alice_actual_balance = contract.get_claimable_balance_for_account(accounts.alice.clone());
    assert_eq!(alice_balance_1 + alice_balance_2, alice_actual_balance.0);

    let bob_actual_balance = contract.get_claimable_balance_for_account(accounts.bob.clone());
    assert_eq!(bob_balance, bob_actual_balance.0);
}

#[test]
#[should_panic(expected = "Unauthorized access! Only oracle can do this!")]
fn record_by_not_oracle() {
    let (_context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance_1 = 1_000_000;
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance_1))]);
}

#[test]
fn test_multiple_records_in_the_same_block() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let target_accruals = [10, 20];
    let batches: Vec<Vec<(AccountId, U128)>> = target_accruals
        .iter()
        .map(|&amount| vec![(accounts.alice.clone(), amount.into())])
        .collect();

    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(batches.get(0).unwrap().clone());
    contract.record_batch_for_hold(batches.get(1).unwrap().clone());

    let accruals = contract.get_sweat_accruals_unsafe().get(&0).unwrap();
    assert_eq!(accruals.0.len(), target_accruals.len() as u32);
    assert_eq!(accruals.1, target_accruals.iter().sum::<u128>());
}
