use async_trait::async_trait;
use model::{ClaimAvailabilityView, Duration};
use near_sdk::json_types::U128;
use serde_json::{json, Value};
use workspaces::{Account, AccountId, Contract};

use crate::interface::common::ContractAccount;

pub(crate) trait ClaimContract: ClaimContractInterface + ContractAccount {}

impl ClaimContract for Contract {}

#[async_trait]
pub(crate) trait ClaimContractInterface {
    async fn init(&self, token_contract_account: &Account) -> anyhow::Result<()>;

    async fn set_claim_period(&self, period: Duration) -> anyhow::Result<()>;

    async fn set_burn_period(&self, period: Duration) -> anyhow::Result<()>;

    async fn get_balance_for_account(&self, account: &Account) -> anyhow::Result<U128>;

    async fn add_oracle(&self, account_id: &AccountId) -> anyhow::Result<()>;

    async fn is_claim_available(&self, account_id: &AccountId) -> anyhow::Result<ClaimAvailabilityView>;

    async fn claim(&self, account: &Account) -> anyhow::Result<()>;
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

    async fn set_claim_period(&self, period: Duration) -> anyhow::Result<()> {
        println!("▶️ Set claim period to {period} seconds");

        self.call("set_claim_period")
            .args_json(json!({
                "period": period,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(())
    }

    async fn set_burn_period(&self, period: Duration) -> anyhow::Result<()> {
        println!("▶️ Set burn period to {period} seconds");

        self.call("set_burn_period")
            .args_json(json!({
                "period": period,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(())
    }

    async fn get_balance_for_account(&self, account: &Account) -> anyhow::Result<U128> {
        println!("▶️ View balance for account {:?}", account.id());

        let args = json!({
            "account_id": account.id(),
        });

        let result = self.view("get_balance_for_account").args_json(args).await?.json()?;

        println!("   ✅ {:?}", result);

        Ok(result)
    }

    async fn add_oracle(&self, account_id: &AccountId) -> anyhow::Result<()> {
        println!("▶️ Add oracle: {account_id}");

        let result = self
            .call("add_oracle")
            .args_json(json!({
                "account_id": account_id,
            }))
            .max_gas()
            .transact()
            .await?
            .into_result();

        println!("Result: {:?}", result);

        Ok(())
    }

    async fn is_claim_available(&self, account_id: &AccountId) -> anyhow::Result<ClaimAvailabilityView> {
        println!("▶️ Is claim available for {account_id}");

        let result: ClaimAvailabilityView = self
            .view("is_claim_available")
            .args_json(json!({
                "account_id": account_id,
            }))
            .await?
            .json()?;

        println!("Result: {:?}", result);

        Ok(result)
    }

    async fn claim(&self, account: &Account) -> anyhow::Result<()> {
        println!("▶️ Claim");

        let result = account
            .call(self.id(), "claim")
            .args_json(json!({}))
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        println!("Result: {:?}", result);

        Ok(())
    }
}
