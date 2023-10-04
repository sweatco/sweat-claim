#![cfg(test)]

use crate::context::{prepare_contract, Context, Prepared};

mod context;
mod interface;

#[tokio::test]
async fn test() -> anyhow::Result<()> {
    let Prepared {
        context,
        manager,
        alice,
    } = prepare_contract().await?;

    Ok(())
}
