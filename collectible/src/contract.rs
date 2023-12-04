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
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Map, String};

use crate::events;

pub trait CollectibleTrait {
    fn initialize(
        env: Env,
        admin: Address,
        supply: u64,
        initial_price: u128,
        initial_seller: Address,
        collection_currency: Address,
        name: String,
        symbol: String,
        metadata_uri: String,
        royalties: Map<Address, Royalty>,
    );

    fn balance(env: Env, id: Address) -> u128;

    fn buy(env: Env, buyer: Address, item_number: u64);

    /// - Use this function when you want to offer one of your Items
    /// - You must be the owner of the Item
    /// - Setting the price to "0" is equal to cancelling the offer
    fn sell(env: Env, item_number: u64, price: u128);

    fn item(env: Env, number: u64) -> Item;

    /// Transferring the ownership of a collectible
    fn transfer(env: Env, item_number: u64, to: Address);

    fn decimals(e: Env) -> u32;

    fn name(e: Env) -> String;

    fn symbol(e: Env) -> String;

    fn metadata_uri(e: Env) -> String;
}

#[contract]
pub struct CollectibleContract;

#[contractimpl]
impl CollectibleTrait for CollectibleContract {
    fn initialize(
        env: Env,
        admin: Address,
        supply: u64,
        initial_price: u128,
        initial_seller: Address,
        collection_currency: Address,
        name: String,
        symbol: String,
        metadata_uri: String,
        royalties: Map<Address, Royalty>,
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

        let royalties: Map<Address, Royalty> = get_royalties(&env);

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
        for (address, royalty) in royalties.iter() {
            if is_minted_val && royalty.first_sale {
                // If is already minted, it means this is not a first sale so we ignore this distribution for second sales
                continue;
            }

            let share = div_floor(price * royalty.percentage, 1_0000000);
            collection_currency.transfer(&buyer, &address, &(share as i128));
            royalties_distributed += share;

            events::royalty_payment(&env, item_number.clone(), address, share);
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

        item.for_sale = if price == 0 { false } else { true };
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
}
