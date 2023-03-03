use near_sdk_sim::{UserAccount, ContractAccount, runtime::GenesisConfig, init_simulator, deploy, to_yocto};
use nft_contract::{NftContract, common::TokenMetadata};
use nft_bid_market::MarketContract;
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    MARKET_WASM_BYTES => "../res/nft_bid_market.wasm",
    NFT_WASM_BYTES => "../res/nft_contract.wasm",
}

const NFT_ID: &str = "nft";
const MARKET_ID: &str = "market";

pub fn init() -> (
    UserAccount,
    ContractAccount<MarketContract>,
    ContractAccount<NftContract>,
) {
    let g_config = GenesisConfig {
        block_prod_time: 1_000_000_000 * 60, // 1 mins/block
        ..Default::default()
    };
    let root = init_simulator(Some(g_config));

    let market = deploy!(
        contract: MarketContract,
        contract_id: MARKET_ID,
        bytes: &MARKET_WASM_BYTES,
        signer_account: root,
        init_method: new(vec![NFT_ID.parse().unwrap()], root.account_id())
    );

    let nft = deploy!(
        contract: NftContract,
        contract_id: NFT_ID,
        bytes: &NFT_WASM_BYTES,
        signer_account: root,
        deposit: to_yocto("200"),
        init_method: new_default_meta(root.account_id())
    );

    (root, market, nft)
}

pub fn prod_block(root: &UserAccount) {
    let mut runtime = root.borrow_runtime_mut();
    runtime.produce_block().unwrap();
}