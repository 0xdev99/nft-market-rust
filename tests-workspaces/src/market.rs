use std::collections::HashMap;

use near_units::parse_near;

use crate::utils::{
    check_outcome_fail, check_outcome_success, create_series_raw, init_market, init_nft,
    mint_token, nft_approve,
};

#[tokio::test]
async fn storage_deposit() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();

    // Negative
    let outcome = user
        .call(&worker, market.id().clone(), "storage_deposit")
        .deposit(20)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Requires minimum deposit of").await;

    // Positive
    let outcome = user
        .call(&worker, market.id().clone(), "storage_deposit")
        .deposit(parse_near!("0.01 N"))
        .transact()
        .await?;
    check_outcome_success(outcome.status).await;
    Ok(())
}

#[tokio::test]
async fn storage_withdraw() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let outcome = user
        .call(&worker, market.id().clone(), "storage_deposit")
        .deposit(parse_near!("5 N"))
        .transact()
        .await?;
    check_outcome_success(outcome.status).await;
    let series = create_series_raw(
        &worker,
        nft.id().clone(),
        &user,
        Some(4),
        HashMap::from([(user.id(), 500)]),
    )
    .await?;
    let token = mint_token(&worker, nft.id().clone(), &user, user.id(), &series).await?;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 42000.into())]);
    nft_approve(
        &worker,
        nft.id().clone(),
        market.id().clone(),
        &user,
        token.clone(),
        sale_conditions.clone(),
        series.clone(),
    )
    .await;

    // Negative
    // - requires 1 yocto
    let outcome = user
        .call(&worker, market.id().clone(), "storage_withdraw")
        .transact()
        .await?;
    check_outcome_fail(
        outcome.status,
        "Requires attached deposit of exactly 1 yoctoNEAR",
    )
    .await;

    // Positive
    // - deposit refunded
    let outcome = user
        .call(&worker, market.id().clone(), "storage_withdraw")
        .deposit(1)
        .transact()
        .await?;
    check_outcome_success(outcome.status).await;

    // TODO: check balances
    Ok(())
}
