use crate::errors::SCErrors;
use crate::storage::core::{CoreData, TokenMetadata};
use crate::storage::items::Item;
use crate::storage::royalties::Royalty;
use crate::utils::balances::{bump_balance, get_balance, write_balance};
use crate::utils::core::{
    bump_instance, collection_currency, get_core_data, get_metadata, is_initialized,
    write_core_data, write_token_metadata,
};
use crate::utils::items::{
    bump_item, get_item, is_item_for_sale, is_minted, is_valid_item_number, write_item,
};
use crate::utils::royalties::{bump_royalties, get_royalties, write_royalties};
use num_integer::div_floor;
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, Address, BytesN, Env, String, Symbol,
    Vec,
};

use crate::events;

pub trait CollectibleTrait {
    /// This function starts the contract with data that can't be updated later without doing a full upgrade
    /// If this function hasn't been called, most functions won't work
    fn init(
        env: Env,
        admin: Address,
        supply: u64,
        initial_price: u128,
        initial_seller: Address,
        collection_currency: Address,
        name: String,
        symbol: String,
        metadata_uri: String,
        royalties: Vec<Royalty>,
    );

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>);

    /// This method could be used to know which version of the smart contract a collectible is using, making it easy if they want to upgrade later
    fn version(env: Env) -> Symbol;

    /// Balance of an account, it defaults to "0" even if the account has never interacted with this contract
    /// The balance is the total amount across all the collectibles, for example if an account owns "5" different collectibles from this contract, the balance will return "5"
    fn balance(env: Env, id: Address) -> u128;

    /// The function to buy items that are for sale, if it's the "first sale" of the item, the contract will send the payment to the "initial_seller"
    /// If an item is not for sale it will throw an error
    fn buy(env: Env, buyer: Address, item_number: u64);

    /// Use this function when you want to offer one of your Items
    /// You must be the owner of the Item
    /// Setting the price to "0" is equal to cancelling the offer
    fn sell(env: Env, item_number: u64, price: u128);

    /// Returns the specific Item, if the item hasn't been sold for the first time it will throw an error
    /// This function can be used to know if an Item is currently for sale
    fn item(env: Env, number: u64) -> Item;

    /// Transferring the ownership of a collectible
    /// The owner of the collectible is used as required authorization
    /// This function doesn't trigger the royalty payments
    fn transfer(env: Env, item_number: u64, to: Address);

    /// Similar to the transfer function but from the point of view of the initial_seller
    /// This "mints" an Item to the new owner, this means that if an item already exists we can not mint it again
    /// This function doesn't trigger the royalty payments
    fn mint(env: Env, item_number: u64, to: Address);

    fn decimals(e: Env) -> u32;

    fn name(e: Env) -> String;

    fn symbol(e: Env) -> String;

    fn metadata_uri(e: Env) -> String;

    fn royalties(e: Env) -> Vec<Royalty>;

    fn supply(e: Env) -> u64;

    fn core_data(e: Env) -> CoreData;
    fn token_metadata(e: Env) -> TokenMetadata;
}

#[contract]
pub struct CollectibleContract;

