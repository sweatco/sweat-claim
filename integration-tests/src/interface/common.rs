use workspaces::{Account, Contract};

pub(crate) trait ContractAccount {
    fn account(&self) -> &Account;
}

impl ContractAccount for Contract {
    fn account(&self) -> &Account {
        self.as_account()
    }
}
