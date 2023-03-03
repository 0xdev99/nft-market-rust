use std::collections::HashMap;

use crate::utils::{
    check_outcome_fail, create_series_raw, init_nft, mint_token,
    nft_transfer_payout_helper,
};
use near_contract_standards::non_fungible_token::{metadata::TokenMetadata, Token};
use near_units::{parse_gas, parse_near};
use nft_bid_market::Fees;
use nft_contract::TokenSeriesJson;

/*
- Can only be called by the autorized account (if authorization enabled)
- Panics if the title of the series is not specified
- Panics if the total royalty payout exceeds 50%
*/
#[tokio::test]
async fn nft_create_series_negative() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let user2 = owner
        .create_subaccount(&worker, "user2")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();

    // Only authorized account can create series
    owner
        .call(&worker, nft.id().clone(), "set_private_minting")
        .args_json(serde_json::json!({
            "enabled": true,
        }))?
        .transact()
        .await?;
    let token_metadata = TokenMetadata {
        title: Some("some title".to_string()),
        description: None,
        media: Some("ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz".to_string()),
        media_hash: None,
        copies: Some(7),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    };
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
            "token_metadata": token_metadata,
            "royalty": null
        }))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err
            .to_string()
            .contains("Access to mint is denied for this contract"))
    } else {
        panic!("Expected failure")
    };
    owner
        .call(&worker, nft.id().clone(), "grant")
        .args_json(serde_json::json!({
            "account_id": user1.id()
        }))?
        .transact()
        .await?;

    // Title of the series should be specified
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
            "token_metadata": TokenMetadata{
                title: None,
                ..token_metadata.clone()},
            "royalty": null
        }))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err
            .to_string()
            .contains("title is missing from token metadata"))
    } else {
        panic!("Expected failure")
    };

    // Royalty can't exceed 50%
    let royalty = HashMap::from([(user1.id(), 500), (user2.id(), 5000)]);
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
            "token_metadata": token_metadata,
            "royalty": royalty,
        }))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err.to_string().contains("maximum royalty cap exceeded"))
    } else {
        panic!("Expected failure")
    };
    Ok(())
}

/*
- Creates a new series with given metadata and royalty
- Refunds a deposit
 */
#[tokio::test]
async fn nft_create_series_positive() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let user2 = owner
        .create_subaccount(&worker, "user2")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let royalty = HashMap::from([(user1.id(), 500), (user2.id(), 2000)]);
    let token_metadata = TokenMetadata {
        title: Some("some title".to_string()),
        description: None,
        media: Some("ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz".to_string()),
        media_hash: None,
        copies: Some(7),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    };
    let series1: String = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
            "token_metadata": token_metadata,
            "royalty": royalty,
        }))?
        .deposit(parse_near!("2 N"))
        .transact()
        .await?
        .json()?;

    owner
        .call(&worker, nft.id().clone(), "set_private_minting")
        .args_json(serde_json::json!({
            "enabled": true,
        }))?
        .transact()
        .await?;

    owner
        .call(&worker, nft.id().clone(), "grant")
        .args_json(serde_json::json!({
            "account_id": user2.id()
        }))?
        .transact()
        .await?;
    let series2: String = user2
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
            "token_metadata": token_metadata,
            "royalty": royalty,
        }))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?
        .json()?;
    let series1_json: TokenSeriesJson = nft
        .view(
            &worker,
            "nft_get_series",
            serde_json::json!({ "token_series_id": series1 })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(series1_json.owner_id.as_str(), user1.id().as_ref());
    let series2_json: TokenSeriesJson = nft
        .view(
            &worker,
            "nft_get_series",
            serde_json::json!({ "token_series_id": series2 })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(series2_json.owner_id.as_str(), user2.id().as_ref());

    assert_eq!(series1_json.metadata, series2_json.metadata);
    // TODO: check balance of user1 after workspaces updated
    Ok(())
}

/*
- Can only be called by the autorized account (if authorization enabled)
- Panics if there is no series `token_series_id`
- Panics if called not by the owner of the series or the approved account to mint this specific series
- Panics if the maximum number of tokens have already been minted
 */
#[tokio::test]
async fn nft_mint_negative() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let user2 = owner
        .create_subaccount(&worker, "user2")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let user3 = owner
        .create_subaccount(&worker, "user3")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let token_metadata = TokenMetadata {
        title: Some("some title".to_string()),
        description: None,
        media: Some("ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz".to_string()),
        media_hash: None,
        copies: Some(1),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    };
    let royalty = HashMap::from([(user1.id(), 500), (user2.id(), 2000)]);
    let series_id: String = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
            "token_metadata": token_metadata,
            "royalty": royalty,
        }))?
        .deposit(parse_near!("2 N"))
        .transact()
        .await?
        .json()?;

    // Only authorized account can mint
    owner
        .call(&worker, nft.id().clone(), "set_private_minting")
        .args_json(serde_json::json!({
            "enabled": true,
        }))?
        .transact()
        .await?;
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_mint")
        .args_json(serde_json::json!({
            "token_series_id": series_id,
            "receiver_id": user1.id()
        }))?
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err
            .to_string()
            .contains("Access to mint is denied for this contract"))
    } else {
        panic!("Expected failure")
    };

    owner
        .call(&worker, nft.id().clone(), "set_private_minting")
        .args_json(serde_json::json!({
            "enabled": false,
        }))?
        .transact()
        .await?;

    // wrong series_id
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_mint")
        .args_json(serde_json::json!({
            "token_series_id": "3",
            "receiver_id": user1.id()
        }))?
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err.to_string().contains("Token series does not exist"))
    } else {
        panic!("Expected failure")
    };

    // only owner can mint
    let outcome = user3
        .call(&worker, nft.id().clone(), "nft_mint")
        .args_json(serde_json::json!({
            "token_series_id": series_id,
            "receiver_id": user1.id()
        }))?
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err.to_string().contains("permission denied"))
    } else {
        panic!("Expected failure")
    };

    // Exceed max tokens
    user1
        .call(&worker, nft.id().clone(), "nft_mint")
        .args_json(serde_json::json!({
            "token_series_id": series_id,
            "receiver_id": user1.id()
        }))?
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_mint")
        .args_json(serde_json::json!({
            "token_series_id": series_id,
            "receiver_id": user1.id()
        }))?
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err.to_string().contains("Max token minted"))
    } else {
        panic!("Expected failure")
    };
    Ok(())
}

