use std::collections::HashMap;

use near_units::{parse_gas, parse_near};
use nft_bid_market::SaleJson;
use nft_contract::common::U64;

use crate::utils::{create_series, deposit, init_market, init_nft, mint_token, nft_approve, offer};

#[tokio::test]
async fn sale_views() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("100 N"))
        .transact()
        .await?
        .unwrap();

    let user2 = owner
        .create_subaccount(&worker, "user2")
        .initial_balance(parse_near!("100 N"))
        .transact()
        .await?
        .unwrap();

    let series1 = create_series(&worker, nft.id().clone(), &user1, owner.id().clone()).await?;
    let mut tokens_series1 = vec![];
    for _ in 0..3 {
        tokens_series1
            .push(mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series1).await?);
    }
    let series2 = create_series(&worker, nft.id().clone(), &user2, owner.id().clone()).await?;
    let mut tokens_series2 = vec![];
    for _ in 0..2 {
        tokens_series2
            .push(mint_token(&worker, nft.id().clone(), &user2, user2.id(), &series2).await?);
    }
    deposit(&worker, market.id().clone(), &user1).await;
    deposit(&worker, market.id().clone(), &user2).await;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    for token1 in tokens_series1.iter() {
        nft_approve(
            &worker,
            nft.id().clone(),
            market.id().clone(),
            &user1,
            token1.clone(),
            sale_conditions.clone(),
            series1.clone(),
        )
        .await;
    }
    for token2 in tokens_series2.iter() {
        nft_approve(
            &worker,
            nft.id().clone(),
            market.id().clone(),
            &user2,
            token2.clone(),
            sale_conditions.clone(),
            series2.clone(),
        )
        .await;
    }

    let supply_sales: U64 = market
        .view(&worker, "get_supply_sales", vec![])
        .await?
        .json()?;
    assert_eq!(
        supply_sales.0 as usize,
        tokens_series1.len() + tokens_series2.len()
    );

    let sales: Vec<SaleJson> = market
        .view(
            &worker,
            "get_sales",
            serde_json::json!({}).to_string().into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(sales.len(), tokens_series1.len() + tokens_series2.len());
    assert!(
        tokens_series1.contains(&sales[1].token_id) || tokens_series2.contains(&sales[1].token_id)
    );

    let supply_by_owner: U64 = market
        .view(
            &worker,
            "get_supply_by_owner_id",
            serde_json::json!({
                "account_id": user1.id()
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(supply_by_owner.0 as usize, tokens_series1.len());

    let sales_user2: Vec<SaleJson> = market
        .view(
            &worker,
            "get_sales_by_owner_id",
            serde_json::json!({
                "account_id": user2.id(),
                "from_index": "0",
                "limit": 10,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(sales_user2.len(), tokens_series2.len());
    assert!(tokens_series2.contains(&sales_user2[1].token_id));

    let supply_by_nft_contract: U64 = market
        .view(
            &worker,
            "get_supply_by_nft_contract_id",
            serde_json::json!({
                "nft_contract_id": nft.id()
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(
        supply_by_nft_contract.0 as usize,
        tokens_series1.len() + tokens_series2.len()
    );

    let sales_nft_contract_id: Vec<SaleJson> = market
        .view(
            &worker,
            "get_sales_by_nft_contract_id",
            serde_json::json!({
                "nft_contract_id": nft.id(),
                "from_index": "0",
                "limit": 10,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(
        sales_nft_contract_id.len(),
        tokens_series1.len() + tokens_series2.len()
    );
    assert!(
        tokens_series1.contains(&sales_nft_contract_id[1].token_id)
            || tokens_series2.contains(&sales_nft_contract_id[1].token_id)
    );

    let supply_by_nft_token_type: U64 = market
        .view(
            &worker,
            "get_supply_by_nft_token_type",
            serde_json::json!({ "token_type": series1 })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(supply_by_nft_token_type.0 as usize, tokens_series1.len());

    let sales_nft_token_type: Vec<SaleJson> = market
        .view(
            &worker,
            "get_sales_by_nft_token_type",
            serde_json::json!({
                "token_type": series2,
                "from_index": "0",
                "limit": 10,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(sales_nft_token_type.len(), tokens_series2.len());
    assert!(tokens_series2.contains(&sales_nft_token_type[1].token_id));

    // check if removing also works correct
    {
        // case1: removed after sale
        let removed_token = tokens_series1[1].clone();
        let sale_json: Option<SaleJson> = market
            .view(
                &worker,
                "get_sale",
                serde_json::json!({
                   "nft_contract_id": nft.id(),
                   "token_id": removed_token
                })
                .to_string()
                .into_bytes(),
            )
            .await?
            .json()?;
        assert!(sale_json.is_some());
        offer(
            &worker,
            nft.id().clone(),
            market.id().clone(),
            &user2,
            tokens_series1[1].clone(),
            500.into(),
        )
        .await;
        user1
            .call(&worker, market.id().clone(), "accept_offer")
            .args_json(serde_json::json!({
                "nft_contract_id": nft.id(),
                "token_id": tokens_series1[1],
                "ft_token_id": "near",
            }))?
            .gas(parse_gas!("300 Tgas") as u64)
            .transact()
            .await?;
        tokens_series1.remove(1);
        let sale_json: Option<SaleJson> = market
            .view(
                &worker,
                "get_sale",
                serde_json::json!({
                   "nft_contract_id": nft.id(),
                   "token_id": removed_token
                })
                .to_string()
                .into_bytes(),
            )
            .await?
            .json()?;
        assert!(sale_json.is_none());
        // case2: removed after sale removed
        user2
            .call(&worker, market.id().clone(), "remove_sale")
            .args_json(serde_json::json!({
                "nft_contract_id": nft.id(),
                "token_id": tokens_series2[1]
            }))?
            .deposit(1)
            .transact()
            .await?;
        tokens_series2.remove(1);
    }

    // back to tests
    let supply_sales: U64 = market
        .view(&worker, "get_supply_sales", vec![])
        .await?
        .json()?;
    assert_eq!(
        supply_sales.0 as usize,
        tokens_series1.len() + tokens_series2.len()
    );

    let sales: Vec<SaleJson> = market
        .view(
            &worker,
            "get_sales",
            serde_json::json!({}).to_string().into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(sales.len(), tokens_series1.len() + tokens_series2.len());
    assert!(
        tokens_series1.contains(&sales[1].token_id) || tokens_series2.contains(&sales[1].token_id)
    );

    let supply_by_owner: U64 = market
        .view(
            &worker,
            "get_supply_by_owner_id",
            serde_json::json!({
                "account_id": user1.id()
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(supply_by_owner.0 as usize, tokens_series1.len());

    let sales_user2: Vec<SaleJson> = market
        .view(
            &worker,
            "get_sales_by_owner_id",
            serde_json::json!({
                "account_id": user2.id(),
                "from_index": "0",
                "limit": 10,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(sales_user2.len(), tokens_series2.len());
    assert!(tokens_series2.contains(&sales_user2[0].token_id));

    let supply_by_nft_contract: U64 = market
        .view(
            &worker,
            "get_supply_by_nft_contract_id",
            serde_json::json!({
                "nft_contract_id": nft.id()
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(
        supply_by_nft_contract.0 as usize,
        tokens_series1.len() + tokens_series2.len()
    );

    let sales_nft_contract_id: Vec<SaleJson> = market
        .view(
            &worker,
            "get_sales_by_nft_contract_id",
            serde_json::json!({
                "nft_contract_id": nft.id(),
                "from_index": "0",
                "limit": 10,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(
        sales_nft_contract_id.len(),
        tokens_series1.len() + tokens_series2.len()
    );
    assert!(
        tokens_series1.contains(&sales_nft_contract_id[1].token_id)
            || tokens_series2.contains(&sales_nft_contract_id[1].token_id)
    );

    let supply_by_nft_token_type: U64 = market
        .view(
            &worker,
            "get_supply_by_nft_token_type",
            serde_json::json!({ "token_type": series1 })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(supply_by_nft_token_type.0 as usize, tokens_series1.len());

    let sales_nft_token_type: Vec<SaleJson> = market
        .view(
            &worker,
            "get_sales_by_nft_token_type",
            serde_json::json!({
                "token_type": series2,
                "from_index": "0",
                "limit": 10,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(sales_nft_token_type.len(), tokens_series2.len());
    assert!(tokens_series2.contains(&sales_nft_token_type[0].token_id));
    Ok(())
}
