#!/bin/bash
rm -rvf res

set -e
sh build-all.sh # building all

rm -f .env
rm -rvf neardev

near dev-deploy /dev/null
source neardev/dev-account.env

CONTRACT_PARENT=$CONTRACT_NAME
MARKET_CONTRACT_ID=market.$CONTRACT_PARENT
NFT_CONTRACT_ID=nft.$CONTRACT_PARENT
ALICE=alice.$CONTRACT_PARENT

echo "CONTRACT_PARENT=$CONTRACT_PARENT" > .env
echo "MARKET_CONTRACT_ID=$MARKET_CONTRACT_ID" >> .env
echo "NFT_CONTRACT_ID=$NFT_CONTRACT_ID" >> .env
echo "ALICE=$ALICE" >> .env

set -e
near create-account $NFT_CONTRACT_ID --masterAccount $CONTRACT_PARENT --initialBalance "27"
near create-account $ALICE --masterAccount $CONTRACT_PARENT --initialBalance 20
near create-account $MARKET_CONTRACT_ID --masterAccount $CONTRACT_PARENT --initialBalance 50

# Set up
near deploy $NFT_CONTRACT_ID --wasmFile res/nft_contract.wasm
near deploy $MARKET_CONTRACT_ID --wasmFile res/nft_bid_market.wasm
