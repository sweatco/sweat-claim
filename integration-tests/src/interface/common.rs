use integration_utils::integration_contract::IntegrationContract;
use near_sdk::AccountId;

use crate::interface::{claim_contract::SweatClaim, ft_contract::SweatFt};

pub(crate) trait ContractAccount {
    fn account(&self) -> AccountId;
}

impl ContractAccount for SweatClaim<'_> {
    fn account(&self) -> AccountId {
        AccountId::new_unchecked(self.contract().as_account().id().to_string())
    }
}

impl ContractAccount for SweatFt<'_> {
    fn account(&self) -> AccountId {
        AccountId::new_unchecked(self.contract().as_account().id().to_string())
    }
}
