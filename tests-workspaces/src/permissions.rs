use workspaces::{Account, DevNetwork, Worker, Contract};
use nft_contract::common::AccountId;

use crate::utils::{init_nft, create_subaccount, 
    check_outcome_success, check_outcome_fail
};

pub async fn set_private_minting(
    worker: &Worker<impl DevNetwork>,
    nft: workspaces::AccountId,
    user: &Account,
    enabled: bool 
) {
    user.call(worker, nft, "set_private_minting")
        .args_json(serde_json::json!({
            "enabled": enabled,
        }))
        .unwrap()
        .transact()
        .await
        .unwrap();
}

pub async fn grant(
    worker: &Worker<impl DevNetwork>,
    nft: workspaces::AccountId,
    user: &Account,
    account_id: AccountId 
) -> anyhow::Result<bool> {
    let result = user.call(worker, nft, "grant")
        .args_json(serde_json::json!({
            "account_id": account_id,
        }))?
        .transact()
        .await?
        .json()?;
    Ok(result)
}

pub async fn is_allowed(
    worker: &Worker<impl DevNetwork>,
    nft: &Contract,
    account_id: AccountId 
) -> anyhow::Result<bool> {
    let result: bool = nft
        .view(
            worker,
            "is_allowed",
            serde_json::json!({
                "account_id": account_id,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    Ok(result)
}

/*
- Can only be called by the owner
- Adds a given account to the list of the autorized accounts
- Returns `true` if the new account has been added to the list, `false` otherwise
*/
#[tokio::test]
async fn permissions_grant() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    
    set_private_minting(&worker, nft.id().clone(), &owner, true).await;

    // Can only be called by the owner
    let outcome = user1.call(&worker, nft.id().clone(), "grant")
        .args_json(serde_json::json!({
            "account_id": AccountId::new_unchecked("user1".to_owned()),
        }))?
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "only owner can grant").await;
    
    // Adds a given account to the list of the autorized accounts
    let outcome = owner.call(&worker, nft.id().clone(), "grant")
        .args_json(serde_json::json!({
            "account_id": AccountId::new_unchecked("user1".to_owned()),
        }))?
        .transact()
        .await?;
    check_outcome_success(outcome.clone().status).await;

    // Returns `true` if the new account has been added to the list
    assert!(outcome.json()?, "Returned false");
    assert!(
        is_allowed(
            &worker,
            &nft,
            AccountId::new_unchecked("user1".to_owned())
        ).await?,
        "The user is not authorized"
    );

    // `user1` is already in the list, thus `false` is returned
    let outcome = owner.call(&worker, nft.id().clone(), "grant")
        .args_json(serde_json::json!({
            "account_id": AccountId::new_unchecked("user1".to_owned()),
        }))?
        .transact()
        .await?;
    check_outcome_success(outcome.clone().status).await;
    assert!(!outcome.json()?, "Returned true");

    Ok(())
}

/*
- Can only be called by the owner
- Removes a given account from the list of the autorized accounts
- Returns `true` if the account has been removed from the list, `false` if it hadn't been in the list
*/
#[tokio::test]
async fn permissions_deny() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    
    set_private_minting(&worker, nft.id().clone(), &owner, true).await;
    grant(
        &worker,
        nft.id().clone(),
        &owner,
        AccountId::new_unchecked("user1".to_owned())
    ).await?;

    // Can only be called by the owner
    let outcome = user1.call(&worker, nft.id().clone(), "deny")
        .args_json(serde_json::json!({
            "account_id": AccountId::new_unchecked("user1".to_owned()),
        }))?
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "only owner can deny").await;

    // Called by the owner
    let outcome = owner.call(&worker, nft.id().clone(), "deny")
        .args_json(serde_json::json!({
            "account_id": AccountId::new_unchecked("user1".to_owned()),
        }))?
        .transact()
        .await?;
    check_outcome_success(outcome.clone().status).await;

    // Returns `true` if the account has been removed from the list
    assert!(outcome.json()?, "Returned false");
    let result = is_allowed(
        &worker,
        &nft,
        AccountId::new_unchecked("user1".to_owned())
    ).await?;
    assert!(!result, "Not authorized");

    // `user1` is not in the list, thus `false` is returned
    let outcome = owner.call(&worker, nft.id().clone(), "deny")
        .args_json(serde_json::json!({
            "account_id": AccountId::new_unchecked("user1".to_owned()),
        }))?
        .transact()
        .await?;
    check_outcome_success(outcome.clone().status).await;
    assert!(!outcome.json()?, "Returned true");

    Ok(())
}

/*
- Can only be called by the owner
- If `enabled` is true, turns on private minting
- If `enabled` is false, turns off private minting
*/
#[tokio::test]
async fn permissions_set_private_minting() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;

    // Can only be called by the owner
    let outcome = user1.call(&worker, nft.id().clone(), "set_private_minting")
        .args_json(serde_json::json!({
            "enabled": true,
        }))?
        .transact()
        .await?;
    check_outcome_fail(
        outcome.status,
        "only owner can enable/disable private minting"
    ).await;
    assert!(
        is_allowed(
            &worker,
            &nft,
            AccountId::new_unchecked("user1".to_owned())
        ).await?,
        "The authorization is turned on"
    );

    // If `enabled` is true, turns on private minting
    let outcome = owner.call(&worker, nft.id().clone(), "set_private_minting")
        .args_json(serde_json::json!({
            "enabled": true,
        }))?
        .transact()
        .await?;
    check_outcome_success(outcome.status).await;
    assert!(
        !is_allowed(
            &worker,
            &nft,
            AccountId::new_unchecked("user1".to_owned())
        ).await?,
        "The authorization is turned off"
    );

    // If `enabled` is false, turns off private minting
    let outcome = owner.call(&worker, nft.id().clone(), "set_private_minting")
        .args_json(serde_json::json!({
            "enabled": false,
        }))?
        .transact()
        .await?;
    check_outcome_success(outcome.status).await;
    assert!(
        is_allowed(
            &worker,
            &nft,
            AccountId::new_unchecked("user1".to_owned())
        ).await?,
        "The authorization is turned on"
    );
    
    Ok(())
}

/*
- Returns true if private minting is not enabled
- If private minting is enabled, returns whether an account is among private minters
*/
#[tokio::test]
async fn permissions_is_allowed() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    
    // Returns true if private minting is not enabled
    let result = is_allowed(
        &worker,
        &nft,
        AccountId::new_unchecked("user1".to_owned())
    ).await?;
    assert!(result, "Not authorized");

    // If private minting is enabled and `user1` not authorized, returns false 
    set_private_minting(&worker, nft.id().clone(), &owner, true).await;
    let result = is_allowed(
        &worker,
        &nft,
        AccountId::new_unchecked("user1".to_owned())
    ).await?;
    assert!(!result, "Not authorized");

    // If private minting is enabled and `user1` authorized, returns true 
    grant(
        &worker,
        nft.id().clone(),
        &owner, AccountId::new_unchecked("user1".to_owned())
    ).await?;
    let result = is_allowed(
        &worker,
        &nft,
        AccountId::new_unchecked("user1".to_owned())
    ).await?;
    assert!(result, "Authorized");

    Ok(())
}
