#![cfg(test)]

use crate::errors::SCErrors;
use crate::tests::test_utils::{create_test_data, TestData};
use soroban_sdk::Env;

#[test]
pub fn test_init() {
    let env: Env = Env::default();
    env.mock_all_auths();

    let test_data: TestData = create_test_data(&env);

    test_data.contract_client.init(
        &test_data.admin,
        &test_data.supply,
        &test_data.initial_price,
        &test_data.initial_seller,
        &test_data.usd_token_client.address,
        &test_data.token_metadata.name,
        &test_data.token_metadata.symbol,
        &test_data.token_metadata.metadata_uri,
        &test_data.default_royalties,
    );

    assert_eq!(test_data.contract_client.supply(), test_data.supply);

    let already_initiated_error = test_data
        .contract_client
        .try_init(
            &test_data.admin,
            &test_data.supply,
            &test_data.initial_price,
            &test_data.initial_seller,
            &test_data.usd_token_client.address,
            &test_data.token_metadata.name,
            &test_data.token_metadata.symbol,
            &test_data.token_metadata.metadata_uri,
            &test_data.default_royalties,
        )
        .unwrap_err()
        .unwrap();

    assert_eq!(already_initiated_error, SCErrors::AlreadyInitialized.into());
}
