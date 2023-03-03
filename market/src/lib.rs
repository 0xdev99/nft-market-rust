mod auction;
mod auction_views;
mod bid;
mod common;
mod fee;
mod inner;
mod market_core;
mod sale;
mod sale_views;
mod token;

mod hack; // TODO: remove

use common::*;

use crate::sale::{Sale, SaleConditions, TokenType,
    ContractAndTokenId, FungibleTokenId};
use crate::auction::Auction;
pub use crate::sale::{SaleJson, BID_HISTORY_LENGTH_DEFAULT};
pub use crate::market_core::{ArgsKind, SaleArgs, AuctionArgs};
pub use crate::auction::{AuctionJson, EXTENSION_DURATION};
pub use crate::fee::{Fees, PAYOUT_TOTAL_VALUE, PROTOCOL_FEE};

const STORAGE_PER_SALE: u128 = 1000 * STORAGE_PRICE_PER_BYTE;

/// Helper structure to for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    Sales,
    ByOwnerId,
    ByOwnerIdInner { account_id_hash: CryptoHash },
    ByNFTContractId,
    ByNFTContractIdInner { account_id_hash: CryptoHash },
    ByNFTTokenType,
    ByNFTTokenTypeInner { token_type_hash: CryptoHash },
    FTTokenIds,
    StorageDeposits,
    OriginFees,
    Auctions,
    AuctionId,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MarketSales {
    pub owner_id: AccountId,
    pub sales: UnorderedMap<ContractAndTokenId, Sale>,
    pub by_owner_id: LookupMap<AccountId, UnorderedSet<ContractAndTokenId>>,
    pub by_nft_contract_id: LookupMap<AccountId, UnorderedSet<TokenId>>,
    pub by_nft_token_type: LookupMap<String, UnorderedSet<ContractAndTokenId>>,
    pub ft_token_ids: UnorderedSet<FungibleTokenId>,
    pub storage_deposits: LookupMap<AccountId, Balance>,
    pub bid_history_length: u8,

    pub auctions: UnorderedMap<u128, Auction>,
    pub next_auction_id: u128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Market {
    non_fungible_token_account_ids: LookupSet<AccountId>,
    market: MarketSales,
}

#[near_bindgen]
impl Market {
    #[init]
    pub fn new(nft_ids: Vec<AccountId>, owner_id: AccountId) -> Self {
        let mut non_fungible_token_account_ids = LookupSet::new(b"n");
        non_fungible_token_account_ids.extend(nft_ids);
        let mut tokens = UnorderedSet::new(StorageKey::FTTokenIds);
        tokens.insert(&AccountId::new_unchecked("near".to_owned()));
        let market = MarketSales {
            owner_id,
            sales: UnorderedMap::new(StorageKey::Sales),
            by_owner_id: LookupMap::new(StorageKey::ByOwnerId),
            by_nft_contract_id: LookupMap::new(StorageKey::ByNFTContractId),
            by_nft_token_type: LookupMap::new(StorageKey::ByNFTTokenType),
            ft_token_ids: tokens,
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits),
            bid_history_length: BID_HISTORY_LENGTH_DEFAULT,
            auctions: UnorderedMap::new(StorageKey::Auctions),
            next_auction_id: 0,
        };
        Self {
            non_fungible_token_account_ids,
            market,
        }
    }

    #[payable]
    pub fn storage_withdraw(&mut self) {
        assert_one_yocto();
        let owner_id = env::predecessor_account_id();
        let mut amount = self.market.storage_deposits.remove(&owner_id).unwrap_or(0);
        let sales = self.market.by_owner_id.get(&owner_id);
        let len = sales.map(|s| s.len()).unwrap_or_default();
        let diff = u128::from(len) * STORAGE_PER_SALE;
        amount -= diff;
        if amount > 0 {
            Promise::new(owner_id.clone()).transfer(amount);
        }
        if diff > 0 {
            self.market.storage_deposits.insert(&owner_id, &diff);
        }
    }

    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {
        let storage_account_id = account_id.unwrap_or_else(env::predecessor_account_id);
        let deposit = env::attached_deposit();
        assert!(
            deposit >= STORAGE_PER_SALE,
            "Requires minimum deposit of {}",
            STORAGE_PER_SALE
        );
        let mut balance: u128 = self
            .market
            .storage_deposits
            .get(&storage_account_id)
            .unwrap_or(0);
        balance += deposit;
        self.market
            .storage_deposits
            .insert(&storage_account_id, &balance);
    }

    pub fn storage_amount(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }
}
