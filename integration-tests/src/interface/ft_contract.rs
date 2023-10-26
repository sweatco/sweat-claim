use async_trait::async_trait;
use integration_utils::integration_contract::IntegrationContract;
use near_sdk::{
    json_types::{U128, U64},
    AccountId,
};
use near_units::parse_near;
use serde_json::json;
use workspaces::{Account, Contract};

pub const FT_CONTRACT: &str = "sweat";

pub struct SweatFt<'a> {
    contract: &'a Contract,
    account: Option<Account>,
}

#[async_trait]
pub(crate) trait FtContractInterface {
    async fn init(&self) -> anyhow::Result<()>;

    async fn add_oracle(&self, account_id: &AccountId) -> anyhow::Result<()>;

    async fn ft_balance_of(&self, user: &AccountId) -> anyhow::Result<U128>;

    async fn mint_for_user(&self, user: &AccountId, amount: u128) -> anyhow::Result<()>;

    async fn storage_deposit(&self) -> anyhow::Result<()>;

    async fn ft_transfer_call(
        &self,
        account: &Account,
        receiver_id: &AccountId,
        amount: u128,
        msg: String,
    ) -> anyhow::Result<()>;

    async fn defer_batch(
        &self,
        steps_batch: Vec<(AccountId, u16)>,
        holding_account_id: AccountId,
    ) -> anyhow::Result<()>;

    async fn formula(&self, steps_since_tge: U64, steps: u16) -> anyhow::Result<U128>;

    async fn formula_detailed(&self, steps_since_tge: U64, steps: u16) -> anyhow::Result<(U128, U128, U128)>;
}

#[async_trait]
impl FtContractInterface for SweatFt<'_> {
    async fn init(&self) -> anyhow::Result<()> {
        self.call_contract(
            "new",
            json!({
                "postfix": ".u.sweat.testnet",
            }),
        )
        .await
    }

    async fn add_oracle(&self, account_id: &AccountId) -> anyhow::Result<()> {
        self.call_contract(
            "add_oracle",
            json!({
                "account_id": account_id,
            }),
        )
        .await
    }

    async fn ft_balance_of(&self, account_id: &AccountId) -> anyhow::Result<U128> {
        self.call_contract(
            "ft_balance_of",
            json!({
                "account_id": account_id,
            }),
        )
        .await
    }

    async fn mint_for_user(&self, user: &AccountId, amount: u128) -> anyhow::Result<()> {
        self.call_contract(
            "tge_mint",
            json!({
                "account_id": user,
                "amount": amount.to_string(),
            }),
        )
        .await
    }

    async fn storage_deposit(&self) -> anyhow::Result<()> {
        let user = self.user_account();

        let args = json!({ "account_id": user.id() });

        user.call(self.contract.id(), "storage_deposit")
            .args_json(args)
            .deposit(parse_near!("0.00235 N"))
            .transact()
            .await?
            .into_result()?;

        Ok(())
    }

    async fn ft_transfer_call(
        &self,
        user: &Account,
        receiver_id: &AccountId,
        amount: u128,
        msg: String,
    ) -> anyhow::Result<()> {
        println!(
            "▶️ Transfer {} fungible tokens to {} with message: {}",
            amount, receiver_id, msg
        );

        let args = json!({
            "receiver_id": receiver_id,
            "amount": amount.to_string(),
            "msg": msg.to_string(),
        });

        let result = user
            .call(self.contract.id(), "ft_transfer_call")
            .args_json(args)
            .max_gas()
            .deposit(parse_near!("1 yocto"))
            .transact()
            .await?
            .into_result()?;

        for log in result.logs() {
            println!("   📖 {:?}", log);
        }

        Ok(())
    }

    async fn defer_batch(
        &self,
        steps_batch: Vec<(AccountId, u16)>,
        holding_account_id: AccountId,
    ) -> anyhow::Result<()> {
        self.call_user(
            "defer_batch",
            json!({
                "steps_batch": steps_batch,
                "holding_account_id": holding_account_id,
            }),
        )
        .await
    }

    async fn formula(&self, steps_since_tge: U64, steps: u16) -> anyhow::Result<U128> {
        self.call_contract(
            "formula",
            json!({
                "steps_since_tge": steps_since_tge,
                "steps": steps,
            }),
        )
        .await
    }

    async fn formula_detailed(&self, steps_since_tge: U64, steps: u16) -> anyhow::Result<(U128, U128, U128)> {
        let token_amount = self.formula(steps_since_tge, steps).await?.0;
        let fee = token_amount * 5 / 100;
        let effective_amount = token_amount - fee;

        Ok((U128(fee), U128(effective_amount), U128(token_amount)))
    }
}

impl<'a> IntegrationContract<'a> for SweatFt<'a> {
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
