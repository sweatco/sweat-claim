#![cfg(test)]

use integration_utils::{integration_contract::IntegrationContract, misc::ToNear};
use model::{
    api::{BurnApiIntegration, ClaimApiIntegration},
    ClaimAvailabilityView,
};
use near_sdk::{json_types::U64, serde_json::json};
use sweat_model::{FungibleTokenCoreIntegration, Payout, SweatApiIntegration, SweatDeferIntegration};

use crate::{
    common::{calculate_fee, PanicFinder},
    interface::common::ContractAccount,
    prepare::{prepare_contract, IntegrationContext, BURN_PERIOD, CLAIM_PERIOD},
};

mod common;
mod interface;
mod measure;
mod prepare;

#[tokio::test]
async fn happy_flow() -> anyhow::Result<()> {
    let mut context = prepare_contract().await?;

    let alice = context.alice().await?;
    let manager = context.manager().await?;

    let alice_steps = 10_000;
    let alice_initial_balance = context.ft_contract().ft_balance_of(alice.to_near()).call().await?;

    let target_token_amount = context.ft_contract().formula(U64(0), alice_steps).call().await?.0;
    let target_payout = Payout::from(target_token_amount);

    context
        .ft_contract()
        .defer_batch(vec![(alice.to_near(), alice_steps)], context.sweat_claim().account())
        .with_user(&manager)
        .call()
        .await?;

    let claim_contract_balance = context
        .ft_contract()
        .ft_balance_of(context.sweat_claim().account())
        .call()
        .await?;

    assert_eq!(claim_contract_balance.0, target_payout.amount_for_user);

    let alice_deferred_balance = context
        .sweat_claim()
        .get_claimable_balance_for_account(alice.to_near(), None)
        .call()
        .await?;
    assert_eq!(alice_deferred_balance.0, target_payout.amount_for_user);

    let is_claim_available = context.sweat_claim().is_claim_available(alice.to_near()).call().await?;
    assert!(matches!(is_claim_available, ClaimAvailabilityView::Unavailable(_)));

    context
        .fast_forward_hours((CLAIM_PERIOD / (60 * 60) + 1) as u64)
        .await?;

    let is_claim_available = context.sweat_claim().is_claim_available(alice.to_near()).call().await?;
    assert_eq!(is_claim_available, ClaimAvailabilityView::Available);

    context.sweat_claim().claim(None).with_user(&alice).call().await?;

    let alice_balance = context.ft_contract().ft_balance_of(alice.to_near()).call().await?;
    let alice_balance_change = alice_balance.0 - alice_initial_balance.0;
    assert_eq!(alice_balance_change, target_payout.amount_for_user);

    Ok(())
}

#[tokio::test]
async fn burn() -> anyhow::Result<()> {
    let mut context = prepare_contract().await?;

    let manager = context.manager().await?;
    let alice = context.alice().await?;

    let alice_steps = 10_000;

    let target_token_amount = context.ft_contract().formula(U64(0), alice_steps).call().await?.0;
    let target_payout = Payout::from(target_token_amount);

    context
        .ft_contract()
        .defer_batch(vec![(alice.to_near(), alice_steps)], context.sweat_claim().account())
        .with_user(&manager)
        .call()
        .await?;

    let claim_contract_balance = context
        .ft_contract()
        .ft_balance_of(context.sweat_claim().account())
        .call()
        .await?;

    assert_eq!(claim_contract_balance.0, target_payout.amount_for_user);

    let burn_result = context.sweat_claim().burn().with_user(&manager).call().await?;
    assert_eq!(0, burn_result.0);

    context.fast_forward_hours((BURN_PERIOD / (60 * 60) + 1) as u64).await?;

    let burn_result = context.sweat_claim().burn().with_user(&manager).call().await?;
    assert_eq!(target_payout.amount_for_user, burn_result.0);

    let alice_deferred_balance = context
        .sweat_claim()
        .get_claimable_balance_for_account(alice.to_near(), None)
        .call()
        .await?;
    assert_eq!(0, alice_deferred_balance.0);

    Ok(())
}

