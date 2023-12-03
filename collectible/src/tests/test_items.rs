#![cfg(test)]

use crate::errors::SCErrors;
use crate::tests::test_utils::{create_test_data, init_with_test_data, TestData};
use soroban_sdk::Env;

#[test]
pub fn test_getting_item_doesnt_exist_and_invalid_number() {
    let env: Env = Env::default();
    let test_data: TestData = create_test_data(&env);
    init_with_test_data(&test_data);

    let non_minted_error = test_data.contract_client.try_item(&0).unwrap_err().unwrap();

    assert_eq!(non_minted_error, SCErrors::ItemHasNotBeenMinted.into());

    let invalid_number_error = test_data
        .contract_client
        .try_item(&(test_data.supply + 100))
        .unwrap_err()
        .unwrap();

    assert_eq!(invalid_number_error, SCErrors::ItemNumberIsInvalid.into());
}
