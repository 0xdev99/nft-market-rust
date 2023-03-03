use near_contract_standards::non_fungible_token::Token;
use near_units::parse_gas;
use near_units::parse_near;
use nft_bid_market::Fees;
use nft_bid_market::{ArgsKind, SaleArgs};
use nft_contract::common::TokenMetadata;
use nft_contract::common::{AccountId, U128, U64};
use nft_contract::Payout;
use std::collections::HashMap;
use workspaces::prelude::*;
use workspaces::{Account, Contract, DevNetwork, Worker};

use near_primitives::views::FinalExecutionStatus;

const NFT_WASM_FILEPATH: &str = "../res/nft_contract.wasm";
const MARKET_WASM_FILEPATH: &str = "../res/nft_bid_market.wasm";

pub async fn init_nft(
    worker: &workspaces::Worker<impl DevNetwork>,
    root_id: &workspaces::AccountId,
) -> anyhow::Result<workspaces::Contract> {
    let wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(wasm).await?;
    let outcome = contract
        .call(worker, "new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": root_id,
        }))?
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?;
    match outcome.status {
        near_primitives::views::FinalExecutionStatus::SuccessValue(_) => (),
        _ => panic!(),
    };
    Ok(contract)
}

pub async fn init_market(
    worker: &workspaces::Worker<impl DevNetwork>,
    root_id: &workspaces::AccountId,
    nft_ids: Vec<&workspaces::AccountId>,
) -> anyhow::Result<workspaces::Contract> {
    let wasm = std::fs::read(MARKET_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(wasm).await?;
    let outcome = contract
        .call(worker, "new")
        .args_json(serde_json::json!({
            "nft_ids": nft_ids,
            "owner_id": root_id,
        }))?
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?;
    match outcome.status {
        near_primitives::views::FinalExecutionStatus::SuccessValue(_) => (),
        _ => panic!(),
    };
    Ok(contract)
}

pub async fn mint_token(
    worker: &workspaces::Worker<impl DevNetwork>,
    nft_id: workspaces::AccountId,
    minter_id: &workspaces::Account,
    receiver_id: &workspaces::AccountId,
    series: &str,
) -> anyhow::Result<String> {
    let token_id = minter_id
        .call(worker, nft_id, "nft_mint")
        .args_json(serde_json::json!({
            "token_series_id": series,
            "receiver_id": receiver_id.as_ref()
        }))?
        .deposit(parse_near!("0.01 N"))
        .transact()
        .await?
        .json()?;
    Ok(token_id)
}

pub async fn check_outcome_success(status: FinalExecutionStatus) {
    assert!(
        matches!(
            status,
            near_primitives::views::FinalExecutionStatus::SuccessValue(_)
        ),
        "Panic: {:?}",
        status
    );
}

pub async fn check_outcome_fail(status: FinalExecutionStatus, expected_err: &str) {
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = status {
        assert!(
            err.to_string().contains(expected_err),
            "actual error: {}",
            err
        )
    } else {
        panic!("Expected failure, got: {:?}", status);
    };
}

pub async fn create_subaccount(
    worker: &Worker<impl DevNetwork>,
    owner: &Account,
    user_id: &str,
) -> anyhow::Result<Account> {
    let user = owner
        .create_subaccount(worker, user_id)
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    Ok(user)
}

pub async fn create_series(
    worker: &Worker<impl DevNetwork>,
    nft: workspaces::AccountId,
    user: &Account,
    owner: workspaces::AccountId,
) -> anyhow::Result<String> {
    let series: String = user
        .call(worker, nft, "nft_create_series")
        .args_json(serde_json::json!({
        "token_metadata":
        {
            "title": "some title",
            "media": "ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz",
            "copies": 10
        },
        "royalty":
        {
            owner.as_ref(): 1000
        }}))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?
        .json()?;
    Ok(series)
}

pub async fn deposit(
    worker: &Worker<impl DevNetwork>,
    market: workspaces::AccountId,
    user: &Account,
) {
    user.call(worker, market, "storage_deposit")
        .deposit(parse_near!("1 N"))
        .transact()
        .await
        .unwrap();
}

pub async fn nft_approve(
    worker: &Worker<impl DevNetwork>,
    nft: workspaces::AccountId,
    market: workspaces::AccountId,
    user: &Account,
    token: String,
    sale_conditions: HashMap<AccountId, U128>,
    series: String,
) {
    user.call(worker, nft.clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token,
            "account_id": market,
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions,
                token_type: Some(series),
                start: None,
                end: None,
                origins: None,
            })).to_string()
        }))
        .unwrap()
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await
        .unwrap();
}

