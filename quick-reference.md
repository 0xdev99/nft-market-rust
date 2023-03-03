# Setup

## Installing the Rust toolchain

Install Rustup, configure your current shell and add `wasm` target to your toolchain by running:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup target add wasm32-unknown-unknown
```

## Installing the `near-cli`

Make sure you have the latest version of `npm` and `NodeJS` installed.

Install near-cli globally by running:
```bash
npm install -g near-cli
```

# NFT bid market

NFT bid market consists of two contracts: _NFT_ and _Market_.

_NFT contract_ allows to create and manage a token or token series. 
It supports Metadata, Approval Management and Royalties [standards](https://nomicon.io/Standards/NonFungibleToken/README.html).

_Market contract_ handles sales, bids and auctions.

To build both contracts and deploy it on dev account:
```bash
sh deploy-testnet.sh
source .env
```

Now we have `CONTRACT_PARENT` and three subaccounts: `MARKET_CONTRACT_ID`, `NFT_CONTRACT_ID` and `ALICE` ready to go.

Initialize contracts:
```bash
near call $NFT_CONTRACT_ID new_default_meta '{"owner_id": "'$CONTRACT_PARENT'"}' --accountId $NFT_CONTRACT_ID
near call $MARKET_CONTRACT_ID new '{"nft_ids": ["'$NFT_CONTRACT_ID'"], "owner_id": "'$CONTRACT_PARENT'"}' --accountId $MARKET_CONTRACT_ID
```

## NFT contract

_NFT contract_ supports [standards](https://nomicon.io/Standards/NonFungibleToken/README.html) for Metadata, Approval Management and Royalties. It also manages private minting.

Suppose `CONTRACT_PARENT` wants to sell a series of NFTs.
The first step is to create the series and mint several NFTs:
```bash
near call $NFT_CONTRACT_ID nft_create_series '{"token_metadata": {"title": "some title", "media": "ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz", "copies": 10}, "royalty": {"'$CONTRACT_PARENT'": 500}}' --accountId $CONTRACT_PARENT --deposit 0.005

near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "receiver_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "receiver_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "receiver_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "receiver_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "receiver_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "receiver_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "receiver_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "receiver_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
```
Now he has eight NFTs.
<!--
Instead of minting NFTs by himself, `CONTRACT_PARENT` can cover the storage for NFTs and give the market an approval to mint tokens.
After this `MARKET_CONTRACT_ID` will be able to mint a new NFT.
```bash
near call $MARKET_CONTRACT_ID storage_deposit --accountId $CONTRACT_PARENT --deposit 0.01

near call $NFT_CONTRACT_ID nft_series_market_approve '{"token_series_id": "1", "sale_conditions": {"near": "1200"}, "copies": 1, "approved_market_id": "'$MARKET_CONTRACT_ID'"}' --accountId $CONTRACT_PARENT --deposit 1

near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "receiver_id": "'$CONTRACT_PARENT'"}' --accountId $MARKET_CONTRACT_ID --deposit 1

