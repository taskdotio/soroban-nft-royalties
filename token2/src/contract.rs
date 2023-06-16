
use soroban_sdk::{contractimpl, contracttype, Address, Bytes, Env, Symbol, String};
use soroban_token_sdk::{TokenMetadata, TokenUtils};
use crate::event;

#[derive(Clone)]
#[contracttype]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Allowance(AllowanceDataKey),
    Balance(Address),
    Nonce(Address),
    State(Address),
    Admin,
    RoyaltyR,
    RoyaltyP,
}


pub trait TokenTrait {
    fn initialize(env: Env, admin: Address, decimal: u32, name: Bytes, symbol: Bytes, royaltyr: Address, royaltyp: u32);

    fn allowance(env: Env, from: Address, spender: Address) -> i128;

    fn increase_allowance(env: Env, from: Address, spender: Address, amount: i128);

    fn decrease_allowance(env: Env, from: Address, spender: Address, amount: i128);

    fn balance(env: Env, id: Address) -> i128;

    fn spendable_balance(env: Env, id: Address) -> i128;

    fn authorized(env: Env, id: Address) -> bool;

    fn transfer(env: Env, from: Address, to: Address, amount: i128);

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128);

    fn burn(env: Env, from: Address, amount: i128);

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128);

    fn clawback(env: Env, from: Address, amount: i128);

    fn set_authorized(env: Env, id: Address, authorize: bool);

    fn mint(env: Env, to: Address, amount: i128);

    fn set_admin(env: Env, new_admin: Address);

    fn decimals(env: Env) -> u32;

    fn name(env: Env) -> Bytes;

    fn symbol(env: Env) -> Bytes;
}

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

pub struct Token;

#[contractimpl]
impl TokenTrait for Token {
    fn initialize(env: Env, admin: Address, decimal: u32, name: Bytes, symbol: Bytes, royaltyr: Address, royaltyp: u32) {
        if has_administrator(&env) {
            panic!("already initialized")
        }
        write_administrator(&env, &admin);
        if decimal > u8::MAX.into() {
            panic!("Decimal must fit in a u8");
        }
        write_royalty(&env, royaltyr);
        write_royalty_rate(&env, royaltyp);

        write_metadata(
            &env,
            TokenMetadata {
                decimal,
                name,
                symbol,
            },
        )
    }

    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&env, from, spender)
    }

    fn increase_allowance(env: Env, from: Address, spender: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        let allowance = read_allowance(&env, from.clone(), spender.clone());
        let new_allowance = allowance
            .checked_add(amount)
            .expect("Updated allowance doesn't fit in an i128");

        write_allowance(&env, from.clone(), spender.clone(), new_allowance);
        event::increase_allowance(&env, from, spender, amount);
    }

    fn decrease_allowance(env: Env, from: Address, spender: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        let allowance = read_allowance(&env, from.clone(), spender.clone());
        if amount >= allowance {
            write_allowance(&env, from.clone(), spender.clone(), 0);
        } else {
            write_allowance(&env, from.clone(), spender.clone(), allowance - amount);
        }
        event::decrease_allowance(&env, from, spender, amount);
    }

    fn balance(env: Env, id: Address) -> i128 {
        read_balance(&env, id)
    }

    fn spendable_balance(env: Env, id: Address) -> i128 {
        read_balance(&env, id)
    }

    fn authorized(env: Env, id: Address) -> bool {
        is_authorized(&env, id)
    }

    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);
        spend_balance(&env, from.clone(), amount);
        receive_balance(&env, to.clone(), amount);
        event::transfer(&env, from, to, amount);
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);
        spend_allowance(&env, from.clone(), spender, amount);
        spend_balance(&env, from.clone(), amount);
        receive_balance(&env, to.clone(), amount);
        event::transfer(&env, from, to, amount)
    }

    fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);
        spend_balance(&env, from.clone(), amount);
        event::burn(&env, from, amount);
    }

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);
        spend_allowance(&env, from.clone(), spender, amount);
        spend_balance(&env, from.clone(), amount);
        event::burn(&env, from, amount)
    }

    fn clawback(env: Env, from: Address, amount: i128) {
        check_nonnegative_amount(amount);
        let admin = read_administrator(&env);
        admin.require_auth();
        spend_balance(&env, from.clone(), amount);
        event::clawback(&env, admin, from, amount);
    }

    fn set_authorized(env: Env, id: Address, authorize: bool) {
        let admin = read_administrator(&env);
        admin.require_auth();
        write_authorization(&env, id.clone(), authorize);
        event::set_authorized(&env, admin, id, authorize);
    }

    fn mint(env: Env, to: Address, amount: i128) {
        check_nonnegative_amount(amount);
        let admin = read_administrator(&env);
        admin.require_auth();
        receive_balance(&env, to.clone(), amount);
        event::mint(&env, admin, to, amount);
    }

    fn set_admin(env: Env, new_admin: Address) {
        let admin = read_administrator(&env);
        admin.require_auth();
        write_administrator(&env, &new_admin);
        event::set_admin(&env, admin, new_admin);
    }

    fn decimals(env: Env) -> u32 {
        read_decimal(&env)
    }

    fn name(env: Env) -> Bytes {
        read_name(&env)
    }

    fn symbol(env: Env) -> Bytes {
        read_symbol(&env)
    }
}


