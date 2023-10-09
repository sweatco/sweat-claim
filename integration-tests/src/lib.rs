#![cfg(test)]

use model::{ClaimAvailabilityView, Duration};
use near_sdk::json_types::U64;

use crate::context::{prepare_contract, Prepared};

mod context;
mod interface;

const CLAIM_PERIOD: Duration = 30 * 60;

const BURN_PERIOD: Duration = 3 * 60 * 60;

#[tokio::test]
async fn test() -> anyhow::Result<()> {
    let Prepared {
        context,
        manager,
        alice,
    } = prepare_contract().await?;

    context.claim_contract.set_claim_period(CLAIM_PERIOD).await?;
    context.claim_contract.set_burn_period(BURN_PERIOD).await?;

    let alice_steps = 10_000;
    let alice_initial_balance = context.ft_contract.ft_balance_of(&alice).await?;

    let target_token_amount = context.ft_contract.formula(U64(0), alice_steps).await?.0;
    let target_fee = target_token_amount * 5 / 100;
    let target_effective_token_amount = target_token_amount - target_fee;

    println!("@@ Target token amount: {target_token_amount}");
    println!("@@ Target effective token amount: {target_effective_token_amount}");

    context
        .ft_contract
        .defer_batch(
            &manager,
            vec![(alice.id().clone(), alice_steps)],
            context.claim_contract.account().id().clone(),
        )
        .await?;

    let claim_contract_balance = context
        .ft_contract
        .ft_balance_of(context.claim_contract.account())
        .await?;

    assert_eq!(claim_contract_balance.0, target_effective_token_amount);

    let alice_deferred_balance = context.claim_contract.get_balance_for_account(&alice).await?;
    assert_eq!(alice_deferred_balance.0, target_effective_token_amount);

    let is_claim_available = context.claim_contract.is_claim_available(alice.id()).await?;
    assert!(matches!(is_claim_available, ClaimAvailabilityView::Unavailable(_)));

    context.fast_forward_hours((CLAIM_PERIOD / 1000 + 1) as u64).await?;

    let is_claim_available = context.claim_contract.is_claim_available(alice.id()).await?;
    assert_eq!(is_claim_available, ClaimAvailabilityView::Available());

    context.claim_contract.claim(&alice).await?;

    let alice_balance = context.ft_contract.ft_balance_of(&alice).await?;
    let alice_balance_change = alice_balance.0 - alice_initial_balance.0;
    assert_eq!(alice_balance_change, target_effective_token_amount);

    Ok(())
}
