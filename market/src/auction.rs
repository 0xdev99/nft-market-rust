use std::collections::HashMap;

use crate::bid::{Bid, Origins};
use crate::fee::calculate_price_with_fees;
use crate::market_core::AuctionArgs;
use crate::sale::{
    ext_contract, ext_self, Payout, GAS_FOR_FT_TRANSFER, GAS_FOR_NFT_TRANSFER, GAS_FOR_ROYALTIES,
    NO_DEPOSIT,
};
use crate::*;
use near_sdk::{near_bindgen, promise_result_as_success};
// should check calculation
pub const EXTENSION_DURATION: u64 = 15 * 60 * NANOS_PER_SEC; // 15 minutes
pub const MAX_DURATION: u64 = 1000 * 60 * 60 * 24 * NANOS_PER_SEC; // 1000 days

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Auction {
    pub owner_id: AccountId,
    pub approval_id: u64,
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub bid: Option<Bid>,
    pub created_at: u64,
    pub ft_token_id: AccountId,
    pub minimal_step: u128,
    pub start_price: u128,
    pub buy_out_price: Option<u128>,

    pub start: u64,
    pub end: u64,

    pub origins: Origins,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AuctionJson {
    pub owner_id: AccountId,
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub bid: Option<Bid>,
    pub created_at: U64,
    pub ft_token_id: AccountId,
    pub minimal_step: U128,
    pub start_price: U128,
    pub buy_out_price: Option<U128>,

    pub start: U64,
    pub end: U64,
}

#[near_bindgen]
impl Market {
    // Called in nft_on_approve to create a new auction
    // Returns a pair of the auction_id and the auction itself
    pub(crate) fn start_auction(
        &mut self,
        args: AuctionArgs,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        nft_contract_id: AccountId,
    ) -> (u128, AuctionJson) {
        require!(
            args.duration.0 >= EXTENSION_DURATION && args.duration.0 <= MAX_DURATION,
            format!(
                "Incorrect duration. Should be at least {}",
                EXTENSION_DURATION
            )
        );
        let ft_token_id = self.token_type_to_ft_token_type(args.token_type);
        let start = args
            .start
            .map(|s| s.into())
            .unwrap_or_else(env::block_timestamp);
        require!(start >= env::block_timestamp(), "incorrect start time");
        let end = start + args.duration.0;
        let auction_id = self.market.next_auction_id;
        let origins = args.origins.unwrap_or_default();
        let auction = Auction {
            owner_id,
            approval_id,
            nft_contract_id,
            token_id,
            bid: None,
            created_at: env::block_timestamp(),
            ft_token_id,
            minimal_step: args.minimal_step.into(),
            start_price: args.start_price.into(),
            buy_out_price: args.buy_out_price.map(|p| p.into()),
            start,
            end,
            origins,
        };
        self.market.auctions.insert(&auction_id, &auction);
        self.market.next_auction_id += 1;

        let auction_json = self.json_from_auction(auction);

        // env::log_str(&near_sdk::serde_json::to_string(&(auction_id, auction)).unwrap());
        (auction_id, auction_json)
    }

    // Adds a bid to the corresponding auction
    // Supports buyout and time extension
    #[payable]
    pub fn auction_add_bid(
        &mut self,
        auction_id: U128,
        token_type: TokenType,
        origins: Option<Origins>,
    ) {
        let ft_token_id = self.token_type_to_ft_token_type(token_type);
        require!(
            self.market.ft_token_ids.contains(&ft_token_id),
            "token not supported"
        );
        require!(
            self.check_auction_in_progress(auction_id),
            "Auction is not in progress"
        );
        let mut auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("auction not active"));
        require!(
            auction.owner_id != env::predecessor_account_id(),
            "Cannot bid on your own auction"
        );
            let deposit = env::attached_deposit();
        let min_deposit =
            calculate_price_with_fees(self.get_minimal_next_bid(auction_id), origins.as_ref());

        // Check that the bid is not smaller than the minimal allowed bid
        require!(
            deposit >= min_deposit,
            format!("Should bid at least {}", min_deposit)
        );
        //Return previous bid
        if let Some(previous_bid) = auction.bid {
            self.refund_bid(ft_token_id, previous_bid.owner_id, previous_bid.price);
        }
        // If the price is bigger than the buy_out_price, the auction end is set to the current time
        let mut bought_out = false;
        if let Some(buy_out_price) = auction.buy_out_price {
            if calculate_price_with_fees(buy_out_price.into(), origins.as_ref()) <= deposit {
                auction.end = env::block_timestamp();
                bought_out = true;
            }
        }
        // Create a bid
        let bid = Bid {
            owner_id: env::predecessor_account_id(),
            price: deposit.into(),
            start: env::block_timestamp().into(),
            end: None,
            origins: origins.unwrap_or_default(),
        };
        // Extend the auction if the bid is added EXTENSION_DURATION (15 min) before the auction end
        // and the token is not bought out
        auction.bid = Some(bid);
        if auction.end - env::block_timestamp() < EXTENSION_DURATION && !bought_out {
            auction.end = env::block_timestamp() + EXTENSION_DURATION;
        }
        self.market.auctions.insert(&auction_id.into(), &auction);
    }

    // Cancels the auction if it doesn't have a bid yet
    // Can be called by the auction owner
    #[payable]
    pub fn cancel_auction(&mut self, auction_id: U128) {
        assert_one_yocto();
        let auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction is not active"));
        require!(
            auction.owner_id == env::predecessor_account_id(),
            "Only the auction owner can cancel the auction"
        );
        require!(
            auction.bid.is_none(),
            "Can't cancel the auction after the first bid is made"
        );
        self.market.auctions.remove(&auction_id.into());
    }

    // Finishes the auction if it has reached its end
    // Can be called by anyone
    pub fn finish_auction(&mut self, auction_id: U128) -> Promise {
        let auction = self
            .market
            .auctions
            .remove(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction is not active"));
        require!(
            env::block_timestamp() > auction.end,
            "Auction can be finalized only after the end time"
        );
        let final_bid = auction
            .bid
            .unwrap_or_else(|| env::panic_str("Can finalize only if there is a bid"));
        let mut buyer = final_bid.origins;
        buyer.insert(env::current_account_id(), PROTOCOL_FEE as u32);
        let mut seller_fee = HashMap::with_capacity(auction.origins.len() + 1);
        seller_fee.extend(auction.origins.clone()); // TODO: dodge this clone
        seller_fee.insert(env::current_account_id(), PROTOCOL_FEE as u32);
        let fees = fee::Fees {
            buyer,
            seller: seller_fee,
        };
        ext_contract::nft_transfer_payout(
            final_bid.owner_id.clone(),
            auction.token_id.clone(),
            auction.approval_id,
            Some(near_sdk::serde_json::to_string(&fees).expect("Failed to sereailize")),
            final_bid.price,
            10,
            auction.nft_contract_id.clone(),
            1,
            GAS_FOR_NFT_TRANSFER,
        )
        .then(ext_self::resolve_finish_auction(
            auction.ft_token_id,
            final_bid.owner_id.clone(),
            final_bid.price,
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_ROYALTIES,
        ))
    }

    // self callback
    // If transfer of token succeded - count fees and transfer payouts
    // If failed - refund price to buyer
    #[private]
    pub fn resolve_finish_auction(
        &mut self,
        ft_token_id: AccountId,
        buyer_id: AccountId,
        price: U128,
    ) -> U128 {
        let payout_option = promise_result_as_success().and_then(|value| {
            near_sdk::serde_json::from_slice::<Payout>(&value)
                .ok()
                .and_then(|payout| {
                    if payout.payout.len() > 10 || payout.payout.is_empty() {
                        env::log_str("Cannot have more than 10 payouts and sale.bids refunds");
                        None
                    } else {
                        let mut remainder = price.0;
                        for &value in payout.payout.values() {
                            remainder = remainder.checked_sub(value.0)?;
                        }
                        if remainder <= 1 {
                            Some(payout)
                        } else {
                            None
                        }
                    }
                })
        });
        // is payout option valid?
        let payout = if let Some(payout_option) = payout_option {
            payout_option
        } else {
            if ft_token_id == "near".parse().unwrap() {
                Promise::new(buyer_id).transfer(u128::from(price));
            }
            // leave function and return all FTs in ft_resolve_transfer
            return price;
        };

        // NEAR payouts
        if ft_token_id == "near".parse().unwrap() {
            for (receiver_id, amount) in payout.payout {
                Promise::new(receiver_id).transfer(amount.0);
            }
            // refund all FTs (won't be any)
            price
        } else {
            // FT payouts
            for (receiver_id, amount) in payout.payout {
                ext_contract::ft_transfer(
                    receiver_id,
                    amount,
                    None,
                    ft_token_id.clone(),
                    1,
                    GAS_FOR_FT_TRANSFER,
                );
            }
            // keep all FTs (already transferred for payouts)
            U128(0)
        }
    }

    fn token_type_to_ft_token_type(&self, token_type: TokenType) -> AccountId {
        let token_type = if let Some(token_type) = token_type {
            AccountId::new_unchecked(token_type)
        } else {
            AccountId::new_unchecked("near".to_owned())
        };
        require!(
            self.market.ft_token_ids.contains(&token_type),
            "token not supported"
        );
        token_type
    }

    pub(crate) fn json_from_auction(&self, auction: Auction) -> AuctionJson {
        AuctionJson {
            owner_id: auction.owner_id,
            nft_contract_id: auction.nft_contract_id,
            token_id: auction.token_id,
            bid: auction.bid,
            created_at: auction.created_at.into(),
            ft_token_id: auction.ft_token_id,
            minimal_step: auction.minimal_step.into(),
            start_price: auction.start_price.into(),
            buy_out_price: auction.buy_out_price.map(|p| p.into()),
            start: auction.start.into(),
            end: auction.end.into(),
        }
    }
}
