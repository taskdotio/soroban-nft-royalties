#![cfg(test)]
extern crate std;

use crate::{token, RoyaltyOfferClient};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, Symbol,
};

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::AdminClient<'a>) {
    let addr = env.register_stellar_asset_contract(admin.clone());
    (
        token::Client::new(env, &addr),
        token::AdminClient::new(env, &addr),
    )
}

fn create_single_offer_contract<'a>(
    env: &Env,
    seller: &Address,
    sell_token: &Address,
    buy_token: &Address,
    sell_price: u32,
    buy_price: u32,
) -> RoyaltyOfferClient<'a> {
    let offer = RoyaltyOfferClient::new(env, &env.register_contract(None, crate::RoyaltyOffer {}));
    offer.create(seller, sell_token, buy_token, &sell_price, &buy_price);

    // Verify that authorization is required for the seller.
    assert_eq!(
        env.auths(),
        std::vec![(
            seller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    offer.address.clone(),
                    symbol_short!("create"),
                    (
                        seller,
                        sell_token.clone(),
                        buy_token.clone(),
                        sell_price,
                        buy_price
                    )
                        .into_val(env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    offer
}

#[test]
fn test() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::random(&env);
    let seller = Address::random(&env);
    let buyer = Address::random(&env);

    let sell_token = create_token_contract(&env, &token_admin);
    let sell_token_client = sell_token.0;
    let sell_token_admin_client = sell_token.1;

    let buy_token = create_token_contract(&env, &token_admin);
    let buy_token_client = buy_token.0;
    let buy_token_admin_client = buy_token.1;

    // The price here is 1 sell_token for 2 buy_token.
    let offer = create_single_offer_contract(
        &env,
        &seller,
        &sell_token_client.address,
        &buy_token_client.address,
        1,
        2,
    );
    // Give some sell_token to seller and buy_token to buyer.
    sell_token_admin_client.mint(&seller, &1000);
    buy_token_admin_client.mint(&buyer, &1000);
    // Deposit 100 sell_token from seller into offer.
    sell_token_client.transfer(&seller, &offer.address, &100);

    // Try trading 20 buy_token for at least 11 sell_token - that wouldn't
    // succeed because the offer price would result in 10 sell_token.
    assert!(offer.try_trade(&buyer, &20_i128, &11_i128).is_err());
    // Buyer trades 20 buy_token for 10 sell_token.
    offer.trade(&buyer, &20_i128, &10_i128);
    // Verify that authorization is required for the buyer.
    assert_eq!(
        env.auths(),
        std::vec![(
            buyer.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    offer.address.clone(),
                    symbol_short!("trade"),
                    (&buyer, 20_i128, 10_i128).into_val(&env)
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        buy_token_client.address.clone(),
                        symbol_short!("transfer"),
                        (buyer.clone(), &offer.address, 20_i128).into_val(&env)
                    )),
                    sub_invocations: std::vec![]
                }]
            }
        )]
    );

    assert_eq!(sell_token_client.balance(&seller), 900);
    assert_eq!(sell_token_client.balance(&buyer), 10);
    assert_eq!(sell_token_client.balance(&offer.address), 90);
    assert_eq!(buy_token_client.balance(&seller), 20);
    assert_eq!(buy_token_client.balance(&buyer), 980);
    assert_eq!(buy_token_client.balance(&offer.address), 0);

    // Withdraw 70 sell_token from offer.
    offer.withdraw(&sell_token_client.address, &70);
    // Verify that the seller has to authorize this.
    assert_eq!(
        env.auths(),
        std::vec![(
            seller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    offer.address.clone(),
                    symbol_short!("withdraw"),
                    (sell_token_client.address.clone(), 70_i128).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(sell_token_client.balance(&seller), 970);
    assert_eq!(sell_token_client.balance(&offer.address), 20);

    // The price here is 1 sell_token = 1 buy_token.
    offer.updt_price(&1, &1);
    // Verify that the seller has to authorize this.
    assert_eq!(
        env.auths(),
        std::vec![(
            seller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    offer.address.clone(),
                    Symbol::new(&env, "updt_price"),
                    (1_u32, 1_u32).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    // Buyer trades 10 buy_token for 10 sell_token.
    offer.trade(&buyer, &10_i128, &9_i128);
    assert_eq!(sell_token_client.balance(&seller), 970);
    assert_eq!(sell_token_client.balance(&buyer), 20);
    assert_eq!(sell_token_client.balance(&offer.address), 10);
    assert_eq!(buy_token_client.balance(&seller), 30);
    assert_eq!(buy_token_client.balance(&buyer), 970);
    assert_eq!(buy_token_client.balance(&offer.address), 0);
}