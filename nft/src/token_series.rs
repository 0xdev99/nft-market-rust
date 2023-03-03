use near_contract_standards::non_fungible_token::metadata::TokenMetadata;

use crate::common::*;

use std::collections::HashMap;

use crate::token::TokenId;

pub type TokenSeriesId = String;
pub const TOKEN_DELIMETER: char = ':';

// note, keep it all pub for now, but later switch to all private fields.

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenSeries {
    pub metadata: TokenMetadata,
    pub owner_id: AccountId,
    pub tokens: UnorderedSet<TokenId>,
    pub royalty: HashMap<AccountId, u32>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct TokenSeriesJson {
    pub metadata: TokenMetadata,
    pub owner_id: AccountId,
    pub royalty: HashMap<AccountId, u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SeriesMintArgs {
    pub token_series_id: TokenSeriesId,
    pub receiver_id: AccountId,
}

pub type SaleConditions = HashMap<AccountId, U128>;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenSeriesSale {
    pub sale_conditions: SaleConditions,
    pub series_id: TokenSeriesId,
    pub owner_id: AccountId,
    pub copies: u64,
}
