use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::utils::{
    check_outcome_fail, create_series, create_series_raw, deposit, init_market, init_nft,
    mint_token, nft_approve, offer,
};
use near_contract_standards::non_fungible_token::Token;
use near_units::{parse_gas, parse_near};
use nft_bid_market::{ArgsKind, SaleArgs, SaleJson, BID_HISTORY_LENGTH_DEFAULT};
use nft_contract::common::{AccountId, U128, U64};

/*
- Can only be called via cross-contract call
- `owner_id` must be the signer
- Panics if `owner_id` didn't pay for one more sale/auction
- Panics if the given `ft_token_id` is not supported by the market
- Panics if `msg` doesn't contain valid parameters for sale or auction
 */
#[tokio::test]
async fn nft_on_approve_negative() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();

    let series: String = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
        "token_metadata":
        {
            "title": "some title",
            "media": "ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz",
            "copies": 10
        },
        "royalty":
        {
            owner.id().as_ref(): 1000
        }}))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?
        .json()?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;

    // try to call nft_on_approve without cross contract call
    let outcome = user1
        .call(&worker, market.id().clone(), "nft_on_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "owner_id": user1.id(),
            "approval_id": 1,
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: HashMap::from([("near".parse().unwrap(), 10000.into())]),
                token_type: Some(series.clone()),
                start: None,
                end: None,
                origins: None,
            })).to_string()
        }))?
        .transact()
        .await?;
    check_outcome_fail(
        outcome.status,
        "nft_on_approve should only be called via cross-contract call",
    )
    .await;

    // TODO: to test `owner_id` must be the signer need to create another contract

    // fail without storage deposit
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: HashMap::from([("near".parse().unwrap(), 10000.into())]),
                token_type: Some(series.clone()),
                start: None,
                end: None,
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Insufficient storage paid").await;

    // not supported ft
    deposit(&worker, market.id().clone(), &user1).await;
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: HashMap::from([("ft.near".parse().unwrap(), 10000.into())]),
                token_type: Some(series),
                start: None,
                end: None,
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Token ft.near not supported by this market").await;

    // bad message, sale/auction shouldn't be added
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!({
                    "a": "b"
            }).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Not valid args").await;

    Ok(())
}

/*
- Start time is set to `block_timestamp` if it is not specified explicitly
- Creates a new sale/auction
 */
