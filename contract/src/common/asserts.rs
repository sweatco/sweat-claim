use near_sdk::{env, require};

use crate::Contract;

impl Contract {
    pub(crate) fn assert_oracle(&self) {
        require!(
            self.oracles.contains(&env::predecessor_account_id()),
            "Unauthorized access! Only oracle can do this!"
        );
    }

    pub(crate) fn assert_private() {
        require!(
            env::current_account_id() == env::predecessor_account_id(),
            "Method is private",
        );
    }
}
