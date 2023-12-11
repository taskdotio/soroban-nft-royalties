#![cfg(test)]

use crate::tests::test_utils::{create_test_data, init_with_test_data, TestData};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

#[test]
pub fn test_royalties_and_payments() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);
    init_with_test_data(&test_data);

    let buyer: Address = Address::generate(&env);
    test_data
        .usd_token_admin_client
        .mint(&buyer, &(test_data.initial_price as i128));

    test_data.contract_client.buy(&buyer, &0);

    assert_eq!(
        test_data
            .usd_token_client
            .balance(&test_data.platform_royalty.address),
        0_1999000
    );
    assert_eq!(
        test_data
            .usd_token_client
            .balance(&test_data.creator_royalty.address),
        0_5997000
    );
    assert_eq!(
        test_data
            .usd_token_client
            .balance(&test_data.charity_royalty.address),
        0_3998000
    );
    assert_eq!(
        test_data
            .usd_token_client
            .balance(&test_data.initial_seller) as u128,
        test_data.initial_price - 0_1999000 - 0_5997000 - 0_3998000
    );

    test_data.contract_client.sell(&0, &50_0000000);

    let new_buyer: Address = Address::generate(&env);

    test_data
        .usd_token_admin_client
        .mint(&new_buyer, &50_0000000);

    test_data.contract_client.buy(&new_buyer, &0);

    // Platform doesn't receive more funds because we set it as a first sale royalty.
    // It doesn't receive royalties for second sales
    assert_eq!(
        test_data
            .usd_token_client
            .balance(&test_data.platform_royalty.address),
        0_1999000
    );
    assert_eq!(
        test_data
            .usd_token_client
            .balance(&test_data.creator_royalty.address),
        0_5997000 + 1_5000000
    );
    assert_eq!(
        test_data
            .usd_token_client
            .balance(&test_data.charity_royalty.address),
        0_3998000 + 1_0000000
    );
    assert_eq!(
        test_data
            .usd_token_client
            .balance(&test_data.initial_seller) as u128,
        test_data.initial_price - 0_1999000 - 0_5997000 - 0_3998000
    );
}