/*
- Mints a new token
- Refunds a deposit
 */
#[tokio::test]
async fn nft_mint_positive() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let user2 = owner
        .create_subaccount(&worker, "user2")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let mut token_metadata = TokenMetadata {
        title: Some("some title".to_string()),
        description: None,
        media: Some("ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz".to_string()),
        media_hash: None,
        copies: Some(1),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    };
    let royalty = HashMap::from([(user1.id(), 500), (user2.id(), 2000)]);
    let series_id: String = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
            "token_metadata": token_metadata,
            "royalty": royalty,
        }))?
        .deposit(parse_near!("2 N"))
        .transact()
        .await?
        .json()?;

    let token_id: String = user1
        .call(&worker, nft.id().clone(), "nft_mint")
        .args_json(serde_json::json!({
            "token_series_id": series_id,
            "receiver_id": user2.id()
        }))?
        .deposit(parse_near!("1 N"))
        .transact()
        .await?
        .json()?;
    let minted_token: Token = nft
        .view(
            &worker,
            "nft_token",
            serde_json::json!({ "token_id": token_id })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    let minted_token_metadata = minted_token.metadata.as_ref().unwrap();
    token_metadata.issued_at = minted_token_metadata.issued_at.clone();
    token_metadata.copies = None;
    assert_eq!(
        minted_token,
        Token {
            token_id,
            owner_id: user2.id().as_ref().parse().unwrap(),
            metadata: Some(token_metadata),
            approved_account_ids: Some(Default::default())
        }
    );
    Ok(())
}

