#![cfg(test)]

use model::api::AuthApi;

use crate::common::tests::Context;

#[test]
fn add_oracle_by_contract_owner() {
    let (mut context, mut contract, accounts) = Context::init();
    context.switch_account(&accounts.owner);
    contract.add_oracle(accounts.oracle.clone());

    let oracles = contract.get_oracles();
    assert_eq!(oracles, vec![accounts.oracle.clone()]);
}

#[test]
#[should_panic(expected = "Method is private")]
fn add_oracle_not_by_contract_owner() {
    let (mut context, mut contract, accounts) = Context::init();

    context.switch_account(&accounts.alice);
    contract.add_oracle(accounts.oracle.clone());
}

#[test]
#[should_panic(expected = "Already exists")]
fn add_oracle_twice() {
    let (mut context, mut contract, accounts) = Context::init();

    context.switch_account(&accounts.owner);
    contract.add_oracle(accounts.oracle.clone());
    contract.add_oracle(accounts.oracle.clone());
}

#[test]
fn remove_oracle_by_contract_owner() {
    let (mut context, mut contract, accounts) = Context::init();

    context.switch_account(&accounts.owner);
    contract.add_oracle(accounts.oracle.clone());

    let oracles = contract.get_oracles();
    assert_eq!(oracles, vec![accounts.oracle.clone()]);

    contract.remove_oracle(accounts.oracle.clone());

    let oracles = contract.get_oracles();
    assert!(oracles.is_empty());
}

#[test]
#[should_panic(expected = "Method is private")]
fn remove_oracle_not_by_contract_owner() {
    let (mut context, mut contract, accounts) = Context::init();

    contract.oracles.insert(accounts.oracle.clone());

    context.switch_account(&accounts.alice);
    contract.remove_oracle(accounts.oracle.clone());
}

#[test]
#[should_panic(expected = "No such oracle")]
fn remove_not_existing_oracle() {
    let (mut context, mut contract, accounts) = Context::init();

    context.switch_account(&accounts.owner);
    contract.remove_oracle(accounts.oracle.clone());
}
