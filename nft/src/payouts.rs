use std::collections::HashMap;

use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_sdk::assert_one_yocto;
use near_sdk::json_types::U128;

use crate::*;

pub const ROYALTY_TOTAL_VALUE: u128 = 10_000;
pub const MAXIMUM_ROYALTY: u32 = 5_000;
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Fees {
    pub buyer: HashMap<AccountId, u32>,
    pub seller: HashMap<AccountId, u32>,
}

pub trait Payouts {
    /// Given a `token_id` and NEAR-denominated balance, return the `Payout`.
    /// struct for the given token. Panic if the length of the payout exceeds
    /// `max_len_payout.`
    fn nft_payout(&self, token_id: String, balance: U128, max_len_payout: u32) -> Payout;
    /// Given a `token_id` and NEAR-denominated balance, transfer the token
    /// and return the `Payout` struct for the given token. Panic if the
    /// length of the payout exceeds `max_len_payout.`
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: u64,
        memo: Option<String>,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout;
}

#[near_bindgen]
impl Payouts for Nft {
    // Payout mapping for the given token, based on 'balance' and royalty
    fn nft_payout(&self, token_id: String, balance: U128, max_len_payout: u32) -> Payout {
        let token_owner = self.tokens.owner_by_id.get(&token_id).expect("no token id");

        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_series_id = token_id_iter.next().unwrap().to_owned();
        let royalty = self
            .token_series_by_id
            .get(&token_series_id)
            .expect("no type")
            .royalty;
        require!(royalty.len() as u32 <= max_len_payout, "Too many recievers");
        let mut total_payout = 0;
        let balance = Balance::from(balance);
        let mut payout: Payout = Payout {
            payout: HashMap::new(),
        };
        for (k, v) in royalty.iter() {
            if *k != token_owner {
                payout
                    .payout
                    .insert(k.clone(), royalty_to_payout(*v, balance));
                total_payout += v;
            }
        }
        require!(
            total_payout <= ROYALTY_TOTAL_VALUE as u32,
            "Royalty total value should be < 10000"
        );
        payout.payout.insert(
            token_owner,
            royalty_to_payout(ROYALTY_TOTAL_VALUE as u32 - total_payout, balance),
        );
        payout
    }

    // nft_transfer with 'balance' for calculation of Payout mapping for the given token
    // extra: lazy minting? for series
    #[payable]
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: u64,
        memo: Option<String>,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout {
        assert_one_yocto();

        let token_owner = self.tokens.owner_by_id.get(&token_id).expect("no token id");

        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_series_id = token_id_iter.next().unwrap().to_owned();
        let royalty = self
            .token_series_by_id
            .get(&token_series_id)
            .expect("no type")
            .royalty;
        require!(royalty.len() as u32 <= max_len_payout, "Too many recievers");

        let mut total_payout = 0;
        let mut max_payout = ROYALTY_TOTAL_VALUE as u32;
        let balance = Balance::from(balance);
        let mut initial_price = balance;
        let mut payout: Payout = Payout {
            payout: HashMap::new(),
        };
        if let Some(fees) = memo {
            let Fees { buyer, seller } =
                near_sdk::serde_json::from_str(&fees).expect("invalid FeesArgs");
            let mut buyer_value = 0;
            for v in buyer.values() {
                buyer_value += v;
            }
            max_payout += buyer_value;
            let buyer_fee =
                balance * buyer_value as u128 / (ROYALTY_TOTAL_VALUE + buyer_value as u128);
            initial_price = balance - buyer_fee;

            for (k, v) in buyer.iter() {
                if *k != token_owner {
                    payout
                        .payout
                        .insert(k.clone(), royalty_to_payout(*v, initial_price));
                    total_payout += v;
                }
            }
            for (k, v) in seller.iter() {
                if *k != token_owner {
                    let val = payout.payout.entry(k.clone()).or_insert(U128(0)); // Protocol fee would repeat and origins can repeat
                    (*val).0 += royalty_to_payout(*v, initial_price).0;
                    total_payout += v;
                }
            }
        }
        for (k, v) in royalty.iter() {
            if *k != token_owner {
                let val = payout.payout.entry(k.clone()).or_insert(U128(0));
                (*val).0 += royalty_to_payout(*v, initial_price).0;
                total_payout += v;
            }
        }
        require!(
            total_payout <= max_payout as u32,
            format!("Royalty total value should be < {}", max_payout)
        );
        payout.payout.insert(
            token_owner,
            royalty_to_payout(max_payout - total_payout, initial_price),
        );
        require!(
            payout.payout.len() as u32 <= max_len_payout,
            "Too many recievers"
        );
        self.nft_transfer(receiver_id, token_id, Some(approval_id), None);
        payout
    }
}

fn royalty_to_payout(a: u32, b: Balance) -> U128 {
    U128(a as u128 * b / ROYALTY_TOTAL_VALUE)
}
