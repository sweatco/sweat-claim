#![cfg(test)]

use model::api::{ClaimApi, RecordApi};
use near_sdk::{json_types::U128, test_utils::accounts};

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
#[should_panic(expected = "Record for this timestamp: 0 already existed. It was owerwritten.")]
fn test_record() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), 10.into())]);
    contract.record_batch_for_hold(vec![(accounts.alice, 10.into())]);
}
