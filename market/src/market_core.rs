use near_sdk::serde_json::json;
use crate::*;
use crate::bid::Origins;


pub trait NonFungibleTokenApprovalReceiver {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleArgs {
    pub sale_conditions: SaleConditions,
    pub token_type: TokenType,

    pub start: Option<U64>,
    pub end: Option<U64>,

    pub origins: Option<Origins>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AuctionArgs {
    pub token_type: TokenType,
    pub minimal_step: U128,
    pub start_price: U128,

    pub start: Option<U64>,
    pub duration: U64,
    pub buy_out_price: Option<U128>,

    pub origins: Option<Origins>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ArgsKind {
    Sale(SaleArgs),
    Auction(AuctionArgs),
}

#[near_bindgen]
impl NonFungibleTokenApprovalReceiver for Market {
    // nft_on_approve is called via cross-contract call in order to create a new sale or auction
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) {
        // make sure that the method is called in a cross contract call and the signer is owner_id

        let nft_contract_id = env::predecessor_account_id();
        let signer_id = env::signer_account_id();
        require!(
            nft_contract_id != signer_id,
            "nft_on_approve should only be called via cross-contract call"
        );
        require!(owner_id == signer_id, "owner_id should be signer_id");

        // check that the signer's storage is enough to cover one more sale

        let storage_amount = self.storage_amount().0;
        let owner_paid_storage = self.market.storage_deposits.get(&signer_id).unwrap_or(0);
        let signer_storage_required =
            (self.get_supply_by_owner_id(signer_id).0 + 1) as u128 * storage_amount;
        assert!(
            owner_paid_storage >= signer_storage_required,
            "Insufficient storage paid: {}, for {} sales at {} rate of per sale",
            owner_paid_storage,
            signer_storage_required / STORAGE_PER_SALE,
            STORAGE_PER_SALE
        );

        // Parse the msg to find Sale or Auction arguments

        let args: ArgsKind = near_sdk::serde_json::from_str(&msg).expect("Not valid args");
        match args {
            ArgsKind::Sale(sale_args) => {
                let sale_json = self.start_sale(
                    sale_args,
                    token_id,
                    owner_id,
                    approval_id,
                    nft_contract_id,
                );
                env::log_str(&near_sdk::serde_json::to_string(&sale_json).unwrap());
            }
            ArgsKind::Auction(auction_args) => {
                let (id, auction_json) = self.start_auction(
                    auction_args,
                    token_id,
                    owner_id,
                    approval_id,
                    nft_contract_id,
                );
                env::log_str(&json!({
                    "auction_id": U128(id),
                    "auction_json": auction_json
                }).to_string())
            }
        }
    }

    /*
    fn nft_on_series_approve(&mut self, token_series: TokenSeriesSale) {
        let nft_contract_id = env::predecessor_account_id();
        let signer_id = env::signer_account_id();
        require!(
            nft_contract_id != signer_id,
            "nft_on_approve should only be called via cross-contract call"
        );
        require!(
            token_series.owner_id == signer_id,
            "owner_id should be signer_id"
        );

        let storage_amount = self.storage_amount().0;
        let owner_paid_storage = self.market.storage_deposits.get(&signer_id).unwrap_or(0);
        let signer_storage_required =
            (self.get_supply_by_owner_id(signer_id).0 + 1) as u128 * storage_amount;
        assert!(
            owner_paid_storage >= signer_storage_required,
            "Insufficient storage paid: {}, for {} sales at {} rate of per sale",
            owner_paid_storage,
            signer_storage_required / STORAGE_PER_SALE,
            STORAGE_PER_SALE
        );

        for (ft_token_id, _price) in token_series.sale_conditions.clone() {
            if !self.market.ft_token_ids.contains(&ft_token_id) {
                env::panic_str(&format!(
                    "Token {} not supported by this market",
                    ft_token_id
                ));
            }
        }

        let contract_and_series_id =
            format!("{}{}{}", nft_contract_id, DELIMETER, token_series.series_id);

        // extra for views

        let mut by_owner_id = self
            .market
            .by_owner_id
            .get(&token_series.owner_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKey::ByOwnerIdInner {
                        account_id_hash: hash_account_id(&token_series.owner_id),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });

        let owner_occupied_storage = u128::from(by_owner_id.len()) * STORAGE_PER_SALE;
        require!(
            owner_paid_storage > owner_occupied_storage,
            "User has more sales than storage paid"
        );
        by_owner_id.insert(&contract_and_series_id);
        self.market
            .by_owner_id
            .insert(&token_series.owner_id, &by_owner_id);

        self.market.series_sales.insert(
            &contract_and_series_id,
            &SeriesSale {
                owner_id: token_series.owner_id,
                nft_contract_id: env::predecessor_account_id(),
                series_id: token_series.series_id,
                sale_conditions: token_series.sale_conditions,
                created_at: env::block_timestamp(),
                copies: token_series.copies,
            },
        );
    }
    */
}
