//need to write more tests and modify these for nft
#![cfg(test)]
extern crate std;

use crate::{contract::Token, TokenClient};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, Symbol,
};

fn create_token<'a>(env: &Env, admin: &Address, royaltyr: &Address, royaltyp: &u32) -> TokenClient<'a> {
    let token = TokenClient::new(env, &env.register_contract(None, Token {}));
    token.initialize(admin, &0, &"name".into_val(env), &"symbol".into_val(env), royaltyr, royaltyp);
    token
}

#[test]
fn test() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::random(&env);
    let admin2 = Address::random(&env);
    let user1 = Address::random(&env);
    let user2 = Address::random(&env);
    let user3 = Address::random(&env);
    let royaltyr = Address::random(&env);
    let royaltyp = 10;

    let token = create_token(&env, &admin1, &royaltyr, &royaltyp);

    token.mint(&user1, &1000);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&user1, 1000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user1), 1000);

    token.approve(&user2, &user3, &500, &200);
    assert_eq!(
        env.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("approve"),
                    (&user2, &user3, 500_i128, 200_u32).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.allowance(&user2, &user3), 500);

    token.transfer(&user1, &user2, &600);
    assert_eq!(
        env.auths(),
        std::vec![(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("transfer"),
                    (&user1, &user2, 600_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user1), 400);
    assert_eq!(token.balance(&user2), 600);

    token.transfer_from(&user3, &user2, &user1, &400);
    assert_eq!(
        env.auths(),
        std::vec![(
            user3.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    Symbol::new(&env, "transfer_from"),
                    (&user3, &user2, &user1, 400_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user1), 800);
    assert_eq!(token.balance(&user2), 200);

    token.transfer(&user1, &user3, &300);
    assert_eq!(token.balance(&user1), 500);
    assert_eq!(token.balance(&user3), 300);

    token.set_admin(&admin2);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("set_admin"),
                    (&admin2,).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    token.set_authorized(&user2, &false);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    Symbol::new(&env, "set_authorized"),
                    (&user2, false).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.authorized(&user2), false);

    token.set_authorized(&user3, &true);
    assert_eq!(token.authorized(&user3), true);

    token.clawback(&user3, &100);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("clawback"),
                    (&user3, 100_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user3), 200);

    // Increase to 500
    token.approve(&user2, &user3, &500, &200);
    assert_eq!(token.allowance(&user2, &user3), 500);
    token.approve(&user2, &user3, &0, &200);
    assert_eq!(
        env.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("approve"),
                    (&user2, &user3, 0_i128, 200_u32).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.allowance(&user2, &user3), 0);
}

#[test]
fn test_burn() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user1 = Address::random(&env);
    let user2 = Address::random(&env);

    let royaltyr = Address::random(&env);
    let royaltyp = 10;

    let token = create_token(&env, &admin, &royaltyr, &royaltyp);
    

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.approve(&user1, &user2, &500, &200);
    assert_eq!(token.allowance(&user1, &user2), 500);

    token.burn_from(&user2, &user1, &500);
    assert_eq!(
        env.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("burn_from"),
                    (&user2, &user1, 500_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(token.allowance(&user1, &user2), 0);
    assert_eq!(token.balance(&user1), 500);
    assert_eq!(token.balance(&user2), 0);

    token.burn(&user1, &500);
    assert_eq!(
        env.auths(),
        std::vec![(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("burn"),
                    (&user1, 500_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.balance(&user2), 0);
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn transfer_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user1 = Address::random(&env);
    let user2 = Address::random(&env);
    let royaltyr = Address::random(&env);
    let royaltyp = 10;

    let token = create_token(&env, &admin, &royaltyr, &royaltyp);
    

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.transfer(&user1, &user2, &1001);
}

#[test]
#[should_panic(expected = "can't receive when deauthorized")]
fn transfer_receive_deauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user1 = Address::random(&env);
    let user2 = Address::random(&env);
    let royaltyr = Address::random(&env);
    let royaltyp = 10;

    let token = create_token(&env, &admin, &royaltyr, &royaltyp);

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.set_authorized(&user2, &false);
    token.transfer(&user1, &user2, &1);
}

#[test]
#[should_panic(expected = "can't spend when deauthorized")]
fn transfer_spend_deauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user1 = Address::random(&env);
    let user2 = Address::random(&env);
    let royaltyr = Address::random(&env);
    let royaltyp = 10;

    let token = create_token(&env, &admin, &royaltyr, &royaltyp);

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.set_authorized(&user1, &false);
    token.transfer(&user1, &user2, &1);
}

#[test]
#[should_panic(expected = "insufficient allowance")]
fn transfer_from_insufficient_allowance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let user1 = Address::random(&env);
    let user2 = Address::random(&env);
    let user3 = Address::random(&env);
    let royaltyr = Address::random(&env);
    let royaltyp = 10;

    let token = create_token(&env, &admin, &royaltyr, &royaltyp);
    

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.approve(&user1, &user3, &100, &200);
    assert_eq!(token.allowance(&user1, &user3), 100);

    token.transfer_from(&user3, &user1, &user2, &101);
}

#[test]
#[should_panic(expected = "already initialized")]
fn initialize_already_initialized() {
    let env = Env::default();
    let admin = Address::random(&env);
    let royaltyr = Address::random(&env);
    let royaltyp = 10;

    let token = create_token(&env, &admin, &royaltyr, &royaltyp);
    

    token.initialize(&admin, &10, &"name".into_val(&env), &"symbol".into_val(&env), &royaltyr, &royaltyp);
}

#[test]
#[should_panic(expected = "Decimal must fit in a u8")]
fn decimal_is_over_max() {
    let env = Env::default();
    let admin = Address::random(&env);
    let royaltyr = Address::random(&env);
    let royaltyp = 10;

    let token = TokenClient::new(&env, &env.register_contract(None, Token {}));
    token.initialize(
        &admin,
        &(u32::from(u8::MAX) + 1),
        &"name".into_val(&env),
        &"symbol".into_val(&env),
        &royaltyr,
        &royaltyp,
    );
}
