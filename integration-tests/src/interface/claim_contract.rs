use anyhow::Result;
use async_trait::async_trait;
use integration_utils::integration_contract::IntegrationContract;
use model::{
    AuthApiIntegration, BurnApiIntegration, ClaimApiIntegration, ClaimAvailabilityView, Duration, InitApiIntegration,
    RecordApiIntegration,
};
use near_sdk::{json_types::U128, AccountId};
use serde_json::json;
use workspaces::{Account, Contract};

pub const SWEAT_CLAIM: &str = "sweat_claim";

pub struct SweatClaim<'a> {
    contract: &'a Contract,
    account: Option<Account>,
}

#[async_trait]
impl InitApiIntegration for SweatClaim<'_> {
    async fn init(&self, token_account_id: near_sdk::AccountId) -> Result<()> {
        self.call_contract(
            "init",
            json!({
                "token_account_id": token_account_id,
            }),
        )
        .await
    }

    async fn set_claim_period(&mut self, period: Duration) -> Result<()> {
        self.call_contract(
            "set_claim_period",
            json!({
                "period": period,
            }),
        )
        .await
    }

    async fn set_burn_period(&mut self, period: Duration) -> Result<()> {
        self.call_contract(
            "set_burn_period",
            json!({
                "period": period,
            }),
        )
        .await
    }
}

#[async_trait]
impl AuthApiIntegration for SweatClaim<'_> {
    async fn add_oracle(&mut self, account_id: AccountId) -> Result<()> {
        self.call_contract(
            "add_oracle",
            json!({
                "account_id": account_id,
            }),
        )
        .await
    }

    async fn remove_oracle(&mut self, account_id: AccountId) -> Result<()> {
        self.call_contract(
            "remove_oracle",
            json!({
                "account_id": account_id,
            }),
        )
        .await
    }

    async fn get_oracles(&self) -> Result<Vec<AccountId>> {
        self.call_contract("get_oracles", ()).await
    }
}

#[async_trait]
impl BurnApiIntegration for SweatClaim<'_> {
    async fn burn(&mut self) -> Result<U128> {
        self.call_user("burn", ()).await
    }
}

#[async_trait]
impl RecordApiIntegration for SweatClaim<'_> {
    async fn record_batch_for_hold(&mut self, amounts: Vec<(AccountId, U128)>) -> Result<()> {
        self.call_contract(
            "record_batch_for_hold",
            json!({
                "amounts": amounts,
            }),
        )
        .await
    }
}

#[async_trait]
impl ClaimApiIntegration for SweatClaim<'_> {
    async fn get_claimable_balance_for_account(&self, account_id: AccountId) -> Result<U128> {
        self.call_contract(
            "get_balance_for_account",
            json!({
                "account_id": account_id,
            }),
        )
        .await
    }

    async fn is_claim_available(&self, account_id: AccountId) -> Result<ClaimAvailabilityView> {
        self.call_contract(
            "is_claim_available",
            json!({
                "account_id": account_id,
            }),
        )
        .await
    }

    async fn claim(&mut self) -> Result<()> {
        self.call_user("claim", ()).await
    }
}

impl<'a> IntegrationContract<'a> for SweatClaim<'a> {
    fn with_contract(contract: &'a Contract) -> Self {
        Self {
            contract,
            account: None,
        }
    }

    fn with_user(mut self, account: &Account) -> Self {
        self.account = account.clone().into();
        self
    }

    fn user_account(&self) -> Account {
        self.account
            .as_ref()
            .expect("Set account with `user` method first")
            .clone()
    }

    fn contract(&self) -> &'a Contract {
        self.contract
    }
}
