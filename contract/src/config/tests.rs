#![cfg(test)]

use model::api::ConfigApi;

use crate::common::tests::Context;

#[test]
fn set_claim_period_by_oracle() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let claim_period = 1_000_000;
    context.switch_account(&accounts.oracle);
    contract.set_claim_period(claim_period);

    assert_eq!(claim_period, contract.claim_period);

    let claim_period = 3_000_000;
    context.switch_account(&accounts.oracle);
    contract.set_claim_period(claim_period);

    assert_eq!(claim_period, contract.claim_period);
}

#[test]
#[should_panic(expected = "Unauthorized access")]
fn set_claim_period_by_not_oracle() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let claim_period = 1_000_000;
    context.switch_account(&accounts.alice);
    contract.set_claim_period(claim_period);
}

#[test]
fn set_burn_period_by_oracle() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let burn_period = 1_000_000;
    context.switch_account(&accounts.oracle);
    contract.set_burn_period(burn_period);

    assert_eq!(burn_period, contract.burn_period);

    let burn_period = 3_000_000;
    context.switch_account(&accounts.oracle);
    contract.set_burn_period(burn_period);

    assert_eq!(burn_period, contract.burn_period);
}

#[test]
#[should_panic(expected = "Unauthorized access")]
fn set_burn_period_by_not_oracle() {
    let (mut context, mut contract, accounts) = Context::init_with_oracle();

    let burn_period = 1_000_000;
    context.switch_account(&accounts.alice);
    contract.set_burn_period(burn_period);
}
