#![cfg(test)]

use model::{
    api::{ClaimApi, RecordApi},
    Asset, ClaimAllResultView, ClaimAvailabilityView, TokensAmount, UnixTimestamp,
};
use near_sdk::{json_types::U128, AccountId, PromiseOrValue};

use crate::{
    claim::api::test::EXT_TRANSFER_FUTURE,
    common::tests::{data::set_test_future_success, Context},
};

#[test]
fn test_check_claim_availability_when_user_is_not_registered() {
    let (_, contract, accounts) = Context::init_with_oracle();

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(0, alice_new_balance);

    let alice_can_claim = contract.is_claim_available(accounts.alice);
    assert_eq!(ClaimAvailabilityView::Unregistered, alice_can_claim);
}

#[test]
fn test_check_claim_availability_when_user_has_tokens_and_claim_period_after_claim_is_not_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance = 400_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(alice_balance, alice_new_balance);

    let claim_timestamp = contract.claim_period as u64 + 100;
    context.set_block_timestamp_in_seconds(claim_timestamp);
    context.switch_account(&accounts.alice);
    contract.claim(None);

    let check_timestamp = claim_timestamp + 10;
    context.set_block_timestamp_in_seconds(check_timestamp);

    let alice_can_claim = contract.is_claim_available(accounts.alice.clone());
    assert_eq!(
        alice_can_claim,
        ClaimAvailabilityView::Unavailable((claim_timestamp as UnixTimestamp, contract.claim_period))
    );
}

#[test]
fn test_check_claim_availability_when_user_has_tokens_and_claim_period_after_claim_is_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance = 300_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(alice_balance, alice_new_balance);

    let claim_timestamp = contract.claim_period as u64 + 100;
    context.set_block_timestamp_in_seconds(claim_timestamp);
    context.switch_account(&accounts.alice);
    contract.claim(None);

    let check_timestamp = claim_timestamp + contract.claim_period as u64 + 100;
    context.set_block_timestamp_in_seconds(check_timestamp);

    let alice_can_claim = contract.is_claim_available(accounts.alice.clone());
    assert_eq!(alice_can_claim, ClaimAvailabilityView::Available);
}

#[test]
fn test_check_claim_availability_when_user_has_tokens_and_claim_period_after_record_creation_is_not_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance = 400_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(alice_balance, alice_new_balance);

    let alice_can_claim = contract.is_claim_available(accounts.alice.clone());
    assert_eq!(
        alice_can_claim,
        ClaimAvailabilityView::Unavailable((0, contract.claim_period))
    );
}

#[test]
fn test_check_claim_availability_when_user_has_tokens_and_claim_period_after_record_creation_is_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let alice_balance = 300_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    context.set_block_timestamp_in_seconds(contract.claim_period as u64 + 100);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(alice_balance, alice_new_balance);

    let alice_can_claim = contract.is_claim_available(accounts.alice.clone());
    assert_eq!(alice_can_claim, ClaimAvailabilityView::Available);
}

#[test]
#[should_panic(expected = "Claim is not available at the moment")]
fn test_claim_when_user_is_not_registered() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, true);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(0, alice_new_balance);

    context.switch_account(&accounts.alice);
    contract.claim(None);
}

#[test]
#[should_panic(expected = "Claim is not available at the moment")]
fn test_claim_when_user_has_tokens_and_claim_period_is_not_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, true);

    let alice_balance = 200_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    context.switch_account(&accounts.alice);
    contract.claim(None);
}

#[test]
fn test_claim_when_user_has_tokens_and_claim_period_is_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, true);

    let alice_balance = 700_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    context.set_block_timestamp_in_seconds(contract.claim_period as u64 + 100);

    context.switch_account(&accounts.alice);
    let claimed_amount = match contract.claim(None) {
        PromiseOrValue::Promise(_) => panic!("Expected value"),
        PromiseOrValue::Value(value) => value,
    };
    assert_eq!(alice_balance, claimed_amount.total.unwrap().0);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(0, alice_new_balance);
}

#[test]
fn test_claim_when_user_has_tokens_and_burn_period_is_passed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, true);

    let alice_balance = 12_000_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    context.set_block_timestamp_in_seconds(contract.burn_period as u64 + 100);

    context.switch_account(&accounts.alice);
    let claimed_amount = match contract.claim(None) {
        PromiseOrValue::Promise(_) => panic!("Expected value"),
        PromiseOrValue::Value(value) => value,
    };
    assert_eq!(0, claimed_amount.total.unwrap().0);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(0, alice_new_balance);
}

