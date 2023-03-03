#![allow(clippy::ref_in_deref)]
mod utils;
use std::collections::HashMap;

use near_contract_standards::non_fungible_token::Token;
use near_sdk_sim::{call, to_yocto, transaction::ExecutionStatus, view};
use nft_contract::{common::TokenMetadata, TokenSeriesJson};
use utils::init;

#[test]
fn nft_create_series_negative() {
    let (root, _, nft) = init();
    let user1 = root.create_user("user1".parse().unwrap(), to_yocto("1000"));
    let user2 = root.create_user("user2".parse().unwrap(), to_yocto("1000"));
    // Only authorized account can create series
    call!(root, nft.set_private_minting(true));
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
    let res = call!(
        user1,
        nft.nft_create_series(token_metadata.clone(), None),
        deposit = to_yocto("0.005")
    );
    if let ExecutionStatus::Failure(execution_error) =
        &res.promise_errors().remove(0).unwrap().outcome().status
    {
        assert!(execution_error
            .to_string()
            .contains("Access to mint is denied for this contract"));
    } else {
        panic!("Expected failure");
    }
    call!(root, nft.grant(user1.account_id()));

    // Title of the series should be specified
    let res = call!(
        user1,
        nft.nft_create_series(
            TokenMetadata {
                title: None,
                ..token_metadata.clone()
            },
            None
        ),
        deposit = to_yocto("0.005")
    );
    if let ExecutionStatus::Failure(execution_error) =
        &res.promise_errors().remove(0).unwrap().outcome().status
    {
        assert!(execution_error
            .to_string()
            .contains("title is missing from token metadata"));
    } else {
        panic!("Expected failure");
    }

    // Royalty can't exceed 50%
    let royalty = HashMap::from([(user1.account_id(), 500), (user2.account_id(), 5000)]);
    let res = call!(
        user1,
        nft.nft_create_series(token_metadata, Some(royalty)),
        deposit = to_yocto("0.005")
    );
    if let ExecutionStatus::Failure(execution_error) =
        &res.promise_errors().remove(0).unwrap().outcome().status
    {
        assert!(execution_error
            .to_string()
            .contains("maximum royalty cap exceeded"));
    } else {
        panic!("Expected failure");
    }
}

#[test]
fn nft_create_series_positive() {
    let (root, _, nft) = init();
    let user1 = root.create_user("user1".parse().unwrap(), to_yocto("1000"));
    let user2 = root.create_user("user2".parse().unwrap(), to_yocto("1000"));
    let royalty = HashMap::from([(user1.account_id(), 500), (user2.account_id(), 2000)]);

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
    call!(
        user1,
        nft.nft_create_series(token_metadata.clone(), None),
        deposit = to_yocto("0.005")
    )
    .assert_success();
    call!(root, nft.set_private_minting(true));
    // with private minting
    call!(root, nft.grant(user2.account_id()));
    let series_id: String = call!(
        user2,
        nft.nft_create_series(token_metadata.clone(), Some(royalty.clone())),
        deposit = to_yocto("1")
    )
    .unwrap_json();
    assert!(user2.account().unwrap().amount > to_yocto("999")); // make sure that deposit is refunded
    let series_json: TokenSeriesJson = view!(nft.nft_get_series(series_id)).unwrap_json();
    //assert_eq!(series_json.royalty, royalty);
    assert_eq!(
        series_json,
        TokenSeriesJson {
            metadata: token_metadata,
            owner_id: user2.account_id(),
            royalty,
        }
    )
}

#[test]
fn nft_mint_negative() {
    let (root, _, nft) = init();
    let user1 = root.create_user("user1".parse().unwrap(), to_yocto("1000"));
    let user2 = root.create_user("user2".parse().unwrap(), to_yocto("1000"));
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
    let series_id: String = call!(
        user1,
        nft.nft_create_series(token_metadata, None),
        deposit = to_yocto("0.005")
    )
    .unwrap_json();
    // Only authorized account can mint
    call!(root, nft.set_private_minting(true));
    let res = call!(
        user1,
        nft.nft_mint(series_id.clone(), user1.account_id(), None),
        deposit = to_yocto("2")
    );
    if let ExecutionStatus::Failure(execution_error) =
        &res.promise_errors().remove(0).unwrap().outcome().status
    {
        assert!(execution_error
            .to_string()
            .contains("Access to mint is denied for this contract"));
    } else {
        panic!("Expected failure");
    }
    call!(root, nft.set_private_minting(false));

    // wrong series_id
    let res = call!(
        user1,
        nft.nft_mint("200".to_string(), user1.account_id(), None),
        deposit = to_yocto("2")
    );
    if let ExecutionStatus::Failure(execution_error) =
        &res.promise_errors().remove(0).unwrap().outcome().status
    {
        assert!(execution_error
            .to_string()
            .contains("Token series does not exist"));
    } else {
        panic!("Expected failure");
    }

    // only owner allowed to mint this series
    let res = call!(
        user2,
        nft.nft_mint(series_id.clone(), user1.account_id(), None),
        deposit = to_yocto("2")
    );
    if let ExecutionStatus::Failure(execution_error) =
        &res.promise_errors().remove(0).unwrap().outcome().status
    {
        assert!(execution_error.to_string().contains("permission denied"));
    } else {
        panic!("Expected failure");
    }

    // Try to exceed max tokens
    call!(
        user1,
        nft.nft_mint(series_id.clone(), user1.account_id(), None),
        deposit = to_yocto("2")
    )
    .assert_success();
    let res = call!(
        user1,
        nft.nft_mint(series_id, user1.account_id(), None),
        deposit = to_yocto("2")
    );
    if let ExecutionStatus::Failure(execution_error) =
        &res.promise_errors().remove(0).unwrap().outcome().status
    {
        assert!(execution_error.to_string().contains("Max token minted"));
    } else {
        panic!("Expected failure");
    }
}

#[test]
fn nft_mint_positive() {
    let (root, _, nft) = init();
    let user1 = root.create_user("user1".parse().unwrap(), to_yocto("1000"));
    let user2 = root.create_user("user2".parse().unwrap(), to_yocto("1000"));
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
    let series_id: String = call!(
        user1,
        nft.nft_create_series(token_metadata.clone(), None),
        deposit = to_yocto("0.005")
    )
    .unwrap_json();
    let token_id: String = call!(
        user1,
        nft.nft_mint(series_id, user2.account_id(), None),
        deposit = to_yocto("2")
    )
    .unwrap_json();
    assert!(user1.account().unwrap().amount > to_yocto("999")); // refunded deposit (1000-2 should be 998, but we have 999+)
    let minted_token: Token = view!(nft.nft_token(token_id.clone())).unwrap_json();
    let minted_token_metadata = minted_token.metadata.as_ref().unwrap();
    token_metadata.issued_at = minted_token_metadata.issued_at.clone();
    token_metadata.copies = None;
    assert_eq!(
        minted_token,
        Token {
            token_id,
            owner_id: user2.account_id,
            metadata: Some(token_metadata),
            approved_account_ids: Some(Default::default())
        }
    );
}
