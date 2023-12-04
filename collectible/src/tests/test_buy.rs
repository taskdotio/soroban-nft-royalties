#![cfg(test)]

use crate::errors::SCErrors;
use crate::storage::items::Item;
use crate::tests::test_utils::{create_test_data, init_with_test_data, TestData};
use soroban_sdk::arbitrary::std::println;
use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal};

#[test]
pub fn test_initial_sale_and_invalid_number() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);
    init_with_test_data(&test_data);

    let buyer: Address = Address::random(&env);
    test_data
        .usd_token_admin_client
        .mint(&buyer, &(test_data.initial_price as i128));

    test_data.contract_client.buy(&buyer, &0);

    let item: Item = test_data.contract_client.item(&0);
    let buyer_balance: u128 = test_data.contract_client.balance(&&buyer);

    assert_eq!(buyer_balance, 1);
    assert_eq!(item.owner, buyer);
    assert_eq!(item.number, 0);
    assert_eq!(item.for_sale, false);

    let new_buyer: Address = Address::random(&env);
    let not_for_sale_error = test_data
        .contract_client
        .try_buy(&new_buyer, &(test_data.supply + 100))
        .unwrap_err()
        .unwrap();

    assert_eq!(not_for_sale_error, SCErrors::ItemNumberIsInvalid.into());
}

#[test]
pub fn test_second_sales_and_not_sale() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);
    init_with_test_data(&test_data);

    let buyer: Address = Address::random(&env);
    test_data
        .usd_token_admin_client
        .mint(&buyer, &(test_data.initial_price as i128));

    test_data.contract_client.buy(&buyer, &0);
    let mut item: Item = test_data.contract_client.item(&0);
    assert_eq!(&item.owner, &buyer);

    assert_eq!(test_data.contract_client.balance(&buyer), 1u128);

    let new_buyer: Address = Address::random(&env);
    let not_for_sale_error = test_data
        .contract_client
        .try_buy(&new_buyer, &0)
        .unwrap_err()
        .unwrap();

    assert_eq!(not_for_sale_error, SCErrors::ItemIsNotForSale.into());

    test_data.contract_client.sell(&0, &50_0000000);

    test_data
        .usd_token_admin_client
        .mint(&new_buyer, &50_0000000);

    test_data.contract_client.buy(&new_buyer, &0);
    item = test_data.contract_client.item(&0);
    assert_eq!(&item.owner, &new_buyer);
    assert_eq!(&item.for_sale, &false);
    assert_eq!(&item.price, &0);

    assert_eq!(test_data.contract_client.balance(&buyer), 0u128);
    assert_eq!(test_data.contract_client.balance(&new_buyer), 1u128);
}
