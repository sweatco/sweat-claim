use near_sdk::{Balance, env::{attached_deposit, current_account_id, predecessor_account_id}, json_types::U128, require};

use crate::Contract;

impl Contract {
    pub(crate) fn assert_oracle(&self) {
        require!(
            self.oracles.contains(&predecessor_account_id()),
            "Unauthorized access! Only oracle can do this!"
        );
    }

    pub(crate) fn assert_private() {
        require!(current_account_id() == predecessor_account_id(), "Method is private",);
    }

    pub(crate) fn assert_deposit(amount: Balance) {
        require!(
            amount == attached_deposit(),
            "Attached deposit does not match the total amount of tokens."
        );
    }
}