#[tokio::test]
async fn nft_on_approve_positive() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();

    let series: String = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
        "token_metadata":
        {
            "title": "some title",
            "media": "ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz",
            "copies": 10
        },
        "royalty":
        {
            owner.id().as_ref(): 1000
        }}))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?
        .json()?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;

    deposit(&worker, market.id().clone(), &user1).await;
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: HashMap::from([("near".parse().unwrap(), 10000.into())]),
                token_type: Some(series.clone()),
                start: None,
                end: None,
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let sale_json: SaleJson = market
        .view(
            &worker,
            "get_sale",
            serde_json::json!({
               "nft_contract_id": nft.id(),
               "token_id": token1
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    let time_passed = since_the_epoch - Duration::from_nanos(sale_json.start.unwrap().0);
    assert!(time_passed < Duration::from_secs(60)); // shouldn't be 60 secs even in worse case
    Ok(())
}

/**
    - Should panic if there is no sale with given `contract_and_token_id`
    - Should panic if the sale is not in progress
    - Should panic if the NFT owner tries to make a bid on his own sale
    - Should panic if the deposit equal to 0
    - Should panic if the NFT can't be bought by `ft_token_id`
- If the `attached_deposit` is equal to the price + fees
  -  panics if number of payouts plus number of bids exceeds 10
- If the `attached_deposit` is not equal to the price + fees
  - should panic if `ft_token_id` is not supported
  - panics if the bid smaller or equal to the previous one
  - panic if origin fee exceeds ORIGIN_FEE_MAX
    */
#[tokio::test]
async fn offer_negative() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

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

    // No sale with given `contract_and_token_id`
    let outcome = user1
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": "1:1",
            "ft_token_id": "near",
        }))?
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "No sale").await;

    // Sale is not in progress
    let series = create_series(&worker, nft.id().clone(), &user1, owner.id().clone()).await?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;

    deposit(&worker, market.id().clone(), &user1).await;
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let waiting_time = Duration::from_secs(15);
    let epoch_plus_waiting_time = (since_the_epoch + waiting_time).as_nanos();
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: sale_conditions.clone(),
                token_type: Some(series.clone()),
                start: Some(U64(epoch_plus_waiting_time as u64)),
                end: None,
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .transact()
        .await?;
    check_outcome_fail(
        outcome.status,
        "Either the sale is finished or it hasn't started yet",
    )
    .await;

    tokio::time::sleep(waiting_time).await;
    let price: U128 = market
        .view(
            &worker,
            "price_with_fees",
            serde_json::json!({
                "price": sale_conditions.get(&AccountId::new_unchecked("near".to_string())).unwrap(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    // NFT owner tries to make a bid on his own sale
    let outcome = user1
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(price.into())
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Cannot bid on your own sale.").await;

    // Deposit equal to 0
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(0)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Attached deposit must be greater than 0").await;

    // Not supported ft
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "nearcoin",
        }))?
        .deposit(1000)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Not supported ft").await;

    // the bid smaller or equal to the previous one
    user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(500)
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(400) // less
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(
        outcome.status,
        "Can't pay less than or equal to current bid price:",
    )
    .await;
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(500) // equal
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(
        outcome.status,
        "Can't pay less than or equal to current bid price:",
    )
    .await;

    // Exceeding ORIGIN_FEE_MAX
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
            "origins": {
                "user1": 4701,
            }
        }))?
        .deposit(2000) // equal
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Max origins exceeded").await;

    // number of payouts plus number of bids exceeds 10
    let too_much_origins: HashMap<AccountId, u32> = HashMap::from([
        ("acc1.near".parse().unwrap(), 100),
        ("acc2.near".parse().unwrap(), 100),
        ("acc3.near".parse().unwrap(), 100),
        ("acc4.near".parse().unwrap(), 100),
        ("acc5.near".parse().unwrap(), 100),
        ("acc6.near".parse().unwrap(), 100),
        ("acc7.near".parse().unwrap(), 100),
        ("acc8.near".parse().unwrap(), 100),
        ("acc9.near".parse().unwrap(), 100),
        ("acc10.near".parse().unwrap(), 100),
        ("acc11.near".parse().unwrap(), 100),
        ("acc12.near".parse().unwrap(), 100),
    ]);
    let price: U128 = market
        .view(
            &worker,
            "price_with_fees",
            serde_json::json!({
                "price": sale_conditions.get(&AccountId::new_unchecked("near".to_string())).unwrap(),
                "origins": too_much_origins
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(price.into())
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    // Promise of offer returning empty value, because of panic on nft_transfer_payout, but
    // TODO: we need to check Failure on nft contract when workspaces add feature to check not only FinalExecutionStatus
    if let near_primitives::views::FinalExecutionStatus::SuccessValue(empty_string) = outcome.status
    {
        assert!(empty_string.is_empty())
    } else {
        panic!("Expected failure {:?}", outcome.status)
    };

    Ok(())
}

/*
- If the `attached_deposit` is equal to the price + fees
    -  NFT is transferred to the buyer
    -  the sale is removed from the list of sales
    -  ft transferred to the previous owner
    -  protocol, royalty and origin fees are paid
    -  royalty paid from seller side
    -  previous bids refunded
- If the `attached_deposit` is not equal to the price + fees
  - a new bid should be added
  - if the number of stored bids exceeds `bid_history_length`, the earliest bid is removed and refunded
*/
#[tokio::test]
async fn offer_positive() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

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

    let series = create_series(&worker, nft.id().clone(), &user1, owner.id().clone()).await?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;

    deposit(&worker, market.id().clone(), &user1).await;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    nft_approve(
        &worker,
        nft.id().clone(),
        market.id().clone(),
        &user1,
        token1.clone(),
        sale_conditions.clone(),
        series.clone(),
    )
    .await;
    let price: U128 = market
        .view(
            &worker,
            "price_with_fees",
            serde_json::json!({
                "price": sale_conditions.get(&AccountId::new_unchecked("near".to_string())).unwrap(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    let before_sell: Option<SaleJson> = market
        .view(
            &worker,
            "get_sale",
            serde_json::json!({
               "nft_contract_id": nft.id(),
               "token_id": token1
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(price.into())
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;

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

    // NFT is transferred to the buyer
    assert_eq!(token.owner_id.as_str(), user2.id().as_ref());
    // the sale is removed from the list of sales
    let after_sell: Option<SaleJson> = market
        .view(
            &worker,
            "get_sale",
            serde_json::json!({
               "nft_contract_id": nft.id(),
               "token_id": token1
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert!(
        after_sell.is_none(),
        "Sale is still active, when it shouldn't"
    );
    assert!(
        before_sell.is_some(),
        "Sale is not active, when it should be"
    );

    // Check if bids can be added
    let token2 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    nft_approve(
        &worker,
        nft.id().clone(),
        market.id().clone(),
        &user1,
        token2.clone(),
        sale_conditions.clone(),
        series.clone(),
    )
    .await;
    let initial_price = 100;
    user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token2,
            "ft_token_id": "near",
        }))?
        .deposit(initial_price)
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    let sale_json: SaleJson = market
        .view(
            &worker,
            "get_sale",
            serde_json::json!({
               "nft_contract_id": nft.id(),
               "token_id": token2
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    let bids = sale_json
        .bids
        .get(&AccountId::new_unchecked("near".to_string()))
        .unwrap();
    assert!(bids.get(0).is_some(), "Bid not added");

    let first_bid = bids.get(0).unwrap();
    // Earliest bid should be removed
    for i in 1..=BID_HISTORY_LENGTH_DEFAULT {
        user2
            .call(&worker, market.id().clone(), "offer")
            .args_json(serde_json::json!({
                "nft_contract_id": nft.id(),
                "token_id": token2,
                "ft_token_id": "near",
            }))?
            .deposit(initial_price * (i + 1) as u128)
            .gas(parse_gas!("300 Tgas") as u64)
            .transact()
            .await?;
        let sale_json: SaleJson = market
            .view(
                &worker,
                "get_sale",
                serde_json::json!({
                   "nft_contract_id": nft.id(),
                   "token_id": token2
                })
                .to_string()
                .into_bytes(),
            )
            .await?
            .json()?;
        let bids = sale_json
            .bids
            .get(&AccountId::new_unchecked("near".to_string()))
            .unwrap();
        if i < BID_HISTORY_LENGTH_DEFAULT {
            assert_eq!(bids.get(0).unwrap(), first_bid);
        }
    }
    // new bid removed last bid
    let sale_json: SaleJson = market
        .view(
            &worker,
            "get_sale",
            serde_json::json!({
               "nft_contract_id": nft.id(),
               "token_id": token2
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    let bids = sale_json
        .bids
        .get(&AccountId::new_unchecked("near".to_string()))
        .unwrap();

    assert_ne!(bids.get(0).unwrap(), first_bid);
    Ok(())
}

/*
- Should panic if there is no sale with the given `nft_contract_id` and `token_id`
- Should panic if there are no bids with given fungible token
- Should panic if the sale is not in progress
- Should panic if the last bid is out of time
 */
#[tokio::test]
async fn accept_offer_negative() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

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
    let series = create_series_raw(
        &worker,
        nft.id().clone(),
        &user1,
        Some(4),
        HashMap::from([(user1.id(), 500)]),
    )
    .await?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id().clone(), &user1).await;

    // No sale with the given `nft_contract_id` and `token_id`
    let outcome = user1
        .call(&worker, market.id().clone(), "accept_offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "No sale").await;

    // no bids with given fungible token
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 42000.into())]);
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let waiting_time = Duration::from_secs(10);
    let epoch_plus_waiting_time = (since_the_epoch + waiting_time).as_nanos();
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: sale_conditions.clone(),
                token_type: Some(series.clone()),
                start: None,
                end: Some(U64(epoch_plus_waiting_time as u64)),
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    let outcome = user1
        .call(&worker, market.id().clone(), "accept_offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "No bids").await;

    // last bid is out of time
    user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
            "duration": "1",
        }))?
        .deposit(200)
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    tokio::time::sleep(Duration::from_nanos(1)).await;
    let outcome = user1
        .call(&worker, market.id().clone(), "accept_offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Out of time limit of the bid").await;
    // Sale is not in progress
    tokio::time::sleep(waiting_time).await;
    let outcome = user1
        .call(&worker, market.id().clone(), "accept_offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(
        outcome.status,
        "Either the sale is finished or it hasn't started yet",
    )
    .await;
    Ok(())
}

// - Nft transfered to the buyer
#[tokio::test]
async fn accept_offer_positive() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

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
    let series = create_series_raw(
        &worker,
        nft.id().clone(),
        &user1,
        Some(4),
        HashMap::from([(user1.id(), 500)]),
    )
    .await?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id().clone(), &user1).await;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 42000.into())]);
    nft_approve(
        &worker,
        nft.id().clone(),
        market.id().clone(),
        &user1,
        token1.clone(),
        sale_conditions.clone(),
        series.clone(),
    )
    .await;
    user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(200)
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    user1
        .call(&worker, market.id().clone(), "accept_offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    let token_data: Token = nft
        .view(
            &worker,
            "nft_token",
            serde_json::json!({ "token_id": token1 })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(token_data.owner_id.as_ref(), user2.id().as_ref());
    Ok(())
}

/*
- Should panic unless 1 yoctoNEAR is attached
- Should panic if there is no sale with the given `nft_contract_id` and `token_id`
- Should panic unless it is called by the creator of the sale
- Should panic if `ft_token_id` is not supported
*/
#[tokio::test]
async fn update_price_negative() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

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
    let series = create_series_raw(
        &worker,
        nft.id().clone(),
        &user1,
        Some(4),
        HashMap::from([(user1.id(), 500)]),
    )
    .await?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id().clone(), &user1).await;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 42000.into())]);
    nft_approve(
        &worker,
        nft.id().clone(),
        market.id().clone(),
        &user1,
        token1.clone(),
        sale_conditions.clone(),
        series.clone(),
    )
    .await;

    // not attaching 1 yocto
    let outcome = user1
        .call(&worker, market.id().clone(), "update_price")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
            "price": "10000",
        }))?
        .transact()
        .await?;
    check_outcome_fail(
        outcome.status,
        "Requires attached deposit of exactly 1 yoctoNEAR",
    )
    .await;

    // no sale with given nft_contract_id:token_id
    let outcome = user1
        .call(&worker, market.id().clone(), "update_price")
        .args_json(serde_json::json!({
            "nft_contract_id": market.id(),
            "token_id": token1,
            "ft_token_id": "near",
            "price": "10000",
        }))?
        .deposit(1)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "No sale").await;

    // called not by the owner
    let outcome = user2
        .call(&worker, market.id().clone(), "update_price")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
            "price": "10000",
        }))?
        .deposit(1)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Must be sale owner").await;

    // ft must be supported
    let outcome = user1
        .call(&worker, market.id().clone(), "update_price")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "nearcoin",
            "price": "10000",
        }))?
        .deposit(1)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "is not supported by this market").await;
    Ok(())
}

