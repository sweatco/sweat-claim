use anyhow::Result;
use async_trait::async_trait;
use integration_utils::integration_contract::IntegrationContract;
use near_sdk::{
    json_types::{U128, U64},
    AccountId,
};
use near_units::parse_near;
use serde_json::json;
use sweat_model::{
    FungibleTokenCoreIntegration, StorageManagementIntegration, SweatApiIntegration, SweatDeferIntegration,
};
use workspaces::{Account, Contract};

pub const FT_CONTRACT: &str = "sweat";

pub struct SweatFt<'a> {
    contract: &'a Contract,
    account: Option<Account>,
}

#[async_trait]
impl FungibleTokenCoreIntegration for SweatFt<'_> {
    async fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) -> Result<()> {
        todo!()
    }

    async fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> Result<U128> {
        println!(
            "▶️ Transfer {:?} fungible tokens to {} with message: {}",
            amount, receiver_id, msg
        );

        let args = json!({
            "receiver_id": receiver_id,
            "amount": amount,
            "msg": msg.to_string(),
        });

        let result = self
            .user_account()
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

        Ok(result.json()?)
    }

    async fn ft_total_supply(&self) -> Result<U128> {
        self.call_contract("ft_total_supply", ()).await
    }

    async fn ft_balance_of(&self, account_id: AccountId) -> Result<U128> {
        self.call_contract(
            "ft_balance_of",
            json!({
                "account_id": account_id,
            }),
        )
        .await
    }
}

#[async_trait]
impl StorageManagementIntegration for SweatFt<'_> {
    async fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> Result<near_contract_standards::storage_management::StorageBalance> {
        let args = json!({ "account_id": account_id });

        let result = self
            .contract
            .call("storage_deposit")
            .args_json(args)
            .deposit(parse_near!("0.00235 N"))
            .transact()
            .await?
            .into_result()?;

        Ok(result.json()?)
    }

    async fn storage_withdraw(
        &mut self,
        amount: Option<U128>,
    ) -> Result<near_contract_standards::storage_management::StorageBalance> {
        todo!()
    }

    async fn storage_unregister(&mut self, force: Option<bool>) -> Result<bool> {
        todo!()
    }

    async fn storage_balance_bounds(
        &self,
    ) -> Result<near_contract_standards::storage_management::StorageBalanceBounds> {
        todo!()
    }

    async fn storage_balance_of(
        &self,
        account_id: AccountId,
    ) -> Result<Option<near_contract_standards::storage_management::StorageBalance>> {
        todo!()
    }
}

#[async_trait]
impl SweatDeferIntegration for SweatFt<'_> {
    async fn defer_batch(&mut self, steps_batch: Vec<(AccountId, u16)>, holding_account_id: AccountId) -> Result<()> {
        self.call_user(
            "defer_batch",
            json!({
                "steps_batch": steps_batch,
                "holding_account_id": holding_account_id,
            }),
        )
        .await
    }
}

#[async_trait]
impl SweatApiIntegration for SweatFt<'_> {
    async fn new(&self, postfix: Option<String>) -> Result<()> {
        self.call_contract(
            "new",
            json!({
                "postfix": postfix,
            }),
        )
        .await
    }

    async fn add_oracle(&mut self, account_id: &AccountId) -> Result<()> {
        self.call_contract(
            "add_oracle",
            json!({
                "account_id": account_id,
            }),
        )
        .await
    }

    async fn remove_oracle(&mut self, account_id: &AccountId) -> anyhow::Result<()> {
        todo!()
    }

    async fn get_oracles(&self) -> anyhow::Result<Vec<AccountId>> {
        todo!()
    }

    async fn tge_mint(&mut self, account_id: &AccountId, amount: U128) -> anyhow::Result<()> {
        self.call_contract(
            "tge_mint",
            json!({
                "account_id": account_id,
                "amount": amount,
            }),
        )
        .await
    }

    async fn tge_mint_batch(&mut self, batch: Vec<(AccountId, U128)>) -> anyhow::Result<()> {
        todo!()
    }

    async fn burn(&mut self, amount: &U128) -> anyhow::Result<()> {
        todo!()
    }

    async fn get_steps_since_tge(&self) -> anyhow::Result<U64> {
        todo!()
    }

    async fn record_batch(&mut self, steps_batch: Vec<(AccountId, u16)>) -> anyhow::Result<()> {
        todo!()
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
}

impl SweatFt<'_> {
    pub async fn formula_detailed(&self, steps_since_tge: U64, steps: u16) -> anyhow::Result<(U128, U128, U128)> {
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
