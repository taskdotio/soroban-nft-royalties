#![cfg(test)]

use crate::tests::test_utils::{create_test_data, init_with_test_data, TestData};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

#[test]
pub fn test_zero_balances() {
    let env: Env = Env::default();
    let test_data: TestData = create_test_data(&env);
    init_with_test_data(&test_data);

    let address: Address = Address::random(&env);

    assert_eq!(0u128, test_data.contract_client.balance(&address));
}
