#![no_std]

use soroban_sdk::{contracterror, contractimpl, contracttype, unwrap::UnwrapOptimized,  token, Address, Env};


mod royalty_index {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/release/soroban_asset_royalty_index.wasm"
    );
}
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Offer,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    ContractAlreadyInitialized = 1,
    ContractNotInitialized = 2,
    InvalidAuth = 3,
    RoyaltyAlreadyWithdrawn = 4,
    InvalidInvoker = 5,
    InvalidArguments = 6,
    InvalidRoyaltyAsset = 7,
}

#[contracttype]
#[derive(Clone)]
//pub enum StorageKey {
  //  Parent,       // Address
  //  Child,        // Address
  //  TokenAddress, // Address
  //  Amount,       // i128
  //  Step,         // u64
  //  Latest,       // u64
//}
pub struct Offer {
    // Owner of this offer. Sells sell_token to get buy_token.
    pub seller: Address,
    pub sell_token: Address,
    pub buy_token: Address,
    // Seller-defined price of the sell token in arbitrary units.
    pub sell_price: u32,
    // Seller-defined price of the buy token in arbitrary units.
    pub buy_price: u32,
    // these are set by the contract.
    pub royalty_recipient: Address,
    pub royalty_pct: u32,
}

pub struct RoyaltyOfferContract;

pub trait RoyaltyOfferTrait {
    fn create(
        e: Env,
        seller: Address,         // the seller
        royalty_index: Address,  // the royalty index
        sell_token: Address,     // The sell token id
        buy_token: Address,      // the buy token id
        sell_price: u32,           // seller defined price of sell token
        buy_price: u32,              // serller defined price of buy token
    ) -> Result<(), Error>;
    fn trade(e: Env, buyer: Address, buy_token_amount: i128, min_sell_token_amount: i128) -> Result<(), Error>;

    fn get_offer(e: Env) -> Offer;
    fn updt_price(e: Env, sell_price: u32, buy_price: u32);

    // When `seller withdraw` is invoked, the contract will transfer the royalty if there is any.
    fn s_withdraw(e: Env, token: Address, amount: i128) -> Result<(), Error>;
    fn r_withdraw(e: Env, token: Address, amount: i128) -> Result<(), Error>;
}

#[contractimpl]
impl RoyaltyOfferTrait for RoyaltyOfferContract {
    fn create(
        e: Env,
        seller: Address,
        royalty_index: Address, // this should be derived from sell_token but not done yet.
        sell_token: Address,
        buy_token: Address,
        sell_price: u32,
        buy_price: u32,
    ) -> Result<(), Error> {
        if e.storage().has(&DataKey::Offer) {
            panic!("offer is already created");
        }

        if buy_price == 0 || sell_price == 0 {
            panic!("zero price is not allowed");
        }

        // Authorize the `create` call by seller to verify their identity.
        seller.require_auth();
        //verify there is royalty info.
        let royaltyclient = royalty_index::Client::new(&e, &royalty_index);
        let roylatydata: (Address, Address, Address, u32) = royaltyclient.get();
        let (royalty_issuer, royalty_asset, royalty_recipient, royalty_pct) = roylatydata;
        if royalty_asset != sell_token {
            return Err(Error::InvalidRoyaltyAsset);
        }
         
        write_offer(
            &e,
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
        Ok(())
    }

    fn trade(e: Env, buyer: Address, buy_token_amount: i128, min_sell_token_amount: i128) {
        // Buyer needs to authorize the trade.
        buyer.require_auth();
    
        // Load the offer and prepare the token clients to do the trade.
        let offer = load_offer(&e);
        let sell_token_client = token::Client::new(&e, &offer.sell_token);
        let buy_token_client = token::Client::new(&e, &offer.buy_token);
        let royalty_percent = offer.royalty_pct;
    
        // Compute the amount of token that buyer needs to receive.
        let sell_token_amount = buy_token_amount
            .checked_mul(offer.sell_price as i128)
            .unwrap_optimized()
            / offer.buy_price as i128;
    
        if sell_token_amount < min_sell_token_amount {
            panic!("price is too low");
        }
    
        let contract = e.current_contract_address();
    
        // Calculate the royalty amount and the seller's share
        let royalty_amount = (buy_token_amount * royalty_percent as i128) / 100;
        let seller_amount = buy_token_amount - royalty_amount;
    
        // Perform the trade in 4 `transfer` steps.
        // Note, that we don't need to verify any balances - the contract would
        // just trap and roll back in case if any of the transfers fails for
        // any reason, including insufficient balance.
    
        // Transfer the `buy_token` from buyer to this contract.
        // This `transfer` call should be authorized by buyer.

        buy_token_client.transfer(&buyer, &contract, &buy_token_amount);
        // Transfer the `sell_token` from contract to buyer.
        sell_token_client.transfer(&contract, &buyer, &sell_token_amount);
        // Transfer the `buy_token` to the seller immediately.
        buy_token_client.transfer(&contract, &offer.seller, &seller_amount);
        // Transfer the `buy_token` to the royalty recipient, this might need to be stored in the contract as a balance if we want to.
        buy_token_client.transfer(&contract, &offer.royalty_recipient, &royalty_amount);
    }
    
//this is not yet implemented as we need to setup a place to store the balance.

    fn r_withdraw(e: Env, token: Address, amount: i128) {
        let offer = load_offer(&e);
        offer.royalty_recipient.require_auth();
       
        token::Client::new(&e, &token).transfer(
            &e.current_contract_address(), //from this contract
            &offer.seller, //to the seller
            &amount, //in this amount which should be a balance stored
        );
    }
    
    fn s_withdraw(e: Env, token: Address, amount: i128) {
        let offer = load_offer(&e);
        offer.seller.require_auth(); //require the sellers auth
        token::Client::new(&e, &token).transfer(
            &e.current_contract_address(),//from this contract
            &offer.seller, //to the seller
            &amount,// the amount requested
        );
    }

    fn updt_price(e: Env, sell_price: u32, buy_price: u32) {
        if buy_price == 0 || sell_price == 0 {
            panic!("zero price is not allowed");
        }
        let mut offer = load_offer(&e);
        offer.seller.require_auth();
        offer.sell_price = sell_price;
        offer.buy_price = buy_price;
        write_offer(&e, &offer);
    }
    fn get_offer(e: Env) -> Offer {
        let offer = load_offer(&e);
        offer
    }
}

fn load_offer(e: &Env) -> Offer {
    e.storage().get_unchecked(&DataKey::Offer).unwrap()
}

fn write_offer(e: &Env, offer: &Offer) {
    e.storage().set(&DataKey::Offer, offer);
}
