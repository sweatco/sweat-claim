use async_trait::async_trait;
use claim_model::{
    api::{AuthApiIntegration, ClaimContract, ConfigApiIntegration, InitApiIntegration},
    Duration,
};
use integration_utils::misc::ToNear;
use near_sdk::json_types::U128;
use near_workspaces::{Account, AccountId};
use sweat_model::{StorageManagementIntegration, SweatApiIntegration, SweatContract};

const SWEAT_FT_CONTRACT: &str = "sweat";
const SWEAT_CLAIM: &str = "sweat_claim";
const USDT_CONTRACT: &str = "example_ft";

pub const CLAIM_PERIOD: Duration = 30 * 60;
pub const BURN_PERIOD: Duration = 3 * 60 * 60;

pub type Context = integration_utils::context::Context<near_workspaces::network::Sandbox>;

#[async_trait]
pub trait IntegrationContext {
    async fn manager(&mut self) -> anyhow::Result<Account>;
    async fn alice(&mut self) -> anyhow::Result<Account>;
    fn sweat_claim(&self) -> ClaimContract;
    fn sweat_ft(&self) -> SweatContract;
    fn usdt_ft(&self) -> SweatContract;
}

#[async_trait]
impl IntegrationContext for Context {
    async fn manager(&mut self) -> anyhow::Result<Account> {
        self.account("manager").await
    }

    async fn alice(&mut self) -> anyhow::Result<Account> {
        self.account("alice").await
    }

    fn sweat_claim(&self) -> ClaimContract {
        ClaimContract {
            contract: &self.contracts[SWEAT_CLAIM],
        }
    }

    fn sweat_ft(&self) -> SweatContract {
        SweatContract {
            contract: &self.contracts[SWEAT_FT_CONTRACT],
        }
    }

    fn usdt_ft(&self) -> SweatContract {
        SweatContract {
            contract: &self.contracts[USDT_CONTRACT],
        }
    }
}

pub async fn prepare_contract() -> anyhow::Result<Context> {
    let mut context = Context::new(
        &[SWEAT_FT_CONTRACT, USDT_CONTRACT, SWEAT_CLAIM],
        true,
        "build-integration".into(),
    )
    .await?;
    let manager = context.manager().await?;
    let alice = context.alice().await?;
    let claim_contract_account = context.sweat_claim().contract.as_account();

    context.sweat_ft().prepare_ft_contract(".u.sweat.testnet", &manager, &alice, claim_contract_account).await?;
    context.usdt_ft().prepare_ft_contract(".u.usdt.testnet", &manager, &alice, claim_contract_account).await?;

    context
        .sweat_claim()
        .init(context.sweat_ft().contract.as_account().to_near())
        .await?;

    context
        .sweat_claim()
        .add_oracle(context.sweat_ft().contract.as_account().to_near())
        .await?;
    context.sweat_claim().add_oracle(manager.to_near()).await?;

    context
        .sweat_claim()
        .set_claim_period(CLAIM_PERIOD)
        .with_user(&manager)
        .await?;
    context
        .sweat_claim()
        .set_burn_period(BURN_PERIOD)
        .with_user(&manager)
        .await?;

    Ok(context)
}

trait SweatContractInit {
    async fn prepare_ft_contract(&self, postfix: &str, manager: &Account, alice: &Account, claim_contract_account: &Account) -> anyhow::Result<()>;
}

impl<'a> SweatContractInit for SweatContract<'a> {
    async fn prepare_ft_contract(&self, postfix: &str, manager: &Account, alice: &Account, claim_contract_account: &Account) -> anyhow::Result<()> {
        self.new(postfix.to_string().into()).await?;
        
        self.add_oracle(&manager.to_near()).await?;
    
        self
            .storage_deposit(claim_contract_account.to_near().into(), None)
            .await?;
    
        self.storage_deposit(alice.to_near().into(), None).await?;
        self.tge_mint(&alice.to_near(), U128(100_000_000)).await?;
    
        Ok(())
    }
}