// Changes the price
#[tokio::test]
async fn update_price_positive() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();

    let series = create_series_raw(
        &worker,
        nft.id().clone(),
        &user1,
        Some(4),
        HashMap::from([(user1.id(), 500)]),
    )
    .await?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id().clone(), &user1).await;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 42000.into())]);
    nft_approve(
        &worker,
        nft.id().clone(),
        market.id().clone(),
        &user1,
        token1.clone(),
        sale_conditions.clone(),
        series.clone(),
    )
    .await;
    user1
        .call(&worker, market.id().clone(), "update_price")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
            "price": "10000",
        }))?
        .deposit(1)
        .transact()
        .await?;

    let sale_json: SaleJson = market
        .view(
            &worker,
            "get_sale",
            serde_json::json!({
               "nft_contract_id": nft.id(),
               "token_id": token1
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(
        sale_json.sale_conditions.get(&"near".parse().unwrap()),
        Some(&U128(10000))
    );
    Ok(())
}

/*
- Should panic unless 1 yoctoNEAR is attached
- If the sale in progress, only the sale creator can remove the sale
 */
#[tokio::test]
async fn remove_sale_negative() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

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

    let series = create_series_raw(
        &worker,
        nft.id().clone(),
        &user1,
        Some(4),
        HashMap::from([(user1.id(), 500)]),
    )
    .await?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id().clone(), &user1).await;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 42000.into())]);
    nft_approve(
        &worker,
        nft.id().clone(),
        market.id().clone(),
        &user1,
        token1.clone(),
        sale_conditions.clone(),
        series.clone(),
    )
    .await;

    // 1 yocto is needed
    let outcome = user1
        .call(&worker, market.id().clone(), "remove_sale")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1
        }))?
        .transact()
        .await?;
    check_outcome_fail(
        outcome.status,
        "Requires attached deposit of exactly 1 yoctoNEAR",
    )
    .await;

    // Can be removed only by the owner of the sale, if not finished
    let outcome = user2
        .call(&worker, market.id().clone(), "remove_sale")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1
        }))?
        .deposit(1)
        .transact()
        .await?;
    check_outcome_fail(
        outcome.status,
        "Until the sale is finished, it can only be removed by the sale owner",
    )
    .await;
    Ok(())
}

/*
- Sale removed
- Refunds all bids
*/
#[tokio::test]
async fn remove_sale_positive() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

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

    let series = create_series_raw(
        &worker,
        nft.id().clone(),
        &user1,
        Some(4),
        HashMap::from([(user1.id(), 500)]),
    )
    .await?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id().clone(), &user1).await;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 42000.into())]);
    nft_approve(
        &worker,
        nft.id().clone(),
        market.id().clone(),
        &user1,
        token1.clone(),
        sale_conditions.clone(),
        series.clone(),
    )
    .await;
    offer(
        &worker,
        nft.id().clone(),
        market.id().clone(),
        &user2,
        token1.clone(),
        4000.into(),
    )
    .await;
    user1
        .call(&worker, market.id().clone(), "remove_sale")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1
        }))?
        .deposit(1)
        .transact()
        .await?;
    let sale_json: Option<SaleJson> = market
        .view(
            &worker,
            "get_sale",
            serde_json::json!({
               "nft_contract_id": nft.id(),
               "token_id": token1
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert!(sale_json.is_none());
    Ok(())
}
