use soroban_sdk::{contracttype, Address, String};

#[contracttype]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,

    /// The Asset Metadata is a URI where the metadata is hosted, this returns an String
    pub metadata_uri: String,
}

#[contracttype]
pub struct CoreData {
    pub admin: Address,

    /// Once minted, there won't be more than the specified supply. Supply is a u64 value
    pub supply: u64,

    /// The initial price is set at the initialization of the contract along with the accepted asset,
    /// once set this can not be changed.
    pub initial_price: u128,
    pub collection_currency: Address,
    pub initial_seller: Address,
}

#[contracttype]
pub enum CoreDataKeys {
    CoreData,

    /// The Token Metadata is compatible with the metadata defined from the soroban-token-sdk
    TokenMetadata,
}
