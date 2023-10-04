use crate::interface::common::ContractAccount;
use async_trait::async_trait;
use serde_json::json;
use workspaces::{Account, Contract};

pub(crate) trait ClaimContract: ClaimContractInterface + ContractAccount {}

impl ClaimContract for Contract {}

#[async_trait]
pub(crate) trait ClaimContractInterface {
    async fn init(&self, token_contract_account: &Account) -> anyhow::Result<()>;
}

#[async_trait]
impl ClaimContractInterface for Contract {
    async fn init(&self, token_contract_account: &Account) -> anyhow::Result<()> {
        println!("▶️ Init claim contract");

        self.call("init")
            .args_json(json!({
                "token_account_id": token_contract_account.id(),
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(())
    }
}
