use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum BalancesDataKeys {
    Balance(Address),
}