pub fn read_balance(env: &Env, addr: Address) -> i128 {
    let key = DataKey::Balance(addr);
    if let Some(balance) = env.storage().get(&key) {
        balance.unwrap()
    } else {
        0
    }
}

fn write_balance(env: &Env, addr: Address, amount: i128) {
    let key = DataKey::Balance(addr);
    env.storage().set(&key, &amount);
}

pub fn receive_balance(env: &Env, addr: Address, amount: i128) {
    let balance = read_balance(env, addr.clone());
    if !is_authorized(env, addr.clone()) {
        panic!("can't receive when deauthorized");
    }
    write_balance(env, addr, balance + amount);
}

pub fn spend_balance(env: &Env, addr: Address, amount: i128) {
    let balance = read_balance(env, addr.clone());
    if !is_authorized(env, addr.clone()) {
        panic!("can't spend when deauthorized");
    }
    if balance < amount {
        panic!("insufficient balance");
    }
    write_balance(env, addr, balance - amount);
}

pub fn is_authorized(env: &Env, addr: Address) -> bool {
    let key = DataKey::State(addr);
    if let Some(state) = env.storage().get(&key) {
        state.unwrap()
    } else {
        true
    }
}

pub fn write_authorization(env: &Env, addr: Address, is_authorized: bool) {
    let key = DataKey::State(addr);
    env.storage().set(&key, &is_authorized);
}
pub fn read_allowance(env: &Env, from: Address, spender: Address) -> i128 {
    let key = DataKey::Allowance(AllowanceDataKey { from, spender });
    if let Some(allowance) = env.storage().get(&key) {
        allowance.unwrap()
    } else {
        0
    }
}

pub fn write_allowance(env: &Env, from: Address, spender: Address, amount: i128) {
    let key = DataKey::Allowance(AllowanceDataKey { from, spender });
    env.storage().set(&key, &amount);
}

pub fn spend_allowance(env: &Env, from: Address, spender: Address, amount: i128) {
    let allowance = read_allowance(env, from.clone(), spender.clone());
    if allowance < amount {
        panic!("insufficient allowance");
    }
    write_allowance(env, from, spender, allowance - amount);
}

pub fn has_administrator(env: &Env) -> bool {
    let key = DataKey::Admin;
    env.storage().has(&key)
}

pub fn read_administrator(env: &Env) -> Address {
    let key = DataKey::Admin;
    env.storage().get_unchecked(&key).unwrap()
}

pub fn write_administrator(env: &Env, id: &Address) {
    let key = DataKey::Admin;
    env.storage().set(&key, id);
}


// Metadata
pub fn read_decimal(env: &Env) -> u32 {
    let util = TokenUtils::new(env);
    util.get_metadata_unchecked().unwrap().decimal
}

pub fn read_name(env: &Env) -> Bytes {
    let util = TokenUtils::new(env);
    util.get_metadata_unchecked().unwrap().name
}

pub fn read_symbol(env: &Env) -> Bytes {
    let util = TokenUtils::new(env);
    util.get_metadata_unchecked().unwrap().symbol
}

pub fn write_metadata(env: &Env, metadata: TokenMetadata) {
    let util = TokenUtils::new(env);
    util.set_metadata(&metadata);
}

//royalty
pub fn write_royalty(env: &Env, recipient: Address) {
    let key = DataKey::RoyaltyR ;
    env.storage().set(&key, &recipient);
}
pub fn write_royalty_rate(env: &Env, percentage: u32) {
    let key = DataKey::RoyaltyP;
    env.storage().set(&key, &percentage);
}
