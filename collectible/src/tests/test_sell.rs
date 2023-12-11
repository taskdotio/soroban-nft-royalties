#![cfg(test)]

use crate::errors::SCErrors;
use crate::storage::items::Item;
use crate::tests::test_utils::{create_test_data, init_with_test_data, TestData};
use soroban_sdk::testutils::arbitrary::std;
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation};
use soroban_sdk::{Address, Env, IntoVal, Symbol};

#[test]
pub fn test_selling_product_doesnt_exist() {
    let env: Env = Env::default();
    env.mock_all_auths();
    let test_data: TestData = create_test_data(&env);
    init_with_test_data(&test_data);

    let does_not_exist_error = test_data
        .contract_client
        .try_sell(&0, &100_0000000)
        .unwrap_err()
        .unwrap();

    assert_eq!(does_not_exist_error, SCErrors::ItemHasNotBeenMinted.into());
}

#[test]
pub fn test_sell_logic() {
    let env: Env = Env::default();
    env.mock_all_auths();
    let test_data: TestData = create_test_data(&env);
    init_with_test_data(&test_data);

    let owner: Address = Address::generate(&env);

    test_data
        .usd_token_admin_client
        .mint(&owner, &(test_data.initial_price as i128));

    test_data.contract_client.buy(&owner, &0);

    let mut item: Item = test_data.contract_client.item(&0);

    assert_eq!(&item.owner, &owner);
    assert_eq!(&item.price, &0);
    assert_eq!(&item.for_sale, &false);
    assert_eq!(&item.number, &0);

    test_data.contract_client.sell(&0, &50_0000000);

    assert_eq!(
        env.auths().first().unwrap(),
        &(
            owner.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    test_data.contract_client.address.clone(),
                    Symbol::new(&env, "sell"),
                    (0u64, 50_0000000u128,).into_val(&env)
                )),
                sub_invocations: std::vec![],
            }
        )
    );

    item = test_data.contract_client.item(&0);
    assert_eq!(&item.owner, &owner);
    assert_eq!(&item.price, &50_0000000u128);
    assert_eq!(&item.for_sale, &true);
    assert_eq!(&item.number, &0);

    test_data.contract_client.sell(&0, &0);

    item = test_data.contract_client.item(&0);
    assert_eq!(&item.owner, &owner);
    assert_eq!(&item.price, &0);
    assert_eq!(&item.for_sale, &false);
    assert_eq!(&item.number, &0);
}
