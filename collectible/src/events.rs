use soroban_sdk::{symbol_short, Address, Env};

pub(crate) fn buy(env: &Env, seller: Address, buyer: Address, item_id: u64, price: u128) {
    let topics = (&symbol_short!("buy"), seller, buyer);
    env.events().publish(topics, (item_id, price));
}

pub(crate) fn sell(env: &Env, seller: Address, item_id: u64, price: u128) {
    let topics = (&symbol_short!("sell"), seller);
    env.events().publish(topics, (item_id, price));
}

pub(crate) fn transfer(env: &Env, from: Address, to: Address, item_id: u64) {
    let topics = (symbol_short!("transfer"), from, to);
    env.events().publish(topics, item_id);
}

pub(crate) fn mint(env: &Env, to: Address, item_id: u64) {
    let topics = (symbol_short!("transfer"), to);
    env.events().publish(topics, item_id);
}

pub(crate) fn royalty_payment(env: &Env, item_id: u64, to: Address, amount: u128) {
    let topics = (symbol_short!("royalty"), to);
    env.events().publish(topics, (item_id, amount));
}
