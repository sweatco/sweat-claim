use near_sdk::{test_utils::VMContextBuilder, testing_env, AccountId};

use crate::{auth::api::AuthApi, Contract};

#[test]
fn add_oracle_by_contract_owner() {
    Context::run_test(|mut context, mut contract, accounts| {
        context.switch_account(&accounts.owner);
        contract.add_oracle(accounts.oracle.clone());

        let oracles = contract.get_oracles();
        assert_eq!(oracles, vec![accounts.oracle.clone()]);
    });
}

#[test]
#[should_panic(expected = "Method is private")]
fn add_oracle_not_by_contract_owner() {
    Context::run_test(|mut context, mut contract, accounts| {
        context.switch_account(&accounts.alice);
        contract.add_oracle(accounts.oracle.clone());
    });
}

#[test]
#[should_panic(expected = "Already exists")]
fn add_oracle_twice() {
    Context::run_test(|mut context, mut contract, accounts| {
        context.switch_account(&accounts.owner);
        contract.add_oracle(accounts.oracle.clone());
        contract.add_oracle(accounts.oracle.clone());
    });
}

#[test]
fn remove_oracle_by_contract_owner() {
    Context::run_test(|mut context, mut contract, accounts| {
        context.switch_account(&accounts.owner);
        contract.add_oracle(accounts.oracle.clone());

        let oracles = contract.get_oracles();
        assert_eq!(oracles, vec![accounts.oracle.clone()]);

        contract.remove_oracle(accounts.oracle.clone());

        let oracles = contract.get_oracles();
        assert!(oracles.is_empty());
    });
}

#[test]
#[should_panic(expected = "Method is private")]
fn remove_oracle_not_by_contract_owner() {
    Context::run_test(|mut context, mut contract, accounts| {
        contract.oracles.insert(accounts.oracle.clone());

        context.switch_account(&accounts.alice);
        contract.remove_oracle(accounts.oracle.clone());
    });
}

#[test]
#[should_panic(expected = "No such oracle")]
fn remove_not_existing_oracle() {
    Context::run_test(|mut context, mut contract, accounts| {
        context.switch_account(&accounts.owner);
        contract.remove_oracle(accounts.oracle.clone());
    });
}

struct Context {
    builder: VMContextBuilder,
}

impl Context {
    fn run_test<F>(test: F)
    where
        F: Fn(Context, Contract, &TestAccounts),
    {
        let accounts = TestAccounts::default();
        let token_account = accounts.token.clone();

        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts.owner.clone())
            .signer_account_id(accounts.owner.clone())
            .predecessor_account_id(accounts.owner.clone())
            .block_timestamp(0);

        testing_env!(builder.build());

        let contract = Contract::init(token_account);
        let context = Context { builder };

        test(context, contract, &accounts);
    }

    pub(crate) fn switch_account(&mut self, account_id: &AccountId) {
        self.builder
            .predecessor_account_id(account_id.clone())
            .signer_account_id(account_id.clone());
        testing_env!(self.builder.build());
    }
}

struct TestAccounts {
    pub alice: AccountId,
    pub bob: AccountId,
    pub oracle: AccountId,
    pub token: AccountId,
    pub owner: AccountId,
}

impl Default for TestAccounts {
    fn default() -> Self {
        Self {
            alice: AccountId::new_unchecked("alice".to_string()),
            bob: AccountId::new_unchecked("bob".to_string()),
            oracle: AccountId::new_unchecked("oracle".to_string()),
            token: AccountId::new_unchecked("token".to_string()),
            owner: AccountId::new_unchecked("owner".to_string()),
        }
    }
}