#[tokio::test]
async fn outdate() -> anyhow::Result<()> {
    let mut context = prepare_contract().await?;

    let manager = context.manager().await?;
    let alice = context.alice().await?;

    let mut steps_since_tge = 0;
    let alice_steps = 10_000;

    let (_, target_effective_token_amount, _) = context
        .ft_contract()
        .formula_detailed(U64(steps_since_tge), alice_steps)
        .await?;

    context
        .ft_contract()
        .defer_batch(vec![(alice.to_near(), alice_steps)], context.sweat_claim().account())
        .with_user(&manager)
        .call()
        .await?;
    steps_since_tge += alice_steps as u64;

    let alice_deferred_balance = context
        .sweat_claim()
        .get_claimable_balance_for_account(alice.to_near(), None)
        .call()
        .await?;
    assert_eq!(target_effective_token_amount, alice_deferred_balance);

    context.fast_forward_hours((BURN_PERIOD / (60 * 60) + 1) as u64).await?;

    let alice_deferred_balance = context
        .sweat_claim()
        .get_claimable_balance_for_account(alice.to_near(), None)
        .call()
        .await?;
    assert_eq!(0, alice_deferred_balance.0);

    let (_, target_outdated_effective_token_amount, _) = context
        .ft_contract()
        .formula_detailed(U64(steps_since_tge), alice_steps)
        .await?;

    context
        .ft_contract()
        .defer_batch(vec![(alice.to_near(), alice_steps)], context.sweat_claim().account())
        .with_user(&manager)
        .call()
        .await?;
    steps_since_tge += alice_steps as u64;

    context.fast_forward_hours(2).await?;

    let (_, target_effective_token_amount, _) = context
        .ft_contract()
        .formula_detailed(U64(steps_since_tge), alice_steps)
        .await?;

    context
        .ft_contract()
        .defer_batch(vec![(alice.to_near(), alice_steps)], context.sweat_claim().account())
        .with_user(&manager)
        .call()
        .await?;

    let alice_deferred_balance = context
        .sweat_claim()
        .get_claimable_balance_for_account(alice.to_near(), None)
        .call()
        .await?;
    assert_eq!(
        target_effective_token_amount.0 + target_outdated_effective_token_amount.0,
        alice_deferred_balance.0
    );

    context.fast_forward_hours((BURN_PERIOD / (60 * 60) - 1) as u64).await?;

    let alice_deferred_balance = context
        .sweat_claim()
        .get_claimable_balance_for_account(alice.to_near(), None)
        .call()
        .await?;
    assert_eq!(target_effective_token_amount, alice_deferred_balance);

    Ok(())
}

#[tokio::test]
async fn on_burn_direct_call() -> anyhow::Result<()> {
    let mut context = prepare_contract().await?;

    let alice = context.alice().await?;

    let result = alice
        .call(context.sweat_claim().contract().as_account().id(), "on_burn")
        .args_json(json!({
            "total_to_burn": "100000",
            "keys_to_remove": vec![1702303000, 1702304333],
        }))
        .max_gas()
        .transact()
        .await?
        .into_result();

    assert!(result.has_panic("Method on_burn is private"));

    Ok(())
}

#[tokio::test]
async fn on_transfer_direct_call() -> anyhow::Result<()> {
    let mut context = prepare_contract().await?;

    let alice = context.alice().await?;

    let result = alice
        .call(context.sweat_claim().contract().as_account().id(), "on_transfer")
        .args_json(json!({
            "now": 1702304333,
            "account_id": alice.id().to_string(),
            "total_accrual": "100000",
            "details": vec![(1702303000, "100000")],
        }))
        .max_gas()
        .transact()
        .await?
        .into_result();

    assert!(result.has_panic("Method on_transfer is private"));

    Ok(())
}
