use near_sdk::{env, require};

use crate::Contract;

impl Contract {
    pub(crate) fn assert_oracle(&self) {
        require!(
            self.oracles.contains(&env::predecessor_account_id()),
            "Unauthorized access"
        );
    }
}
