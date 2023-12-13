#![cfg(test)]

use crate::storage::items::Item;
use crate::tests::test_utils::{create_test_data, init_with_test_data, TestData};
use soroban_sdk::arbitrary::std;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{symbol_short, Address, Env, IntoVal};

#[test]
pub fn test_transferring_ownership() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);
    init_with_test_data(&test_data);

    let buyer: Address = Address::random(&env);
    test_data
        .usd_token_admin_client
        .mint(&buyer, &(test_data.initial_price as i128));

    test_data.contract_client.buy(&buyer, &5);

    let mut item: Item = test_data.contract_client.item(&5);

    assert_eq!(&item.owner, &buyer);

    // Let's just test for the sake of it that if we had it as an offer it will get reset once is transferred
    test_data.contract_client.sell(&5, &50_0000000);

    let new_owner: Address = Address::random(&env);

    test_data.contract_client.transfer(&5, &new_owner);

    item = test_data.contract_client.item(&5);

    assert_eq!(&item.owner, &new_owner);
}

#[test]
pub fn test_transferring_by_minting() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);
    init_with_test_data(&test_data);

    let recipient: Address = Address::random(&env);

    assert_eq!(&test_data.contract_client.balance(&recipient), &0);

    test_data.contract_client.mint(&10, &recipient);

    assert_eq!(
        env.auths().last().unwrap(),
        &(
            test_data.initial_seller,
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    test_data.contract_client.address.clone(),
                    symbol_short!("mint"),
                    (10u64, recipient.clone()).into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        )
    );

    assert_eq!(&test_data.contract_client.balance(&recipient), &1);

    let item: Item = test_data.contract_client.item(&10);

    assert_eq!(&item.owner, &recipient);
    assert_eq!(&item.for_sale, &false);
    assert_eq!(&item.number, &10);
    assert_eq!(&item.price, &0);
}
