use async_trait::async_trait;
use integration_utils::{contract_call::ContractCall, integration_contract::IntegrationContract};
use model::{
    api::{
        AuthApiIntegration, BurnApiIntegration, ClaimApiIntegration, ConfigApiIntegration, InitApiIntegration,
        RecordApiIntegration,
    },
    ClaimAvailabilityView, ClaimResultView, Duration, TokenSymbol,
};
use near_sdk::{json_types::U128, serde_json::json, AccountId};
use near_workspaces::Contract;

pub const SWEAT_CLAIM: &str = "sweat_claim";

pub struct SweatClaim<'a> {
    contract: &'a Contract,
}

#[async_trait]
impl InitApiIntegration for SweatClaim<'_> {
    fn init(&self, default_token: (TokenSymbol, AccountId)) -> ContractCall<()> {
        self.make_call("init")
            .args_json(json!({
                "default_token": default_token,
            }))
            .unwrap()
    }
}

#[async_trait]
impl ConfigApiIntegration for SweatClaim<'_> {
    fn set_claim_period(&mut self, period: Duration) -> ContractCall<()> {
        self.make_call("set_claim_period")
            .args_json(json!({
                "period": period,
            }))
            .unwrap()
    }

    fn set_burn_period(&mut self, period: Duration) -> ContractCall<()> {
        self.make_call("set_burn_period")
            .args_json(json!({
                "period": period,
            }))
            .unwrap()
    }
}

#[async_trait]
impl AuthApiIntegration for SweatClaim<'_> {
    fn add_oracle(&mut self, account_id: AccountId) -> ContractCall<()> {
        self.make_call("add_oracle")
            .args_json(json!({
                "account_id": account_id,
            }))
            .unwrap()
    }

    fn remove_oracle(&mut self, account_id: AccountId) -> ContractCall<()> {
        self.make_call("remove_oracle")
            .args_json(json!({
                "account_id": account_id,
            }))
            .unwrap()
    }

    fn get_oracles(&self) -> ContractCall<Vec<AccountId>> {
        self.make_call("get_oracles")
    }
}

#[async_trait]
impl BurnApiIntegration for SweatClaim<'_> {
    fn burn(&mut self) -> ContractCall<U128> {
        self.make_call("burn")
    }
}

#[async_trait]
impl RecordApiIntegration for SweatClaim<'_> {
    fn record_batch_for_hold(
        &mut self,
        amounts: Vec<(AccountId, U128)>,
        token_symbol: Option<TokenSymbol>,
    ) -> ContractCall<()> {
        self.make_call("record_batch_for_hold")
            .args_json(json!({
                "amounts": amounts,
                "token_symbol": token_symbol,
            }))
            .unwrap()
    }
}

#[async_trait]
impl ClaimApiIntegration for SweatClaim<'_> {
    fn get_claimable_balance_for_account(
        &self,
        account_id: AccountId,
        token_symbol: Option<TokenSymbol>,
    ) -> ContractCall<U128> {
        self.make_call("get_claimable_balance_for_account")
            .args_json(json!({
                "account_id": account_id,
                "token_symbol": token_symbol,
            }))
            .unwrap()
    }

    fn is_claim_available(&self, account_id: AccountId) -> ContractCall<ClaimAvailabilityView> {
        self.make_call("is_claim_available")
            .args_json(json!({
                "account_id": account_id,
            }))
            .unwrap()
    }

    fn claim(&mut self, token_symbol: Option<TokenSymbol>) -> ContractCall<ClaimResultView> {
        self.make_call("claim")
            .args_json(json!({
                "token_symbol": token_symbol,
            }))
            .unwrap()
    }
}

impl<'a> IntegrationContract<'a> for SweatClaim<'a> {
    fn with_contract(contract: &'a Contract) -> Self {
        Self { contract }
    }

    fn contract(&self) -> &'a Contract {
        self.contract
    }
}
