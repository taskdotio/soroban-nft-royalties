use soroban_sdk::{contracttype, Address};

#[contracttype]
pub struct CoreData {
    pub admin: Address,

    /// Once minted, there won't be more than the specified supply. Supply is a u64 value
    pub supply: u64,

    /// The initial price is set at the initialization of the contract along with the accepted asset,
    /// once set this can not be changed.
    pub initial_price: u128,
    pub initial_asset: Address,
}

#[contracttype]
pub enum CoreDataKeys {
    CoreData,

    /// The Token Metadata is the metadata defined from the soroban-token-sdk
    TokenMetadata,

    /// The Asset Metadata is a URI where the metadata is hosted, this returns an String
    AssetMetadataUri,
}
