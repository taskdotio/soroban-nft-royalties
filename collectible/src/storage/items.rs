use soroban_sdk::{contracttype, Address};

/// Each collectible contract has Items, each Item has only one owner while a user can have many Items.

#[contracttype]
#[derive(Debug, Clone)]
pub struct Item {
    pub number: u64,
    pub owner: Address,
    pub for_sale: bool,
    pub price: u128,
}

#[contracttype]
pub enum ItemsDataKeys {
    Item(u64),
}
