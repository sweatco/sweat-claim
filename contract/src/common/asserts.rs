use crate::*;

impl Contract {
    pub(crate) fn assert_oracle(&self) {
        require!(
            self.oracles.contains(&env::predecessor_account_id()),
            "Unauthorized access"
        );
    }
}
