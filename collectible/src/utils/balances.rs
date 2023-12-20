use crate::bumps::{BALANCES_BUMP_CONSTANT, BALANCES_BUMP_CONSTANT_THRESHOLD};
use crate::storage::balances::BalancesDataKeys;
use soroban_sdk::{Address, Env};

pub fn bump_balance(env: &Env, id: &Address) {
    if env
        .storage()
        .persistent()
        .has(&BalancesDataKeys::Balance(id.clone()))
    {
        env.storage().persistent().extend_ttl(
            &BalancesDataKeys::Balance(id.clone()),
            BALANCES_BUMP_CONSTANT_THRESHOLD,
            BALANCES_BUMP_CONSTANT,
        );
    }
}

pub fn write_balance(env: &Env, address: &Address, balance: &u128) {
    env.storage()
        .persistent()
        .set(&BalancesDataKeys::Balance(address.clone()), balance);
}

pub fn get_balance(env: &Env, address: &Address) -> u128 {
    if !env
        .storage()
        .persistent()
        .has(&BalancesDataKeys::Balance(address.clone()))
    {
        0u128
    } else {
        env.storage()
            .persistent()
            .get(&BalancesDataKeys::Balance(address.clone()))
            .unwrap()
    }
}
