use async_trait::async_trait;
use integration_utils::{integration_contract::IntegrationContract, misc::ToNear};
use model::{
    api::{AuthApiIntegration, ConfigApiIntegration, InitApiIntegration},
    Duration,
};
use near_sdk::json_types::U128;
use near_workspaces::Account;
use sweat_integration::{SweatFt, FT_CONTRACT};
use sweat_model::{StorageManagementIntegration, SweatApiIntegration};

use crate::interface::{
    claim_contract::{SweatClaim, SWEAT_CLAIM},
    common::ContractAccount,
};

pub const CLAIM_PERIOD: Duration = 30 * 60;
pub const BURN_PERIOD: Duration = 3 * 60 * 60;

pub type Context = integration_utils::context::Context<near_workspaces::network::Sandbox>;

#[async_trait]
pub trait IntegrationContext {
    async fn manager(&mut self) -> anyhow::Result<Account>;
    async fn alice(&mut self) -> anyhow::Result<Account>;
    fn sweat_claim(&self) -> SweatClaim;
    fn ft_contract(&self) -> SweatFt;
}

#[async_trait]
impl IntegrationContext for Context {
    async fn manager(&mut self) -> anyhow::Result<Account> {
        self.account("manager").await
    }

    async fn alice(&mut self) -> anyhow::Result<Account> {
        self.account("alice").await
    }

    fn sweat_claim(&self) -> SweatClaim {
        SweatClaim::with_contract(&self.contracts[SWEAT_CLAIM])
    }

    fn ft_contract(&self) -> SweatFt {
        SweatFt::with_contract(&self.contracts[FT_CONTRACT])
    }
}

pub async fn prepare_contract() -> anyhow::Result<Context> {
    let mut context = Context::new(&[FT_CONTRACT, SWEAT_CLAIM], "build".into()).await?;
    let manager = context.manager().await?;
    let alice = context.alice().await?;

    context.ft_contract().new(".u.sweat.testnet".to_string().into()).await?;
    context.sweat_claim().init(context.ft_contract().account()).await?;

    context.ft_contract().add_oracle(&manager.to_near()).await?;

    context
        .sweat_claim()
        .add_oracle(context.ft_contract().account())
        .await?;
    context.sweat_claim().add_oracle(manager.to_near()).await?;

    context
        .ft_contract()
        .storage_deposit(context.sweat_claim().contract().as_account().to_near().into(), None)
        .await?;

    context
        .ft_contract()
        .storage_deposit(alice.to_near().into(), None)
        .await?;
    context
        .ft_contract()
        .tge_mint(&alice.to_near(), U128(100_000_000))
        .await?;

    context
        .sweat_claim()
        .with_user(&manager)
        .set_claim_period(CLAIM_PERIOD)
        .await?;
    context
        .sweat_claim()
        .with_user(&manager)
        .set_burn_period(BURN_PERIOD)
        .await?;

    Ok(context)
}
