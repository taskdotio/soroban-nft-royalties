#![cfg(test)]

use crate::storage::items::Item;
use crate::tests::test_utils::{create_test_data, init_with_test_data, TestData};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

#[test]
pub fn test_transferring_ownership() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);
    init_with_test_data(&test_data);

    let buyer: Address = Address::generate(&env);
    test_data
        .usd_token_admin_client
        .mint(&buyer, &(test_data.initial_price as i128));

    test_data.contract_client.buy(&buyer, &5);

    let mut item: Item = test_data.contract_client.item(&5);

    assert_eq!(&item.owner, &buyer);

    // Let's just test for the sake of it that if we had it as an offer it will get reset once is transferred
    test_data.contract_client.sell(&5, &50_0000000);

    let new_owner: Address = Address::generate(&env);

    test_data.contract_client.transfer(&5, &new_owner);

    item = test_data.contract_client.item(&5);

    assert_eq!(&item.owner, &new_owner);
}
