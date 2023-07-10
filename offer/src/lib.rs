//! This contract implements trading of one token pair between one seller and
//! multiple buyer.
//! It demonstrates one of the ways of how trading might be implemented.
#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, token, unwrap::UnwrapOptimized, Address, Env,};

mod royalty_token {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_nft_with_royalty_contract.wasm"
    );
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Offer,
}

// Represents an offer managed by the SingleOffer contract.
// If a seller wants to sell 1000 XLM for 100 USDC the `sell_price` would be 1000
// and `buy_price` would be 100 (or 100 and 10, or any other pair of integers
// in 10:1 ratio).
#[derive(Clone)]
#[contracttype]
pub struct Offer {
    // Owner of this offer. Sells sell_token to get buy_token.
    pub seller: Address,
    pub sell_token: Address,
    pub buy_token: Address,
    // Seller-defined price of the sell token in arbitrary units.
    pub sell_price: u32,
    // Seller-defined price of the buy token in arbitrary units.
    pub buy_price: u32,
    // these are set by the token contract.
    pub royalty_recipient: Address,
    pub royalty_pct: u32,
}

#[contract]
pub struct RoyaltyOffer;

/*
pub trait RoyaltyOfferTrait {
    fn create(
        env: Env,
        seller: Address,         // the seller
        sell_token: Address,     // The sell token id
        buy_token: Address,      // the buy token id
        sell_price: u32,           // seller defined price of sell token
        buy_price: u32,              // serller defined price of buy token
    ) -> Result<(), Error>;
    fn trade(env: Env, buyer: Address, buy_token_amount: i128, min_sell_token_amount: i128) -> Result<(), Error>;

    fn get_offer(env: Env) -> Offer;
    fn updt_price(env: Env, sell_price: u32, buy_price: u32);

    // When `seller withdraw` is invoked, the contract will transfer the royalty if there is any.
    fn s_withdraw(env: Env, token: Address, amount: i128) -> Result<(), Error>;
    fn r_withdraw(env: Env, token: Address, amount: i128) -> Result<(), Error>;
}
*/

/*
How this contract should be used:

1. Call `create` once to create the offer and register its seller.
2. Seller may transfer arbitrary amounts of the `sell_token` for sale to the
   contract address for trading. They may also update the offer price.
3. Buyers may call `trade` to trade with the offer. The contract will
   immediately perform the trade and send the respective amounts of `buy_token`
   and `sell_token` to the seller and buyer respectively.
4. Seller may call `withdraw` to claim any remaining `sell_token` balance.
*/
#[contractimpl]
impl RoyaltyOffer {
    // Creates the offer for seller for the given token pair and initial price.
    // See comment above the `Offer` struct for information on pricing.
    pub fn create(
        env: Env,
        seller: Address,
        sell_token: Address,
        buy_token: Address,
        sell_price: u32,
        buy_price: u32,
    ) {
        if env.storage().instance().has(&DataKey::Offer) {
            panic!("offer is already created");
        }
        if buy_price == 0 || sell_price == 0 {
            panic!("zero price is not allowed");
        }
        // Authorize the `create` call by seller to verify their identity.
        seller.require_auth();
        //get the royalty info:
        let sell_token_client = royalty_token::Client::new(&env, &sell_token);
        let royalty_recipient: Address = sell_token_client.get_royalty_recipient();
        let royalty_pct = sell_token_client.get_royalty_rate();
        //let royalty_pct = roayltydata.1;

        write_offer(
            &env,
            &Offer {
                seller,
                sell_token,
                buy_token,
                sell_price,
                buy_price,
                royalty_recipient,
                royalty_pct,
            },
        );
    }

    // Trades `buy_token_amount` of buy_token from buyer for `sell_token` amount
    // defined by the price.
    // `min_sell_amount` defines a lower bound on the price that the buyer would
    // accept.
    // Buyer needs to authorize the `trade` call and internal `transfer` call to
    // the contract address.
    pub fn trade(env: Env, buyer: Address, buy_token_amount: i128, min_sell_token_amount: i128) {
        // Buyer needs to authorize the trade.
        buyer.require_auth();

        // Load the offer and prepare the token clients to do the trade.
        let offer = load_offer(&env);
        let sell_token_client = token::Client::new(&env, &offer.sell_token);
        let buy_token_client = token::Client::new(&env, &offer.buy_token);
        let royalty_percent = offer.royalty_pct;

        // Compute the amount of token that buyer needs to receive.
        let sell_token_amount = buy_token_amount
            .checked_mul(offer.sell_price as i128)
            .unwrap_optimized()
            / offer.buy_price as i128;

        if sell_token_amount < min_sell_token_amount {
            panic!("price is too low");
        }

        let contract = env.current_contract_address();
       // Calculate the royalty amount and the seller's share
        let royalty_amount = (buy_token_amount * royalty_percent as i128) / 100;
        let seller_amount = buy_token_amount - royalty_amount;

        // Perform the trade in 4 `transfer` steps.
        // Note, that we don't need to verify any balances - the contract would
        // just trap and roll back in case if any of the transfers fails for
        // any reason, including insufficient balance.

        // Transfer the `buy_token` from buyer to this contract.
        // This `transfer` call should be authorized by buyer.
        // This could as well be a direct transfer to the seller, but sending to
        // the contract address allows building more transparent signature
        // payload where the buyer doesn't need to worry about sending token to
        // some 'unknown' third party.

        buy_token_client.transfer(&buyer, &contract, &buy_token_amount);
        // Transfer the `sell_token` from contract to buyer.
        sell_token_client.transfer(&contract, &buyer, &sell_token_amount);
        // Transfer the `buy_token` to the seller immediately.
        buy_token_client.transfer(&contract, &offer.seller, &seller_amount);
        // transfer the 'buy_token' royalty to the royalty recipient, this might need to be stored in the contract as a balance for the recipient to withdraw later on depending on the token.
        buy_token_client.transfer(&contract, &offer.royalty_recipient, &royalty_amount);

    }

    // Sends amount of token from this contract to the seller.
    // This is intentionally flexible so that the seller can withdraw any
    // outstanding balance of the contract (in case if they mistakenly
    // transferred wrong token to it).
    // Must be authorized by seller.
    pub fn withdraw(env: Env, token: Address, amount: i128) {
        let offer = load_offer(&env);
        offer.seller.require_auth();
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &offer.seller,
            &amount,
        );
    }

    // Updates the price.
    // Must be authorized by seller.
    pub fn updt_price(env: Env, sell_price: u32, buy_price: u32) {
        if buy_price == 0 || sell_price == 0 {
            panic!("zero price is not allowed");
        }
        let mut offer = load_offer(&env);
        offer.seller.require_auth();
        offer.sell_price = sell_price;
        offer.buy_price = buy_price;
        write_offer(&env, &offer);
    }

    // Returns the current state of the offer.
    pub fn get_offer(env: Env) -> Offer {
        load_offer(&env)
    }
}

fn load_offer(env: &Env) -> Offer {
    env.storage().instance().get(&DataKey::Offer).unwrap()
}

fn write_offer(env: &Env, offer: &Offer) {
    env.storage().instance().set(&DataKey::Offer, offer);
}

mod test;
