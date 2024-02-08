use near_sdk::{
    env::{current_account_id, predecessor_account_id},
    require, AccountId,
};

use crate::Contract;

impl Contract {
    pub(crate) fn assert_oracle(&self) {
        require!(
            self.oracles.contains(&predecessor_account_id()),
            "Unauthorized access! Only oracle can do this!"
        );
    }

    pub(crate) fn assert_oracle_or_registered_token_contract(&self) {
        require!(
            self.oracles.contains(&predecessor_account_id())
                || self.is_registered_token_account(predecessor_account_id()),
            "Unauthorized access! Only oracle or owner can do this!"
        );
    }

    pub(crate) fn assert_private() {
        require!(current_account_id() == predecessor_account_id(), "Method is private",);
    }

    fn is_registered_token_account(&self, token_account_id: AccountId) -> bool {
        self.token_account_ids.values().any(|value| *value == token_account_id)
    }
}
