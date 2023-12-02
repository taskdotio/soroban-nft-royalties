use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone)]
pub struct Royalty {
    pub name: String,
    pub address: Address,
    pub percentage: u128,
    pub first_sale: bool,
}

#[contracttype]
pub enum RoyaltiesDataKeys {
    /// The Royalties defined for the collectible, this returns a Map<Royalty>
    /// This value isn't kept as part of the instance storage and instead is defined as permanent
    Royalties,
}