#[test]
fn test_claim_when_user_has_tokens_and_claim_period_is_passed_and_transfer_failed() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, false);

    let alice_balance = 123_100_000;
    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance))], None);

    context.set_block_timestamp_in_seconds(contract.claim_period as u64 + 100);

    context.switch_account(&accounts.alice);
    let claimed_amount = match contract.claim(None) {
        PromiseOrValue::Promise(_) => panic!("Expected value"),
        PromiseOrValue::Value(value) => value,
    };
    assert_eq!(0, claimed_amount.total.unwrap().0);

    let alice_new_balance = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(alice_balance, alice_new_balance);
}

#[test]
fn test_claim_different_assets() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, true);

    contract.register_token("TOK".into(), AccountId::new_unchecked("another.token".to_string()));

    let alice_balance_sweat = 700_000;
    let alice_balance_another_token = 1_000_000;

    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance_sweat))], None);
    context.set_block_timestamp_in_seconds(1);
    contract.record_batch_for_hold(
        vec![(accounts.alice.clone(), U128(alice_balance_another_token))],
        Some("TOK".to_string()),
    );

    context.set_block_timestamp_in_seconds(contract.claim_period as u64 + 100);

    context.switch_account(&accounts.alice);
    let claimed_amount = match contract.claim(None) {
        PromiseOrValue::Promise(_) => panic!("Expected value"),
        PromiseOrValue::Value(value) => value,
    };
    assert_eq!(alice_balance_sweat, claimed_amount.total.unwrap().0);

    let alice_new_balance_sweat = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(0, alice_new_balance_sweat);

    let alice_new_balance_another_token = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), Some("TOK".into()))
        .0;
    assert_eq!(alice_balance_another_token, alice_new_balance_another_token);

    context.set_block_timestamp_in_seconds(3 * contract.claim_period as u64);

    let claimed_amount = match contract.claim(Some("TOK".into())) {
        PromiseOrValue::Promise(_) => panic!("Expected value"),
        PromiseOrValue::Value(value) => value,
    };
    assert_eq!(alice_balance_another_token, claimed_amount.total.unwrap().0);

    let alice_new_balance_another_token = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), Some("TOK".into()))
        .0;
    assert_eq!(0, alice_new_balance_another_token);
}

#[test]
fn test_claim_all_assets() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();
    set_test_future_success(EXT_TRANSFER_FUTURE, true);

    contract.register_token("USDT".into(), AccountId::new_unchecked("another.token".to_string()));

    let alice_balance_sweat = 1_000_000;
    let alice_balance_usdt = 200_000_000;

    context.switch_account(&accounts.oracle);
    contract.record_batch_for_hold(vec![(accounts.alice.clone(), U128(alice_balance_sweat))], None);
    context.set_block_timestamp_in_seconds(1);
    contract.record_batch_for_hold(
        vec![(accounts.alice.clone(), U128(alice_balance_usdt))],
        Some("USDT".to_string()),
    );

    context.set_block_timestamp_in_seconds(contract.claim_period as u64 + 100);

    context.switch_account(&accounts.alice);
    let claimed_amount = match contract.claim_all() {
        PromiseOrValue::Promise(_) => panic!("Expected value"),
        PromiseOrValue::Value(value) => value,
    };

    let claimed_sweat = claimed_amount.total_for_asset("SWEAT".into());
    assert_eq!(alice_balance_sweat, claimed_sweat);

    let claimed_usdt = claimed_amount.total_for_asset("USDT".into());
    assert_eq!(alice_balance_usdt, claimed_usdt);

    let alice_new_balance_sweat = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), None)
        .0;
    assert_eq!(0, alice_new_balance_sweat);

    let alice_new_balance_tok = contract
        .get_claimable_balance_for_account(accounts.alice.clone(), Some("USDT".into()))
        .0;
    assert_eq!(0, alice_new_balance_tok);
}

trait ClaimAllResultViewExt {
    fn total_for_asset(&self, asset: Asset) -> TokensAmount;
}

impl ClaimAllResultViewExt for ClaimAllResultView {
    fn total_for_asset(&self, asset: Asset) -> TokensAmount {
        self.iter().find(|x| x.asset == asset).unwrap().total.unwrap().0
    }
}
