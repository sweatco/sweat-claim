#![cfg(test)]

use model::{
    api::{ClaimApi, RecordApi},
    AssetAbbreviation,
};
use near_sdk::{json_types::U128, AccountId};

use crate::common::tests::Context;

#[test]
fn record_by_oracle() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance_1 = 1_000_000;

    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance_1))], None);

    let alice_actual_balance = contract.get_claimable_balance_for_account(accounts.alice.clone(), None);
    assert_eq!(alice_balance_1, alice_actual_balance.0);

    context.set_block_timestamp_in_seconds(1_000);

    let alice_balance_2 = 500_000;
    let bob_balance = 200_000;

    contract.record_batch_for_hold(
        vec![
            (accounts.alice.clone(), U128(alice_balance_2)),
            (accounts.bob.clone(), U128(bob_balance)),
        ],
        None,
    );

    let alice_actual_balance = contract.get_claimable_balance_for_account(accounts.alice.clone(), None);
    assert_eq!(alice_balance_1 + alice_balance_2, alice_actual_balance.0);

    let bob_actual_balance = contract.get_claimable_balance_for_account(accounts.bob.clone(), None);
    assert_eq!(bob_balance, bob_actual_balance.0);
}

#[test]
#[should_panic(expected = "Unauthorized access! Only oracle can do this!")]
fn record_by_not_oracle() {
    let (_context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance_1 = 1_000_000;
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance_1))], None);
}

#[test]
#[should_panic(expected = "Record for this timestamp: 0 already existed. It was overwritten.")]
fn test_record() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), 10.into())], None);
    contract.record_batch_for_hold(vec![(accounts.alice, 10.into())], None);
}

#[test]
fn record_for_different_tokens() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let another_token: (AssetAbbreviation, AccountId) = (
        "TOK".to_string(),
        AccountId::new_unchecked("another.token.testnet".to_string()),
    );

    contract.register_token(another_token.0.clone(), another_token.1);

    let alice_balance_1_sweat = 1_000_000;
    let alice_balance_1_another_token = 5_000_000;

    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance_1_sweat))], None);

    context.set_block_timestamp_in_seconds(1);
    contract.record_batch_for_hold(
        vec![(accounts.alice.clone(), U128(alice_balance_1_another_token))],
        Some(another_token.0.clone()),
    );

    let alice_actual_balance_sweat = contract.get_claimable_balance_for_account(accounts.alice.clone(), None);
    assert_eq!(alice_balance_1_sweat, alice_actual_balance_sweat.0);

    let alice_actual_balance_another_token =
        contract.get_claimable_balance_for_account(accounts.alice.clone(), Some(another_token.0.clone()));
    assert_eq!(alice_balance_1_another_token, alice_actual_balance_another_token.0);
}
