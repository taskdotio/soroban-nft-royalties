use crate::bumps::{ITEMS_BUMP_CONSTANT, ITEMS_BUMP_CONSTANT_THRESHOLD};
use crate::errors::SCErrors;
use crate::storage::core::CoreData;
use crate::storage::items::{Item, ItemsDataKeys};
use soroban_sdk::{panic_with_error, Address, Env};

pub fn bump_item(env: &Env, number: &u64) {
    if env
        .storage()
        .persistent()
        .has(&ItemsDataKeys::Item(number.clone()))
    {
        env.storage().persistent().bump(
            &ItemsDataKeys::Item(number.clone()),
            ITEMS_BUMP_CONSTANT_THRESHOLD,
            ITEMS_BUMP_CONSTANT,
        );
    }
}

pub fn is_minted(env: &Env, number: &u64) -> bool {
    env.storage()
        .persistent()
        .has(&ItemsDataKeys::Item(number.clone()))
}

pub fn is_valid_item_number(core_data: &CoreData, number: &u64) -> bool {
    &core_data.supply >= number && number >= &0u64
}

pub fn get_item(env: &Env, number: &u64) -> Item {
    if is_minted(&env, &number) {
        env.storage()
            .persistent()
            .get(&ItemsDataKeys::Item(number.clone()))
            .unwrap()
    } else {
        panic_with_error!(&env, &SCErrors::ItemHasNotBeenMinted);
    }
}

pub fn set_item(env: &Env, item: &Item) {
    env.storage()
        .persistent()
        .set(&ItemsDataKeys::Item(item.number.clone()), item);
}

/// An Item is for sale in two situations:
/// - The owner puts it for sale on an offer IE Item.for_sale == true
/// - The Item record doesn't exist, this means it's still on the first sale IE hasn't been "minted" yet
pub fn is_item_for_sale(env: &Env, number: &u64) -> bool {
    if is_minted(&env, &number) {
        get_item(&env, &number).for_sale
    } else {
        true
    }
}

///
pub fn buy_item(env: &Env, core_data: &CoreData, buyer: &Address, number: u64) {
    if is_minted(&env, &number) {
        // TODO: needs to be done
        panic_with_error!(env, &SCErrors::UnexpectedError);
    } else {
    }
}
