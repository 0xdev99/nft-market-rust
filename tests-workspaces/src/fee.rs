use std::collections::HashMap;

use near_units::parse_near;
use nft_bid_market::{PAYOUT_TOTAL_VALUE, PROTOCOL_FEE};
use nft_contract::common::{AccountId, U128};

use crate::utils::init_market;

#[tokio::test]
async fn price_with_fees() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let market = init_market(&worker, worker.root_account().id(), vec![]).await?;
    let price_without_fees = U128(23456788765);
    let price_with_fees: U128 = market
        .view(
            &worker,
            "price_with_fees",
            serde_json::json!({ "price": price_without_fees })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(
        price_with_fees,
        U128(price_without_fees.0 * (PAYOUT_TOTAL_VALUE + PROTOCOL_FEE) / PAYOUT_TOTAL_VALUE)
    );

    let origins: HashMap<AccountId, u32> = HashMap::from([
        ("user1".parse().unwrap(), 100),
        ("user2".parse().unwrap(), 200),
        ("user3".parse().unwrap(), 300),
        ("user4".parse().unwrap(), 400),
        ("user5".parse().unwrap(), 500),
        ("user6".parse().unwrap(), 600),
    ]);
    let origins_sum: u32 = origins.values().sum();
    let price_without_fees = U128(parse_near!("5 N"));
    let price_with_fees: U128 = market
        .view(
            &worker,
            "price_with_fees",
            serde_json::json!({
                "price": price_without_fees,
                "origins": origins
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(
        price_with_fees,
        U128(
            price_without_fees.0 * (PAYOUT_TOTAL_VALUE + PROTOCOL_FEE + origins_sum as u128)
                / PAYOUT_TOTAL_VALUE
        )
    );
    Ok(())
}
