use soroban_sdk::{symbol_short, Address, Env, Symbol};

pub(crate) fn approve(env: &Env, from: Address, to: Address, amount: i128, expiration_ledger: u32) {
    let topics = (Symbol::new(env, "approve"), from, to);
    env.events().publish(topics, (amount, expiration_ledger));
}

pub(crate) fn transfer(env: &Env, from: Address, to: Address, amount: i128) {
    let topics = (symbol_short!("transfer"), from, to);
    env.events().publish(topics, amount);
}

pub(crate) fn mint(env: &Env, admin: Address, to: Address, amount: i128) {
    let topics = (symbol_short!("mint"), admin, to);
    env.events().publish(topics, amount);
}

pub(crate) fn clawback(env: &Env, admin: Address, from: Address, amount: i128) {
    let topics = (symbol_short!("clawback"), admin, from);
    env.events().publish(topics, amount);
}

pub(crate) fn set_authorized(env: &Env, admin: Address, id: Address, authorize: bool) {
    let topics = (Symbol::new(env, "set_authorized"), admin, id);
    env.events().publish(topics, authorize);
}

pub(crate) fn set_admin(env: &Env, admin: Address, new_admin: Address) {
    let topics = (symbol_short!("set_admin"), admin);
    env.events().publish(topics, new_admin);
}

pub(crate) fn burn(env: &Env, from: Address, amount: i128) {
    let topics = (symbol_short!("burn"), from);
    env.events().publish(topics, amount);
}