#[contractimpl]
impl CollectibleTrait for CollectibleContract {
    fn init(
        env: Env,
        admin: Address,
        supply: u64,
        initial_price: u128,
        initial_seller: Address,
        collection_currency: Address,
        name: String,
        symbol: String,
        metadata_uri: String,
        royalties: Vec<Royalty>,
    ) {
        if is_initialized(&env) {
            panic_with_error!(&env, &SCErrors::AlreadyInitialized);
        }

        write_core_data(
            &env,
            &CoreData {
                admin,
                supply,
                initial_price,
                initial_seller,
                collection_currency,
            },
        );

        write_token_metadata(
            &env,
            TokenMetadata {
                name,
                symbol,
                metadata_uri,
            },
        );

        write_royalties(&env, &royalties);

        bump_instance(&env);
        bump_royalties(&env);
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        bump_instance(&env);
        get_core_data(&env).admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn version(env: Env) -> Symbol {
        bump_instance(&env);
        symbol_short!("0_0_1")
    }

    fn balance(env: Env, id: Address) -> u128 {
        bump_instance(&env);
        bump_balance(&env, &id);
        get_balance(&env, &id)
    }

    // NOTE: Something we can improve is the use of "is_minted" and "get_item" so we don't call the storage that often
    fn buy(env: Env, buyer: Address, item_number: u64) {
        bump_instance(&env);
        buyer.require_auth();

        let core_data: CoreData = get_core_data(&env);

        if !is_valid_item_number(&core_data, &item_number) {
            panic_with_error!(&env, &SCErrors::ItemNumberIsInvalid);
        }

        if !is_item_for_sale(&env, &item_number) {
            panic_with_error!(&env, &SCErrors::ItemIsNotForSale);
        }

        let is_minted_val: bool = is_minted(&env, &item_number);

        // If is already minted, we reduce the balance amount of the old owner
        if is_minted_val {
            let item: Item = get_item(&env, &item_number);
            let balance: u128 = get_balance(&env, &item.owner);
            write_balance(&env, &item.owner, &(balance - 1));
            bump_balance(&env, &item.owner);
        }

        let royalties: Vec<Royalty> = get_royalties(&env);

        let collection_currency = collection_currency(&env, &core_data);
        let price: u128 = if is_minted_val {
            get_item(&env, &item_number).price
        } else {
            core_data.initial_price
        };
        let seller: Address = if is_minted_val {
            get_item(&env, &item_number).owner
        } else {
            core_data.initial_seller
        };

        // We distribute the royalties and we pay the owner
        let mut royalties_distributed: u128 = 0u128;
        for royalty in royalties.iter() {
            if is_minted_val && royalty.first_sale {
                // If is already minted, it means this is not a first sale so we ignore this distribution for second sales
                continue;
            }

            let share = div_floor(price * royalty.percentage, 1_0000000);
            collection_currency.transfer(&buyer, &royalty.address, &(share as i128));
            royalties_distributed += share;

            events::royalty_payment(&env, item_number.clone(), royalty.address, share);
        }

        // We set the new owner and increase its balance
        write_item(
            &env,
            &Item {
                number: item_number.clone(),
                for_sale: false,
                owner: buyer.clone(),
                price: 0,
            },
        );
        let mut new_owner_balance: u128 = get_balance(&env, &buyer);
        new_owner_balance += 1;
        write_balance(&env, &buyer, &new_owner_balance);

        collection_currency.transfer(&buyer, &seller, &((price - royalties_distributed) as i128));

        bump_item(&env, &item_number);
        bump_balance(&env, &buyer);
        bump_royalties(&env);

        events::buy(&env, seller, buyer, item_number, price);
    }

    fn sell(env: Env, item_number: u64, price: u128) {
        bump_instance(&env);

        let mut item: Item = get_item(&env, &item_number);
        item.owner.require_auth();

        item.for_sale = price != 0;
        item.price = price;

        write_item(&env, &item);

        bump_item(&env, &item_number);
        bump_royalties(&env);
        bump_balance(&env, &item.owner);

        events::sell(&env, item.owner, item_number, price);
    }

    fn item(env: Env, number: u64) -> Item {
        bump_instance(&env);
        let core_data: CoreData = get_core_data(&env);

        if !is_valid_item_number(&core_data, &number) {
            panic_with_error!(&env, &SCErrors::ItemNumberIsInvalid);
        }

        bump_item(&env, &number);
        get_item(&env, &number)
    }

    fn transfer(env: Env, item_number: u64, to: Address) {
        bump_instance(&env);

        let mut item: Item = get_item(&env, &item_number);
        item.owner.require_auth();

        // We first reduce the balance of the current owner
        let current_owner_balance = get_balance(&env, &item.owner);
        write_balance(&env, &item.owner, &(current_owner_balance - 1));

        // We now increase new owner balance
        let new_owner_balance = get_balance(&env, &to);
        write_balance(&env, &to, &(new_owner_balance + 1));

        // We update the ownership of the item
        item.owner = to.clone();
        item.price = 0;
        item.for_sale = false;
        write_item(&env, &item);

        bump_item(&env, &item_number);
        bump_royalties(&env);
        bump_balance(&env, &item.owner);

        events::transfer(&env, item.owner, to, item_number);
    }

    fn mint(e: Env, item_number: u64, to: Address) {
        bump_instance(&e);

        let core_data: CoreData = get_core_data(&e);
        core_data.initial_seller.require_auth();

        if is_minted(&e, &item_number) {
            panic_with_error!(&e, &SCErrors::ItemWasAlreadyMinted);
        }

        // We now increase new owner balance
        let new_owner_balance: u128 = get_balance(&e, &to);
        write_balance(&e, &to, &(new_owner_balance + 1));

        // We set the new owner and increase its balance
        write_item(
            &e,
            &Item {
                number: item_number.clone(),
                for_sale: false,
                owner: to.clone(),
                price: 0,
            },
        );

        bump_item(&e, &item_number);
        bump_balance(&e, &to);
        bump_royalties(&e);

        events::mint(&e, to, item_number);
    }

    fn decimals(e: Env) -> u32 {
        bump_instance(&e);
        0
    }

    fn name(e: Env) -> String {
        bump_instance(&e);
        get_metadata(&e).name
    }

    fn symbol(e: Env) -> String {
        bump_instance(&e);
        get_metadata(&e).symbol
    }

    fn metadata_uri(e: Env) -> String {
        bump_instance(&e);
        get_metadata(&e).metadata_uri
    }

    fn royalties(e: Env) -> Vec<Royalty> {
        bump_instance(&e);
        get_royalties(&e)
    }

    fn supply(e: Env) -> u64 {
        bump_instance(&e);
        get_core_data(&e).supply
    }

    fn core_data(e: Env) -> CoreData {
        bump_instance(&e);
        get_core_data(&e)
    }

    fn token_metadata(e: Env) -> TokenMetadata {
        bump_instance(&e);
        get_metadata(&e)
    }
}
