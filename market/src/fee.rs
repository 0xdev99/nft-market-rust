use crate::*;
use crate::{bid::Origins, common::*};
use std::collections::HashMap;

pub const PAYOUT_TOTAL_VALUE: u128 = 10_000;
pub const PROTOCOL_FEE: u128 = 300; // 10_000 is 100%, so 300 is 3%

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Fees {
    pub buyer: HashMap<AccountId, u32>,
    pub seller: HashMap<AccountId, u32>,
}

pub fn calculate_origins(origins: &Origins) -> u32 {
    let mut total: u32 = 0;
    for val in origins.values() {
        total += val;
    }
    total
}

pub fn calculate_actual_amount(amount: u128, total_origins: u32) -> u128 {
    let origin_fee = amount * (total_origins as u128 + PROTOCOL_FEE)
        / (PAYOUT_TOTAL_VALUE + total_origins as u128 + PROTOCOL_FEE);
    amount - origin_fee
}

pub fn calculate_price_with_fees(price: U128, origins: Option<&Origins>) -> u128 {
    let total_origins = if let Some(origins) = origins {
        calculate_origins(origins)
    } else {
        0
    };
    price.0 * (PAYOUT_TOTAL_VALUE + PROTOCOL_FEE + total_origins as u128) / PAYOUT_TOTAL_VALUE
}

#[near_bindgen]
impl Market {
    pub fn price_with_fees(&self, price: U128, origins: Option<Origins>) -> U128 {
        calculate_price_with_fees(price, origins.as_ref()).into()
    }
}

// pub fn with_fees(price: u128) -> u128 {
//     price * (PAYOUT_TOTAL_VALUE + PROTOCOL_FEE) / PAYOUT_TOTAL_VALUE
// }

// pub fn get_fee(price: u128) -> u128 {
//     price * PROTOCOL_FEE / PAYOUT_TOTAL_VALUE
// }

// pub fn calculate_origins(price: u128, origins: Origins) -> HashMap<AccountId, u128> {
//     let mut map = HashMap::with_capacity(origins.len());
//     for (origin, p) in origins {
//         map.insert(origin, price * p / PAYOUT_TOTAL_VALUE);
//     }
//     map
// }

// pub fn calculate_origin_fee(price: u128, origins: &Origins) -> u128 {
//     let mut total = 0;
//     for p in origins.values() {
//         total += p.0;
//     }
//     price * total / (PAYOUT_TOTAL_VALUE + total)
// }
// #[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
// //#[serde(crate = "near_sdk::serde")]
// pub struct Fees {
//     pub protocol_fee: u128,
//     pub origins: UnorderedMap<AccountId, u128>,
//     pub royalty: u128,
// }

// impl Fees {
//     //Should be called in add_bid to check that the buyer attached enough deposit to pay the price + fee.
//     pub fn total_amount_fee_side(&self, price: U128) -> U128 {
//         U128(price.0 + self.calculate_protocol_fee(price).0 + self.calculate_origin_fee(price).0)
//     }

//     pub fn calculate_protocol_fee(&self, price: U128) -> U128 {
//         U128(price.0 * self.protocol_fee / 10_000 as u128)
//     }

//     pub fn calculate_origin_fee(&self, price: U128) -> U128 {
//         //    let accounts_and_fees = self.origins.get(&token).unwrap();
//         //    let mut total_origin: u128 = 0;
//         //    for (_account, fee) in accounts_and_fees.iter() {
//         //        total_origin += fee;
//         //    }
//         //    U128(price.0*total_origin)

//         let mut total_origin: u128 = 0;
//         for (_account, fee) in self.origins.iter() {
//             total_origin += fee;
//         }

//         U128(price.0 * total_origin / 10_000 as u128)
//     }

//     pub fn calculate_royalty(&self, price: U128) -> U128 {
//         U128(price.0 * self.royalty / 10_000 as u128)
//     }
// }

// //Fee side here is the account which buys nft. It pays with NEAR (or FT?).
// //It pays protocol_fees and origins.
// //Non-fee side pays protocol_fees, origins and royalty.

// //doTransfersWithFees on the fee side
// //transferPayouts on the non-fee side
