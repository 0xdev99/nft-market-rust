use crate::token_series::TokenSeriesJson;
use crate::*;

#[near_bindgen]
impl Nft {
    pub fn nft_get_series(&self, token_series_id: TokenSeriesId) -> TokenSeriesJson {
        let token_series = self.token_series_by_id.get(&token_series_id).expect("no series");
        TokenSeriesJson {
            metadata: token_series.metadata,
            owner_id: token_series.owner_id,
            royalty: token_series.royalty,
        }
    }

    pub fn nft_series(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<TokenSeriesJson> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.token_series_by_id.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");

        self.token_series_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(_token_series_id, token_series)| TokenSeriesJson {
                // token_series_id, do we need it?
                metadata: token_series.metadata,
                owner_id: token_series.owner_id,
                royalty: token_series.royalty,
            })
            .collect()
    }

    pub fn nft_supply_for_series(&self, token_series_id: TokenSeriesId) -> U128 {
        U128::from(
            self.token_series_by_id
                .get(&token_series_id)
                .unwrap_or_else(|| env::panic_str("Could not find token series"))
                .tokens
                .len() as u128,
        )
    }
}
