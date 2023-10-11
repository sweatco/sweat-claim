use async_trait::async_trait;
use near_sdk::json_types::{U128, U64};
use near_units::parse_near;
use serde_json::json;
use workspaces::{Account, AccountId, Contract};

use crate::interface::common::ContractAccount;

pub(crate) trait FtContract: FtContractInterface + ContractAccount {}

impl FtContract for Contract {}

#[async_trait]
pub(crate) trait FtContractInterface {
    async fn init(&self) -> anyhow::Result<()>;

    async fn add_oracle(&self, account_id: &AccountId) -> anyhow::Result<()>;

    async fn ft_balance_of(&self, user: &Account) -> anyhow::Result<U128>;

    async fn mint_for_user(&self, user: &Account, amount: u128) -> anyhow::Result<()>;

    async fn storage_deposit(&self, user: &Account) -> anyhow::Result<()>;

    async fn ft_transfer_call(
        &self,
        account: &Account,
        receiver_id: &AccountId,
        amount: u128,
        msg: String,
    ) -> anyhow::Result<()>;

    async fn defer_batch(
        &self,
        manager: &Account,
        steps_batch: Vec<(AccountId, u16)>,
        holding_account_id: AccountId,
    ) -> anyhow::Result<()>;

    async fn formula(&self, steps_since_tge: U64, steps: u16) -> anyhow::Result<U128>;
    
    async fn formula_detailed(&self, steps_since_tge: U64, steps: u16) -> anyhow::Result<(U128, U128, U128)>;
}

#[async_trait]
impl FtContractInterface for Contract {
    async fn init(&self) -> anyhow::Result<()> {
        println!("▶️ Init ft contract");

        let result = self
            .call("new")
            .args_json(json!({
                "postfix": ".u.sweat.testnet",
            }))
            .max_gas()
            .transact()
            .await?
            .into_result();

        println!("Result: {:?}", result);

        Ok(())
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

    async fn ft_balance_of(&self, user: &Account) -> anyhow::Result<U128> {
        println!("▶️ View ft balance of user {:?}", user.id());

        let args = json!({
            "account_id": user.id(),
        });

        let result = self.view("ft_balance_of").args_json(args).await?.json()?;

        println!("   ✅ {:?}", result);

        Ok(result)
    }

    async fn mint_for_user(&self, user: &Account, amount: u128) -> anyhow::Result<()> {
        println!("▶️ Mint {:?} tokens for user {:?}", amount, user.id());

        let args = json!({
            "account_id": user.id(),
            "amount": amount.to_string(),
        });

        self.account()
            .call(self.id(), "tge_mint")
            .args_json(args)
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        Ok(())
    }

    async fn storage_deposit(&self, user: &Account) -> anyhow::Result<()> {
        println!("▶️ Register {} in ft contract (storage_deposit)", user.id());

        let args = json!({
            "account_id": user.id()
        });

        user.call(self.id(), "storage_deposit")
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
            .call(self.id(), "ft_transfer_call")
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
        manager: &Account,
        steps_batch: Vec<(AccountId, u16)>,
        holding_account_id: AccountId,
    ) -> anyhow::Result<()> {
        println!("▶️ Mint (for {holding_account_id}) and defer: {:?}", steps_batch);

        let args = json!({
            "steps_batch": steps_batch,
            "holding_account_id": holding_account_id,
        });

        let result = manager
            .call(self.id(), "defer_batch")
            .args_json(args)
            .max_gas()
            .transact()
            .await?
            .into_result()?;

        for log in result.logs() {
            println!("   📖 {:?}", log);
        }

        Ok(())
    }

    async fn formula(&self, steps_since_tge: U64, steps: u16) -> anyhow::Result<U128> {
        println!("▶️ Formula for {steps_since_tge:?} steps since TGE and {steps} steps provided");

        let args = json!({
            "steps_since_tge": steps_since_tge,
            "steps": steps,
        });

        let result = self.view("formula").args_json(args).await?.json()?;

        println!("   ✅ {:?}", result);

        Ok(result)
    }

    async fn formula_detailed(&self, steps_since_tge: U64, steps: u16) -> anyhow::Result<(U128, U128, U128)> {
        let token_amount = self.formula(steps_since_tge, steps).await?.0;
        let fee = token_amount * 5 / 100;
        let effective_amount = token_amount - fee;

        Ok((U128(fee), U128(effective_amount), U128(token_amount)))
    }
}
