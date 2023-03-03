use crate::auction::AuctionJson;
use crate::common::*;
use crate::*;

#[near_bindgen]
impl Market {
    pub fn get_current_buyer(&self, auction_id: U128) -> Option<AccountId> {
        let auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction does not exist"));
        if let Some(bid) = auction.bid {
            Some(bid.owner_id)
        } else {
            None
        }
    }

    pub fn check_auction_in_progress(&self, auction_id: U128) -> bool {
        let auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction does not exist"));
        auction.end >= env::block_timestamp() && auction.start < env::block_timestamp()
    }

    pub fn get_auction(&self, auction_id: U128) -> AuctionJson {
        let auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction does not exist"));
        self.json_from_auction(auction)
    }

    // Returns the minimum amount of the next auction bid (not including fees)
    pub fn get_minimal_next_bid(&self, auction_id: U128) -> U128 {
        let auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction does not exist"));
        let min_deposit = if let Some(ref bid) = auction.bid {
            let total_origins = fee::calculate_origins(&bid.origins);
            let actual_amount = fee::calculate_actual_amount(bid.price.0, total_origins); // TODO: need more tests here
            actual_amount + auction.minimal_step
        } else {
            auction.start_price
        };
        U128(min_deposit)
    }

    // Returns current bid amount (not including fees)
    pub fn get_current_bid(&self, auction_id: U128) -> Option<U128> {
        let auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction does not exist"));
        auction.bid.map(|bid| {
            {
                let total_origins = fee::calculate_origins(&bid.origins);
                let actual_amount = fee::calculate_actual_amount(bid.price.0, total_origins);
                actual_amount
            }
            .into()
        })
    }

    pub fn get_auctions(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<AuctionJson> {
        let auctions = &self.market.auctions;
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        auctions
            .values()
            .skip(start_index as usize)
            .take(limit)
            .map(|auction| self.json_from_auction(auction))
            .collect()
    }

    //pub fn get_bid_total_amount() -> U128;
}
