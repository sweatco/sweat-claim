#![cfg(test)]

use model::api::ClaimApi;

use crate::common::tests::Context;

#[test]
fn test_claim_when_user_has_zero_balance() {
    let (_, contract, accounts) = Context::init_with_oracle();

    let alice_new_balance = contract.get_claimable_balance_for_account(accounts.alice).0;
    assert_eq!(0, alice_new_balance);
}
