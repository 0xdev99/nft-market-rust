use std::{
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use near_units::{parse_gas, parse_near};
use crate::utils::{init_market, init_nft, create_subaccount, create_series, deposit,
    mint_token, check_outcome_success
};
use nft_bid_market::{ArgsKind, AuctionArgs, AuctionJson};
use nft_contract::common::AccountId;
use nft_contract::common::{U64, U128};

#[tokio::test]
async fn view_auction_get_auction() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(
        &worker,
        worker.root_account().id(),
        vec![nft.id()]
    ).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;

    let series = create_series(
        &worker,
        nft.id().clone(),
        &user1,
        owner.id().clone()
    ).await?;
    let token1 = mint_token(
        &worker,
        nft.id().clone(),
        &user1,
        user1.id(),
        &series
    ).await?;

    deposit(&worker, market.id().clone(), &user1).await;
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Auction(AuctionArgs {
                token_type: None,
                minimal_step: 100.into(),
                start_price: 10000.into(),
                start: None,
                duration: 900000000000.into(),
                buy_out_price: Some(10000000000.into()),
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    // Check that method fails in case of wrong `auction_id` 
    let outcome = market
        .view(
            &worker,
            "get_auction",
            serde_json::json!({ "auction_id": "1".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await;
    //println!("{}, {}", outcome.result, outcome.logs);
    match outcome {
        Err(err) => {
            println!("{}", err); 
            /*assert!(
                err.to_string().contains("Auction does not exist"),
                "wrong error"
            );*/
        },
        Ok(_) => panic!("Expected failure"),
    };

    // Check that method works in case of correct `auction_id` 
    let auction: AuctionJson = market
        .view(
            &worker,
            "get_auction",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    
    assert_eq!(auction.owner_id, AccountId::new_unchecked("user1.test.near".to_owned()));
    assert_eq!(auction.token_id, "1:1".to_string());
    assert_eq!(auction.ft_token_id, AccountId::new_unchecked("near".to_owned()));
    assert_eq!(auction.minimal_step.0, 100);
    assert_eq!(auction.start_price.0, 10000);
    assert_eq!(auction.buy_out_price.unwrap().0, 10000000000);
    
    Ok(())
}

#[tokio::test]
async fn view_auction_get_auctions() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(
        &worker,
        worker.root_account().id(),
        vec![nft.id()]
    ).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    let user2 = create_subaccount(&worker, &owner, "user2").await?;

    let series1 = create_series(
        &worker,
        nft.id().clone(),
        &user1,
        owner.id().clone()
    ).await?;
    let token1 = mint_token(
        &worker,
        nft.id().clone(),
        &user1,
        user1.id(),
        &series1
    ).await?;

    let series2 = create_series(
        &worker,
        nft.id().clone(),
        &user2,
        owner.id().clone()
    ).await?;
    let token2 = mint_token(
        &worker,
        nft.id().clone(),
        &user2,
        user2.id(),
        &series2
    ).await?;

    deposit(&worker, market.id().clone(), &user1).await;
    deposit(&worker, market.id().clone(), &user2).await;

    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Auction(AuctionArgs {
                token_type: None,
                minimal_step: 100.into(),
                start_price: 10000.into(),
                start: None,
                duration: 900000000000.into(),
                buy_out_price: Some(10000000000.into()),
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    user2
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token2,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Auction(AuctionArgs {
                token_type: None,
                minimal_step: 110.into(),
                start_price: 100000.into(),
                start: None,
                duration: 900000000000.into(),
                buy_out_price: Some(1000000000.into()),
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    let auctions: Vec<AuctionJson> = market
        .view(
            &worker,
            "get_auctions",
            serde_json::json!({ "from_index": null, "limit": null })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert!(auctions.len() == 2, "wrong length");
    let auction1 = &auctions[0];
    let auction2 = &auctions[1];

    assert_eq!(auction1.owner_id, AccountId::new_unchecked("user1.test.near".to_owned()));
    assert_eq!(auction1.token_id, "1:1".to_string());
    assert_eq!(auction1.ft_token_id, AccountId::new_unchecked("near".to_owned()));
    assert_eq!(auction1.minimal_step.0, 100);
    assert_eq!(auction1.start_price.0, 10000);
    assert_eq!(auction1.buy_out_price.unwrap().0, 10000000000);

    assert_eq!(auction2.owner_id, AccountId::new_unchecked("user2.test.near".to_owned()));
    assert_eq!(auction2.token_id, "2:1".to_string());
    assert_eq!(auction2.ft_token_id, AccountId::new_unchecked("near".to_owned()));
    assert_eq!(auction2.minimal_step.0, 110);
    assert_eq!(auction2.start_price.0, 100000);
    assert_eq!(auction2.buy_out_price.unwrap().0, 1000000000);

    Ok(())
}

#[tokio::test]
async fn view_auction_get_current_buyer() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(
        &worker,
        worker.root_account().id(),
        vec![nft.id()]
    ).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    let user2 = create_subaccount(&worker, &owner, "user2").await?;

    let series = create_series(
        &worker,
        nft.id().clone(),
        &user1,
        owner.id().clone()
    ).await?;
    let token1 = mint_token(
        &worker,
        nft.id().clone(),
        &user1,
        user1.id(),
        &series
    ).await?;

    deposit(&worker, market.id().clone(), &user1).await;
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Auction(AuctionArgs {
                token_type: None,
                minimal_step: 100.into(),
                start_price: 10000.into(),
                start: None,
                duration: 900000000000.into(),
                buy_out_price: Some(10000000000.into()),
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    // Check that method fails in case of wrong `auction_id` 
    let outcome = market
        .view(
            &worker,
            "get_current_buyer",
            serde_json::json!({ "auction_id": "1".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await;
    match outcome {
        Err(err) => {
            println!("{}", err); 
            /*assert!(
                err.to_string().contains("Auction does not exist"),
                "wrong error"
            );*/
        },
        Ok(_) => panic!("Expected failure"),
    };

    let current_buyer: Option<AccountId> = market
        .view(
            &worker,
            "get_current_buyer",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert!(current_buyer.is_none(), "Should be None");

    let outcome = user2
        .call(&worker, market.id().clone(), "auction_add_bid")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string(),
        }))?
        .deposit(10300)
        .transact()
        .await?;
    check_outcome_success(outcome.status).await;
    let current_buyer: Option<AccountId> = market
        .view(
            &worker,
            "get_current_buyer",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert!(current_buyer.is_some(), "Should be some account");
    assert_eq!(
        current_buyer.unwrap(),
        AccountId::new_unchecked("user2.test.near".to_owned()),
        "wrong account"
    );

    Ok(())
}

#[tokio::test]
async fn view_auction_get_current_bid() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(
        &worker,
        worker.root_account().id(),
        vec![nft.id()]
    ).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    let user2 = create_subaccount(&worker, &owner, "user2").await?;

    let series = create_series(
        &worker,
        nft.id().clone(),
        &user1,
        owner.id().clone()
    ).await?;
    let token1 = mint_token(
        &worker,
        nft.id().clone(),
        &user1,
        user1.id(),
        &series
    ).await?;

    deposit(&worker, market.id().clone(), &user1).await;
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Auction(AuctionArgs {
                token_type: None,
                minimal_step: 100.into(),
                start_price: 10000.into(),
                start: None,
                duration: 900000000000.into(),
                buy_out_price: Some(10000000000.into()),
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    // Check that method fails in case of wrong `auction_id` 
    let outcome = market
        .view(
            &worker,
            "get_current_bid",
            serde_json::json!({ "auction_id": "1".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await;
    match outcome {
        Err(err) => {
            println!("{}", err); 
            /*assert!(
                err.to_string().contains("Auction does not exist"),
                "wrong error"
            );*/
        },
        Ok(_) => panic!("Expected failure"),
    };

    let current_bid: Option<U128> = market
        .view(
            &worker,
            "get_current_bid",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert!(current_bid.is_none(), "Should not be any bids");

    // add a bid with deposit 10300
    // 300 yocto is protocol see
    let outcome = user2
        .call(&worker, market.id().clone(), "auction_add_bid")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string(),
        }))?
        .deposit(10300)
        .transact()
        .await?;
    check_outcome_success(outcome.status).await;
    let current_bid: Option<U128> = market
        .view(
            &worker,
            "get_current_bid",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert!(current_bid.is_some(), "Should be a bid");
    assert_eq!(
        current_bid.unwrap().0, 
        10000,
        "wrong amount"
    );

    Ok(())
}

#[tokio::test]
async fn view_auction_get_minimal_next_bid() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(
        &worker,
        worker.root_account().id(),
        vec![nft.id()]
    ).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    let user2 = create_subaccount(&worker, &owner, "user2").await?;

    let series = create_series(
        &worker,
        nft.id().clone(),
        &user1,
        owner.id().clone()
    ).await?;
    let token1 = mint_token(
        &worker,
        nft.id().clone(),
        &user1,
        user1.id(),
        &series
    ).await?;

    deposit(&worker, market.id().clone(), &user1).await;
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Auction(AuctionArgs {
                token_type: None,
                minimal_step: 100.into(),
                start_price: 10000.into(),
                start: None,
                duration: 900000000000.into(),
                buy_out_price: Some(10000000000.into()),
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    // Check that method fails in case of wrong `auction_id` 
    let outcome = market
        .view(
            &worker,
            "get_minimal_next_bid",
            serde_json::json!({ "auction_id": "1".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await;
    match outcome {
        Err(err) => {
            println!("{}", err); 
            /*assert!(
                err.to_string().contains("Auction does not exist"),
                "wrong error"
            );*/
        },
        Ok(_) => panic!("Expected failure"),
    };
    
    let min_bid: U128 = market
        .view(
            &worker,
            "get_minimal_next_bid",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(min_bid.0, 10000, "Should be initial price");

    // add a bid with deposit 103000
    // this bid without fees is equal to 100000
    // the next bid (without fees) is equal to 100100
    let outcome = user2
        .call(&worker, market.id().clone(), "auction_add_bid")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string(),
        }))?
        .deposit(103000)
        .transact()
        .await?;
    check_outcome_success(outcome.status).await;
    let min_bid: U128 = market
        .view(
            &worker,
            "get_minimal_next_bid",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(min_bid.0, 100100, "wrong next bid");

    Ok(())
}

#[tokio::test]
async fn view_auction_check_auction_in_progress() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(
        &worker,
        worker.root_account().id(),
        vec![nft.id()]
    ).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;

    let series = create_series(
        &worker,
        nft.id().clone(),
        &user1,
        owner.id().clone()
    ).await?;
    let token1 = mint_token(
        &worker,
        nft.id().clone(),
        &user1,
        user1.id(),
        &series
    ).await?;
    let token2 = mint_token(
        &worker,
        nft.id().clone(),
        &user1,
        user1.id(),
        &series
    ).await?;
    deposit(&worker, market.id().clone(), &user1).await;

    // create an auction that starts now
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Auction(AuctionArgs {
                token_type: None,
                minimal_step: 100.into(),
                start_price: 10000.into(),
                start: None,
                duration: 900000000000.into(),
                buy_out_price: Some(10000000000.into()),
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    // Check that method fails in case of wrong `auction_id` 
    let outcome = market
        .view(
            &worker,
            "check_auction_in_progress",
            serde_json::json!({ "auction_id": "1".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await;
    match outcome {
        Err(err) => {
            println!("{}", err); 
            /*assert!(
                err.to_string().contains("Auction does not exist"),
                "wrong error"
            );*/
        },
        Ok(_) => panic!("Expected failure"),
    };
    
    let in_progress: bool = market
        .view(
            &worker,
            "check_auction_in_progress",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert!(in_progress, "The auction should be in progress");

    // create an auction which starts one minute after now
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let waiting_time = Duration::from_secs(60);
    let epoch_plus_waiting_time = (since_the_epoch + waiting_time).as_nanos();
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token2,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Auction(AuctionArgs {
                token_type: None,
                minimal_step: 100.into(),
                start_price: 10000.into(),
                start: Some(U64(epoch_plus_waiting_time as u64)),
                duration: 900000000000.into(),
                buy_out_price: Some(10000000000.into()),
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    let in_progress: bool = market
        .view(
            &worker,
            "check_auction_in_progress",
            serde_json::json!({ "auction_id": "1".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert!(!in_progress, "The auction already started");

    // TODO: check `check_auction_in_progress` if auction is ended
    
    Ok(())
}