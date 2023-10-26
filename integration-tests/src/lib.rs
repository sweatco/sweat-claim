#![cfg(test)]

use integration_utils::{integration_contract::IntegrationContract, misc::ToNear};
use model::{BurnApiIntegration, ClaimApiIntegration, ClaimAvailabilityView};
use near_sdk::json_types::U64;

use crate::{
    interface::{common::ContractAccount, ft_contract::FtContractInterface},
    prepare::{prepare_contract, IntegrationContext, BURN_PERIOD, CLAIM_PERIOD},
};

mod interface;
mod prepare;

#[tokio::test]
async fn happy_flow() -> anyhow::Result<()> {
    let mut context = prepare_contract().await?;

    let alice = context.alice().await?;
    let manager = context.manager().await?;

    let alice_steps = 10_000;
    let alice_initial_balance = context.ft_contract().ft_balance_of(&alice.to_near()).await?;

    let target_token_amount = context.ft_contract().formula(U64(0), alice_steps).await?.0;
    let target_fee = target_token_amount * 5 / 100;
    let target_effective_token_amount = target_token_amount - target_fee;

    context
        .ft_contract()
        .with_user(&manager)
        .defer_batch(vec![(alice.to_near(), alice_steps)], context.sweat_claim().account())
        .await?;

    let claim_contract_balance = context
        .ft_contract()
        .ft_balance_of(&context.sweat_claim().account())
        .await?;

    assert_eq!(claim_contract_balance.0, target_effective_token_amount);

    let alice_deferred_balance = context
        .sweat_claim()
        .get_claimable_balance_for_account(alice.to_near())
        .await?;
    assert_eq!(alice_deferred_balance.0, target_effective_token_amount);

    let is_claim_available = context.sweat_claim().is_claim_available(alice.to_near()).await?;
    assert!(matches!(is_claim_available, ClaimAvailabilityView::Unavailable(_)));

    context
        .fast_forward_hours((CLAIM_PERIOD / (60 * 60) + 1) as u64)
        .await?;

    let is_claim_available = context.sweat_claim().is_claim_available(alice.to_near()).await?;
    assert_eq!(is_claim_available, ClaimAvailabilityView::Available);

    context.sweat_claim().with_user(&alice).claim().await?;

    let alice_balance = context.ft_contract().ft_balance_of(&alice.to_near()).await?;
    let alice_balance_change = alice_balance.0 - alice_initial_balance.0;
    assert_eq!(alice_balance_change, target_effective_token_amount);

    Ok(())
}

#[tokio::test]
async fn burn() -> anyhow::Result<()> {
    let mut context = prepare_contract().await?;

    let manager = context.manager().await?;
    let alice = context.alice().await?;

    let alice_steps = 10_000;

    let target_token_amount = context.ft_contract().formula(U64(0), alice_steps).await?.0;
    let target_fee = target_token_amount * 5 / 100;
    let target_effective_token_amount = target_token_amount - target_fee;

    context
        .ft_contract()
        .with_user(&manager)
        .defer_batch(vec![(alice.to_near(), alice_steps)], context.sweat_claim().account())
        .await?;

    let claim_contract_balance = context
        .ft_contract()
        .ft_balance_of(&context.sweat_claim().account())
        .await?;

    assert_eq!(claim_contract_balance.0, target_effective_token_amount);

    let burn_result = context.sweat_claim().with_user(&manager).burn().await?;
    assert_eq!(0, burn_result.0);

    context.fast_forward_hours((BURN_PERIOD / (60 * 60) + 1) as u64).await?;

    let burn_result = context.sweat_claim().with_user(&manager).burn().await?;
    assert_eq!(target_effective_token_amount, burn_result.0);

    let alice_deferred_balance = context
        .sweat_claim()
        .get_claimable_balance_for_account(alice.to_near())
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
        .with_user(&manager)
        .defer_batch(vec![(alice.to_near(), alice_steps)], context.sweat_claim().account())
        .await?;
    steps_since_tge += alice_steps as u64;

    let alice_deferred_balance = context
        .sweat_claim()
        .get_claimable_balance_for_account(alice.to_near())
        .await?;
    assert_eq!(target_effective_token_amount, alice_deferred_balance);

    context.fast_forward_hours((BURN_PERIOD / (60 * 60) + 1) as u64).await?;

    let alice_deferred_balance = context
        .sweat_claim()
        .get_claimable_balance_for_account(alice.to_near())
        .await?;
    assert_eq!(0, alice_deferred_balance.0);

    let (_, target_outdated_effective_token_amount, _) = context
        .ft_contract()
        .formula_detailed(U64(steps_since_tge), alice_steps)
        .await?;

    context
        .ft_contract()
        .with_user(&manager)
        .defer_batch(vec![(alice.to_near(), alice_steps)], context.sweat_claim().account())
        .await?;
    steps_since_tge += alice_steps as u64;

    context.fast_forward_hours(2).await?;

    let (_, target_effective_token_amount, _) = context
        .ft_contract()
        .formula_detailed(U64(steps_since_tge), alice_steps)
        .await?;

    context
        .ft_contract()
        .with_user(&manager)
        .defer_batch(vec![(alice.to_near(), alice_steps)], context.sweat_claim().account())
        .await?;

    let alice_deferred_balance = context
        .sweat_claim()
        .get_claimable_balance_for_account(alice.to_near())
        .await?;
    assert_eq!(
        target_effective_token_amount.0 + target_outdated_effective_token_amount.0,
        alice_deferred_balance.0
    );

    context.fast_forward_hours((BURN_PERIOD / (60 * 60) - 1) as u64).await?;

    let alice_deferred_balance = context
        .sweat_claim()
        .get_claimable_balance_for_account(alice.to_near())
        .await?;
    assert_eq!(target_effective_token_amount, alice_deferred_balance);

    Ok(())
}