near view $NFT_CONTRACT_ID nft_token '{"token_id": "1:9"}'
```
-->
### List of view methods for nft token series

The contract supports methods for Metadata, Approval Management and Royalties according to the [standards](https://nomicon.io/Standards/NonFungibleToken/README.html). Below we list only additional methods.

To get metadata, owner_id and royalty of the series:
```bash
near view $NFT_CONTRACT_ID nft_get_series '{"token_series_id": "1"}'
```

To get the number of NFTs which have alredy been minted from the series:
```bash
near view $NFT_CONTRACT_ID nft_supply_for_series '{"token_series_id": "1"}'
```

To get a list of all series (with pagination or without it):
```bash
near view $NFT_CONTRACT_ID nft_series '{"from_index": "0", "limit": 10}'
near view $NFT_CONTRACT_ID nft_series
```

## Market contract

Using _Market contract_ a user can put his NFT on a sale or an auction.
He specifies the conditions on which he wants to sell NFT, such as FT type and price, start and end (or duration for auction), origins.
Other users create bids, offering to buy (or buying) the NFT. Bids for sales can have start/end time.

### Workflow for creating and using sales

Before creating a sale the user needs to cover the storage (0.01 per one sale):
```bash
near call $MARKET_CONTRACT_ID storage_deposit --accountId $CONTRACT_PARENT --deposit 0.1
```

`CONTRACT_PARENT` puts three NFTs on sale using [approval management](https://nomicon.io/Standards/NonFungibleToken/ApprovalManagement.html):
```bash
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:1", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": {\"'$NFT_CONTRACT_ID'\": 100}} }"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:2", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:3", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:4", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:5", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": \"3153600000000000000\", \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1

near view $MARKET_CONTRACT_ID get_sales
```

`CONTRACT_PARENT` could have set the specific start time, since he hadn't done it, the auction started as soon as the command was complete.
Only the last sale has end time.
Only the first sale has origin fee. It will be paid by `CONTRACT_PARENT` to `NFT_CONTRACT_ID` after the NFT is sold. The number `100` in the method corresponds to 1% origin fee.
`CONTRACT_PARENT` specified the price to be `10000` yoctoNEAR for each token. It doesn't include protocol and origin fees. To see the full price you can call `price_with_fees`:
```bash
near view $MARKET_CONTRACT_ID price_with_fees '{"price": "10000", "origins": null}'
```
Here `price` is the amount you want to pay and `origins` you want to add to your bid.

Seller can withdraw the unused storage deposit:
```bash
near call $MARKET_CONTRACT_ID storage_withdraw --accountId $CONTRACT_PARENT --depositYocto 1
```

Any other account (in our case it is `ALICE`) can buy or offer to buy any of these NFTs. 
The difference is in the deposit which she attaches to `offer`. 
If `ALICE` calls `offer` to buy the first NFT and the attached deposit is equal to the price (`10300` including protocol fee), she automatically buys it.
If `ALICE` calls `offer` on the second NFT, but attaches less deposit than the price, she will only offer to buy the token.
`ALICE` gets the second NFT only after `CONTRACT_PARENT` accepts the offer using `accept_offer`.
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:1", "ft_token_id": "near"}' --accountId $ALICE --depositYocto 10300 --gas 200000000000000
near view $NFT_CONTRACT_ID nft_token '{"token_id": "1:1"}'

near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:2", "ft_token_id": "near"}' --accountId $ALICE --depositYocto 10200 --gas 200000000000000
near view $NFT_CONTRACT_ID nft_token '{"token_id": "1:2"}'
near call $MARKET_CONTRACT_ID accept_offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:2", "ft_token_id": "near"}' --accountId $CONTRACT_PARENT --gas 200000000000000
near view $NFT_CONTRACT_ID nft_token '{"token_id": "1:2"}'
```

`ALICE` can attach an origin fee to her offer:
```bash
near view $MARKET_CONTRACT_ID price_with_fees '{"price": "10000", "origins": {"'$NFT_CONTRACT_ID'": 150}}'
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3", "ft_token_id": "near", "origins": {"'$NFT_CONTRACT_ID'": 150}}' --accountId $ALICE --depositYocto 10450 --gas 200000000000000
near view $NFT_CONTRACT_ID nft_token '{"token_id": "1:3"}'
```
Here the final price is `10450` due to 3% protocol fee and 1.5% origin fee.
Origin fee is paid by `ALICE` to `$NFT_CONTRACT_ID` when the purchase is made.

If `CONTRACT_PARENT` wants to increase or decrease the price of the third NFT, he can call `update_price`.
```bash
near call $MARKET_CONTRACT_ID update_price '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4", "ft_token_id": "near", "price": "12000"}' --accountId $CONTRACT_PARENT --depositYocto 1

near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4"}'
```

Bids for sales can be deleted. If `ALICE` adds a bid and then decides to remove it, she could call `remove_bid`. This would remove her bid and return her money, even before the bid ends:
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4", "ft_token_id": "near"}' --accountId $ALICE --depositYocto 10000 --gas 200000000000000
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4"}'

near call $MARKET_CONTRACT_ID remove_bid '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4", "ft_token_id": "near", "price": "10000"}' --accountId $ALICE --depositYocto 1
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4"}'
```

Suppose some purchasers had added some bids and later they expired.
After this anyone can refund them:
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4", "ft_token_id": "near", "start": null, "duration": "100000000"}' --accountId $ALICE --depositYocto 500 --gas 200000000000000
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4", "ft_token_id": "near", "start": null, "duration": "100000000"}' --accountId $ALICE --depositYocto 600 --gas 200000000000000
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4", "ft_token_id": "near", "start": null, "duration": "100000000"}' --accountId $ALICE --depositYocto 800 --gas 200000000000000
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4"}'

near call $MARKET_CONTRACT_ID cancel_expired_bids '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4", "ft_token_id": "near"}' --accountId $NFT_CONTRACT_ID
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4"}'
```
For this example we created bids with duration equal to 0.1 second and canceled them.

It is possible to refund a specific bid (if it is ended):
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4", "ft_token_id": "near", "start": null, "duration": "100000000"}' --accountId $ALICE --depositYocto 700 --gas 200000000000000
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4"}'

near call $MARKET_CONTRACT_ID cancel_bid '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4", "ft_token_id": "near", "owner_id": "'$ALICE'", "price": "700"}' --accountId $NFT_CONTRACT_ID
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4"}'
```

`CONTRACT_PARENT` can call `remove_sale` to remove his sale and refund all the bids:
```bash
near call $MARKET_CONTRACT_ID remove_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:4"}' --accountId $CONTRACT_PARENT --depositYocto 1

near view $MARKET_CONTRACT_ID get_sales
```
When the sale is in progress, only `CONTRACT_PARENT` can call it. 
If the sale ends and no bid is accepted, anyone can call `remove_sale`.

If the sale is finished, you cannot call `offer` or `accept_offer`.
> `offer` and `accept_offer` should fail after `hack_finish_sale`.
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:5", "ft_token_id": "near", "start": null, "duration": "100000000"}' --accountId $ALICE --depositYocto 300 --gas 200000000000000
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:5"}'

near call $MARKET_CONTRACT_ID hack_finish_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'" "token_id": "1:5"}' --accountId $ALICE

near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:5", "ft_token_id": "near", "start": null, "duration": "100000000"}' --accountId $ALICE --depositYocto 400 --gas 200000000000000
near call $MARKET_CONTRACT_ID accept_offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:5", "ft_token_id": "near"}' --accountId $CONTRACT_PARENT --gas 200000000000000

near view $NFT_CONTRACT_ID nft_token '{"token_id": "1:5"}'
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:5"}'
```
> Here we called `hack_finish_sale` in order to finish the sale ahead of time. It is done for demonstration purposes. All content of `hack.rs` should be deleted later.

If `ALICE` decides to sell one of her NFTs, the royalty fee will be taken from the price:
```bash
near call $MARKET_CONTRACT_ID storage_deposit --accountId $ALICE --deposit 0.1

near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:1", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": null} }"}' --accountId $ALICE --deposit 1
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:1"}'

near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:1", "ft_token_id": "near", "start": null, "duration": "100000000"}' --accountId $NFT_CONTRACT_ID --depositYocto 10300 --gas 300000000000000
near view $NFT_CONTRACT_ID nft_token '{"token_id": "1:1"}'
```

### List of view methods for sales
To find number of sales:
```bash
near view $MARKET_CONTRACT_ID get_supply_sales
```

To show all sales (with pagination or without it):
```bash
near view $MARKET_CONTRACT_ID get_sales
near view $MARKET_CONTRACT_ID get_sales '{"from_index": "0", "limit": 10}'
```

To get the sale:
```bash
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:1"}'
```

To find number of sales for given owner:
```bash
near view $MARKET_CONTRACT_ID get_supply_by_owner_id '{"account_id": "'$CONTRACT_PARENT'"}'
```

To get sales for the given owner:
```bash
near view $MARKET_CONTRACT_ID get_sales_by_owner_id '{"account_id": "'$CONTRACT_PARENT'", "from_index": "0", "limit": 10}'
```

To find number of sales for given nft contract:
```bash
near view $MARKET_CONTRACT_ID get_supply_by_nft_contract_id '{"nft_contract_id": "'$NFT_CONTRACT_ID'"}'
```

To get sales for the given nft contract:
```bash
near view $MARKET_CONTRACT_ID get_sales_by_nft_contract_id '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "from_index": "0", "limit": 10}'
```

To find number of sales for token type:
```bash
near view $MARKET_CONTRACT_ID get_supply_by_nft_token_type '{"token_type": "near"}'
```

To get sales for token type:
```bash
near view $MARKET_CONTRACT_ID get_sales_by_nft_token_type '{"token_type": "near", "from_index": "0", "limit": 10}'
```

To get the full price with a protocol and origins fee:
```bash
near view $MARKET_CONTRACT_ID price_with_fees '{"price": "10000", "origins": null}'
```
<sub> This method is not specific for sales. Can be used in context of auctions.

### Workflow for creating and using auction

`CONTRACT_PARENT` puts three NFTs on auction:
```bash
near call $MARKET_CONTRACT_ID storage_deposit --accountId $CONTRACT_PARENT --deposit 0.03

near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:6", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Auction\": {\"token_type\": \"near\", \"minimal_step\": \"100\", \"start_price\": \"10000\", \"start\": null, \"duration\": \"900000000000\", \"buy_out_price\": \"10000000000\", \"origins\": {\"'$NFT_CONTRACT_ID'\": 100}} }"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:7", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Auction\": {\"token_type\": \"near\", \"minimal_step\": \"100\", \"start_price\": \"10000\", \"start\": null, \"duration\": \"900000000000\", \"buy_out_price\": \"10000000000\", \"origins\": {\"'$NFT_CONTRACT_ID'\": 100}} }"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:8", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Auction\": {\"token_type\": \"near\", \"minimal_step\": \"100\", \"start_price\": \"10000\", \"start\": null, \"duration\": \"900000000000\", \"buy_out_price\": \"10000000000\", \"origins\": {\"'$NFT_CONTRACT_ID'\": 100}} }"}' --accountId $CONTRACT_PARENT --deposit 1

near view $MARKET_CONTRACT_ID get_auctions
near view $MARKET_CONTRACT_ID price_with_fees '{"price": "10000", "origins": null}'
```

The duration `900000000000` corresponds to 15 minutes.
You can't set the duration lower than that. `CONTRACT_PARENT` can set the specific start time, otherwise the auction starts as soon as the command is complete.
There is a `buy_out_price`, meaning that anyone can buy the NFT for this price. `CONTRACT_PARENT` could have disabled this feature by setting `buy_out_price` to `null`.
The parameters `start_price`, `minimal_step` and `buy_out_price` do not include fees, to get the final amounts we can call `price_with_fees`.

`CONTRACT_PARENT` can cancel his auction before it has reached its end. It is possible only in case there is no bid for this auction:
```bash
near call $MARKET_CONTRACT_ID cancel_auction '{"auction_id": "0"}' --accountId $CONTRACT_PARENT --depositYocto 1

near view $MARKET_CONTRACT_ID get_auctions
```

`ALICE` can create a bid on the ongoing auction:
```bash
near call $MARKET_CONTRACT_ID auction_add_bid '{"auction_id": "1", "token_type": "near"}' --accountId $ALICE --depositYocto 10300

near view $MARKET_CONTRACT_ID get_auction '{"auction_id": "1"}'
```
In our case, this call happens less than 15 minutes before the end of the auction, thus the auction is extended.

A bid for an auction can't be deleted.

If `ALICE` calls `auction_add_bid` with deposit more or equal to buyout price (with fees), she automatically buys it. In this case the auction ends ahead of time.
```bash
near call $MARKET_CONTRACT_ID auction_add_bid '{"auction_id": "2", "token_type": "near"}' --accountId $ALICE --depositYocto 10300000000

near view $MARKET_CONTRACT_ID get_auction '{"auction_id": "2"}'
```

After auction ends anyone can finish it. It will transfer NFTs to those who bought it:
```bash
near call $MARKET_CONTRACT_ID hack_finish_auction '{"auction_id": "1"}' --accountId $ALICE

near call $MARKET_CONTRACT_ID finish_auction '{"auction_id": "1"}' --accountId $ALICE --gas 200000000000000
near call $MARKET_CONTRACT_ID finish_auction '{"auction_id": "2"}' --accountId $ALICE --gas 200000000000000

near view $NFT_CONTRACT_ID nft_token '{"token_id": "1:7"}'
near view $NFT_CONTRACT_ID nft_token '{"token_id": "1:8"}'

near view $MARKET_CONTRACT_ID get_auctions
```
> Here we called `hack_finish_auction` in order to finish the auction ahead of time. It is done for demonstration purposes. All content of `hack.rs` should be deleted later.

### List of view methods for auctions

To show all auctions (with pagination or without it):
```bash
near view $MARKET_CONTRACT_ID get_auctions '{"from_index": "0", "limit": 10}'
near view $MARKET_CONTRACT_ID get_auctions
```

To get the auction:
```bash
near view $MARKET_CONTRACT_ID get_auction '{"auction_id": "0"}'
```

To get the creator of the latest bid:
```bash
near view $MARKET_CONTRACT_ID get_current_buyer '{"auction_id": "0"}'
```

To check whether the auction in progress:
```bash
near view $MARKET_CONTRACT_ID check_auction_in_progress '{"auction_id": "0"}'
```

To get the minimal bid one could bid (including protocol and origin fees):
```bash
near view $MARKET_CONTRACT_ID get_minimal_next_bid '{"auction_id": "0"}'
```

To get the amount of the latest bid (with protocol and origin fees):
```bash
near view $MARKET_CONTRACT_ID get_current_bid '{"auction_id": "0"}'
```

To get the full price with a protocol and origins fee:
```bash
near view $MARKET_CONTRACT_ID price_with_fees '{"price": "10000", "origins": null}'
```
<sub> This method is not specific for auctions. Can be used in context of sales.