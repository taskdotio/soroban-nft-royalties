
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};
use soroban_token_sdk::{TokenMetadata, TokenUtils};
use crate::event;


#[derive(Clone)]
#[contracttype]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
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
    fn initialize(env: Env, admin: Address, decimal: u32, name: String, symbol: String, royaltyr: Address, royaltyp: u32);

    fn allowance(env: Env, from: Address, spender: Address) -> i128;

    fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32);

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

    fn name(env: Env) -> String;

    fn symbol(env: Env) -> String;

    fn get_royalty_recipient(env: Env) -> Address;

    fn get_royalty_rate(env: Env) -> u32;
}

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}
#[contract]
pub struct Token;

#[contractimpl]
impl TokenTrait for Token {
    //we probably don't need to take the decimal since it will always be zero
    fn initialize(env: Env, admin: Address, decimal: u32, name: String, symbol: String, royaltyr: Address, royaltyp: u32) {
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
    fn get_royalty_rate(env: Env) -> u32 {
        let key = DataKey::RoyaltyP;
        //env.storage().get_unchecked(&key).unwrap()
        if let Some(royaltyrate) = env.storage().persistent().get::<DataKey, u32>(&key) {
            royaltyrate
        } else {
            panic!("no royalty data")
        }

    }
    fn get_royalty_recipient(env: Env) -> Address {
        let key = DataKey::RoyaltyR;
        //env.storage().get_unchecked(&key).unwrap()
        if let Some(royaltyrecipient) = env.storage().persistent().get::<DataKey, Address>(&key) {
            royaltyrecipient
        } else {
            panic!("no royalty recipient")
        }
    }

    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&env, from, spender).amount
    }

    fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        check_nonnegative_amount(amount);

        write_allowance(&env, from.clone(), spender.clone(), amount, expiration_ledger);
        event::approve(&env, from, spender, amount, expiration_ledger);
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

    fn name(env: Env) -> String {
        read_name(&env)
    }

    fn symbol(env: Env) -> String {
        read_symbol(&env)
    }
}

//balances
pub fn read_balance(env: &Env, addr: Address) -> i128 {
    let key = DataKey::Balance(addr);
    if let Some(balance) = env.storage().persistent().get::<DataKey, i128>(&key) {
        balance
    } else {
        0
    }
}

fn write_balance(env: &Env, addr: Address, amount: i128) {
    let key = DataKey::Balance(addr);
    env.storage().persistent().set(&key, &amount);
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
    if let Some(state) = env.storage().persistent().get::<DataKey, bool>(&key) {
        state
    } else {
        true
    }
}

pub fn write_authorization(env: &Env, addr: Address, is_authorized: bool) {
    let key = DataKey::State(addr);
    env.storage().persistent().set(&key, &is_authorized);
}

//allowances
pub fn read_allowance(env: &Env, from: Address, spender: Address) -> AllowanceValue {
    let key = DataKey::Allowance(AllowanceDataKey { from, spender });
    if let Some(allowance) = env.storage().temporary().get::<_, AllowanceValue>(&key) {
        if allowance.expiration_ledger < env.ledger().sequence() {
            AllowanceValue {
                amount: 0,
                expiration_ledger: allowance.expiration_ledger,
            }
        } else {
            allowance
        }
    } else {
        AllowanceValue {
            amount: 0,
            expiration_ledger: 0,
        }
    }
}

pub fn write_allowance(env: &Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
    let allowance = AllowanceValue {
        amount,
        expiration_ledger,
    };

    if amount > 0 && expiration_ledger < env.ledger().sequence() {
        panic!("expiration_ledger is less than ledger seq when amount > 0")
    }

    let key = DataKey::Allowance(AllowanceDataKey { from, spender });
    env.storage().temporary().set(&key.clone(), &allowance);

    if amount > 0 {
        env.storage().temporary().bump(
            &key,
            expiration_ledger
                .checked_sub(env.ledger().sequence())
                .unwrap(),
        )
    }
}

pub fn spend_allowance(env: &Env, from: Address, spender: Address, amount: i128) {
    let allowance = read_allowance(env, from.clone(), spender.clone());
    if allowance.amount < amount {
        panic!("insufficient allowance");
    }
    write_allowance(
        env,
        from,
        spender,
        allowance.amount - amount,
        allowance.expiration_ledger,
    );
}

//administrator
pub fn has_administrator(env: &Env) -> bool {
    let key = DataKey::Admin;
    env.storage().instance().has(&key)
}

pub fn read_administrator(env: &Env) -> Address {
    let key = DataKey::Admin;
    env.storage().instance().get(&key).unwrap()
}

pub fn write_administrator(env: &Env, id: &Address) {
    let key = DataKey::Admin;
    env.storage().instance().set(&key, id);
}

// Metadata
pub fn read_decimal(env: &Env) -> u32 {
    let util = TokenUtils::new(env);
    util.get_metadata().decimal
}

pub fn read_name(env: &Env) -> String {
    let util = TokenUtils::new(env);
    util.get_metadata().name
}

pub fn read_symbol(env: &Env) -> String {
    let util = TokenUtils::new(env);
    util.get_metadata().symbol
}

pub fn write_metadata(env: &Env, metadata: TokenMetadata) {
    let util = TokenUtils::new(env);
    util.set_metadata(&metadata);
}

//royalty
pub fn write_royalty(env: &Env, recipient: Address) {
    let key = DataKey::RoyaltyR ;
    env.storage().instance().set(&key, &recipient);
}
pub fn write_royalty_rate(env: &Env, percentage: u32) {
    let key = DataKey::RoyaltyP;
    env.storage().instance().set(&key, &percentage);
}