/*
- Should panic unless 1 yoctoNEAR is attached
- Panics if `token_id` which doesn't exist
- Panics if the number of royalties exceeds `max_len_payout`
- Panics if invalid `memo` is provided
- Panics if total payout exceeds `ROYALTY_TOTAL_VALUE`
*/
#[tokio::test]
async fn nft_transfer_payout_negative() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let user2 = owner
        .create_subaccount(&worker, "user2")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();

    let user3 = owner
        .create_subaccount(&worker, "user3")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();

    let series = create_series_raw(
        &worker,
        nft.id().clone(),
        &user1,
        Some(4),
        HashMap::from([
            (user1.id(), 500),
            (&"acc1.near".parse().unwrap(), 100),
            (&"acc2.near".parse().unwrap(), 100),
            (&"acc3.near".parse().unwrap(), 100),
            (&"acc4.near".parse().unwrap(), 100),
            (&"acc5.near".parse().unwrap(), 100),
            (&"acc6.near".parse().unwrap(), 100),
        ]),
    )
    .await?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": user2.id(),
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    let approval_id: u64 = {
        let token: Token = nft
            .view(
                &worker,
                "nft_token",
                serde_json::json!({ "token_id": token1 })
                    .to_string()
                    .into_bytes(),
            )
            .await?
            .json()?;
        let approval_account_ids = token.approved_account_ids.unwrap();
        *approval_account_ids
            .get(&user2.id().as_ref().parse().unwrap())
            .unwrap()
    };
    // 1 yoctoNEAR not attached
    let outcome = user2
        .call(&worker, nft.id().clone(), "nft_transfer_payout")
        .args_json(serde_json::json!({
            "receiver_id": user3.id(),
            "token_id": token1,
            "approval_id": approval_id,
            "balance": "10000",
            "max_len_payout": 10,
        }))?
        .transact()
        .await?;
    check_outcome_fail(
        outcome.status,
        "Requires attached deposit of exactly 1 yoctoNEAR",
    )
    .await;

    // `token_id` contains `token_series_id`, which doesn't exist
    let outcome = user2
        .call(&worker, nft.id().clone(), "nft_transfer_payout")
        .args_json(serde_json::json!({
            "receiver_id": user3.id(),
            "token_id": "2:1",
            "approval_id": approval_id,
            "balance": "10000",
            "max_len_payout": 10,
        }))?
        .deposit(1)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "no token id").await;

    // number of royalties exceeds `max_len_payout`
    let outcome = user2
        .call(&worker, nft.id().clone(), "nft_transfer_payout")
        .args_json(serde_json::json!({
            "receiver_id": user3.id(),
            "token_id": token1,
            "approval_id": approval_id,
            "balance": "10000",
            "max_len_payout": 5,
        }))?
        .deposit(1)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Too many recievers").await;

    // invalid `memo` is provided
    let outcome = user2
        .call(&worker, nft.id().clone(), "nft_transfer_payout")
        .args_json(serde_json::json!({
            "receiver_id": user3.id(),
            "token_id": token1,
            "approval_id": approval_id,
            "memo": "some_wrong_memo",
            "balance": "10000",
            "max_len_payout": 10,
        }))?
        .deposit(1)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "invalid FeesArgs").await;

    // if total payout exceeds `ROYALTY_TOTAL_VALUE`
    let fees = Fees {
        buyer: HashMap::from([
            ("acc1.near".parse().unwrap(), 100),
            ("acc2.near".parse().unwrap(), 100),
            ("acc3.near".parse().unwrap(), 100),
            ("acc4.near".parse().unwrap(), 100),
            ("acc5.near".parse().unwrap(), 100),
            ("acc6.near".parse().unwrap(), 100),
        ]),
        seller: HashMap::from([
            ("acc7.near".parse().unwrap(), 100),
            ("acc8.near".parse().unwrap(), 100),
            ("acc9.near".parse().unwrap(), 100),
            ("acc10.near".parse().unwrap(), 100),
            ("acc11.near".parse().unwrap(), 100),
            ("acc12.near".parse().unwrap(), 100),
        ]),
    };
    let outcome = user2
        .call(&worker, nft.id().clone(), "nft_transfer_payout")
        .args_json(serde_json::json!({
            "receiver_id": user3.id(),
            "token_id": token1,
            "approval_id": approval_id,
            "memo": serde_json::json!(fees).to_string(),
            "balance": "10000",
            "max_len_payout": 10,
        }))?
        .deposit(1)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Too many recievers").await;
    Ok(())
}

// - Returns payout, which contains royalties and payouts from `memo`
// Checking calculations here
#[tokio::test]
async fn nft_transfer_payout_positive() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let user2 = owner
        .create_subaccount(&worker, "user2")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();

    let user3 = owner
        .create_subaccount(&worker, "user3")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();

    let parsed_near = parse_near!("2.01 N").into();
    let payouts = nft_transfer_payout_helper(
        &worker,
        &nft,
        &user1,
        &user2,
        &user3,
        HashMap::from([(user1.id(), 500)]),
        Fees {
            buyer: HashMap::from([(user2.id().as_ref().parse().unwrap(), 300)]),
            seller: HashMap::from([(user2.id().as_ref().parse().unwrap(), 300)]),
        },
        parsed_near,
    )
    .await;

    let sum: u128 = payouts.payout.values().map(|val| val.0).sum();
    assert!(parsed_near.0 - sum <= 1);

    let parsed_near = parse_near!("1.23 N").into();
    let payouts = nft_transfer_payout_helper(
        &worker,
        &nft,
        &user1,
        &user2,
        &user3,
        HashMap::from([(user1.id(), 500)]),
        Fees {
            buyer: HashMap::from([(user2.id().as_ref().parse().unwrap(), 300)]),
            seller: HashMap::from([(user2.id().as_ref().parse().unwrap(), 300)]),
        },
        parsed_near,
    )
    .await;

    let sum: u128 = payouts.payout.values().map(|val| val.0).sum();
    assert!(parsed_near.0 - sum <= 1);

    let parsed_near = parse_near!("3.45 N").into();
    let payouts = nft_transfer_payout_helper(
        &worker,
        &nft,
        &user1,
        &user2,
        &user3,
        HashMap::from([(user1.id(), 500)]),
        Fees {
            buyer: HashMap::from([(user2.id().as_ref().parse().unwrap(), 300), (user1.id().as_ref().parse().unwrap(), 500)]),
            seller: HashMap::from([(user2.id().as_ref().parse().unwrap(), 300)]),
        },
        parsed_near,
    )
    .await;

    let sum: u128 = payouts.payout.values().map(|val| val.0).sum();
    assert!(parsed_near.0 - sum <= 1);
    Ok(())
}
