use crate::*;

#[near_bindgen]
impl Market {

    pub fn hack_finish_sale(&mut self, nft_contract_token: ContractAndTokenId) {
        let mut sale = self.market.sales.get(&nft_contract_token).expect("no sale");
        sale.end = Some(env::block_timestamp());
        self.market.sales.insert(&nft_contract_token, &sale);
    }

    pub fn hack_finish_bid(&mut self, nft_contract_token: ContractAndTokenId) {
        let mut sale = self.market.sales.get(&nft_contract_token).expect("no sale");
        let bid = sale
            .bids
            .get_mut(&("near".parse().unwrap()))
            .expect("no bids")
            .last_mut();
        if let Some(bid) = bid {
            bid.end = Some(U64(env::block_timestamp()))
        }
        self.market.sales.insert(&nft_contract_token, &sale);
    }

    pub fn hack_finish_auction(&mut self, auction_id: U128) {
        let mut auction = self.market.auctions.get(&auction_id.into()).expect("no auction");
        auction.end = env::block_timestamp();
        self.market.auctions.insert(&auction_id.into(), &auction);
    }
}
