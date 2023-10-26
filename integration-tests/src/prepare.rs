use async_trait::async_trait;
use integration_utils::{integration_contract::IntegrationContract, misc::ToNear};
use model::{AuthApiIntegration, Duration, InitApiIntegration};
use workspaces::Account;

use crate::interface::{
    claim_contract::{SweatClaim, SWEAT_CLAIM},
    common::ContractAccount,
    ft_contract::{FtContractInterface, SweatFt, FT_CONTRACT},
};

pub const CLAIM_PERIOD: Duration = 30 * 60;
pub const BURN_PERIOD: Duration = 3 * 60 * 60;

pub type Context = integration_utils::context::Context<workspaces::network::Sandbox>;

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
    let mut context = Context::new(&[FT_CONTRACT, SWEAT_CLAIM]).await?;
    let manager = context.manager().await?;
    let alice = context.alice().await?;

    context.ft_contract().init().await?;
    context.sweat_claim().init(context.ft_contract().account()).await?;

    context.ft_contract().add_oracle(&manager.to_near()).await?;

    context
        .sweat_claim()
        .add_oracle(context.ft_contract().account())
        .await?;
    context.sweat_claim().add_oracle(manager.to_near()).await?;

    context
        .ft_contract()
        .with_user(context.sweat_claim().contract().as_account())
        .storage_deposit()
        .await?;

    context.ft_contract().with_user(&alice).storage_deposit().await?;
    context
        .ft_contract()
        .mint_for_user(&alice.to_near(), 100_000_000)
        .await?;

    context.sweat_claim().set_claim_period(CLAIM_PERIOD).await?;
    context.sweat_claim().set_burn_period(BURN_PERIOD).await?;

    Ok(context)
}