pub async fn price_with_fees(
    worker: &Worker<impl DevNetwork>,
    market: &Contract,
    sale_conditions: HashMap<AccountId, U128>,
) -> anyhow::Result<U128> {
    let price: U128 = market
        .view(
            worker,
            "price_with_fees",
            serde_json::json!({
                "price": sale_conditions.get(&AccountId::new_unchecked("near".to_string())).unwrap(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    Ok(price)
}

pub async fn offer(
    worker: &Worker<impl DevNetwork>,
    nft: workspaces::AccountId,
    market: workspaces::AccountId,
    user: &Account,
    token: String,
    price: U128,
) {
    user.call(worker, market.clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft,
            "token_id": token,
            "ft_token_id": "near",
        }))
        .unwrap()
        .deposit(price.into())
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await
        .unwrap();
}

pub async fn create_series_raw(
    worker: &Worker<impl DevNetwork>,
    nft: workspaces::AccountId,
    owner: &Account,
    copies: Option<u64>,
    royalty: HashMap<&workspaces::AccountId, u64>,
) -> anyhow::Result<String> {
    let token_metadata = TokenMetadata {
        title: Some("some title".to_string()),
        description: None,
        media: Some("ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz".to_string()),
        media_hash: None,
        copies,
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    };
    Ok(owner
        .call(worker, nft, "nft_create_series")
        .args_json(serde_json::json!({
                "token_metadata": token_metadata,
                "royalty": royalty
        }))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?
        .json()?)
}

pub async fn offer_with_duration(
    worker: &Worker<impl DevNetwork>,
    nft: workspaces::AccountId,
    market: workspaces::AccountId,
    user: &Account,
    token: String,
    price: U128,
    duration: U64,
) {
    user.call(worker, market.clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft,
            "token_id": token,
            "ft_token_id": "near",
            "duration": duration
        }))
        .unwrap()
        .deposit(price.into())
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await
        .unwrap();
}

pub async fn nft_transfer_payout_helper(
    worker: &Worker<impl DevNetwork>,
    nft: &Contract,
    user1: &Account,
    user2: &Account,
    user3: &Account,
    royalty: HashMap<&workspaces::AccountId, u64>,
    fees: Fees,
    balance: U128,
) -> Payout {
    let series = create_series_raw(worker, nft.id().clone(), user1, Some(4), royalty)
        .await
        .unwrap();

    let token_id = mint_token(worker, nft.id().clone(), user1, user1.id(), &series)
        .await
        .unwrap();
    user1
        .call(worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token_id,
            "account_id": user2.id(),
        }))
        .unwrap()
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await
        .unwrap();
    let approval_id: u64 = {
        let token: Token = nft
            .view(
                worker,
                "nft_token",
                serde_json::json!({ "token_id": token_id })
                    .to_string()
                    .into_bytes(),
            )
            .await
            .unwrap()
            .json()
            .unwrap();
        let approval_account_ids = token.approved_account_ids.unwrap();
        *approval_account_ids
            .get(&user2.id().as_ref().parse().unwrap())
            .unwrap()
    };
    user2
        .call(worker, nft.id().clone(), "nft_transfer_payout")
        .args_json(serde_json::json!({
            "receiver_id": user3.id(),
            "token_id": token_id,
            "approval_id": approval_id,
            "memo": serde_json::json!(fees).to_string(),
            "balance": balance,
            "max_len_payout": 10,
        }))
        .unwrap()
        .deposit(1)
        .transact()
        .await
        .unwrap()
        .json()
        .unwrap()
}
