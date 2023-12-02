use crate::bumps::{ROYALTIES_BUMP_CONSTANT, ROYALTIES_BUMP_CONSTANT_THRESHOLD};
use crate::storage::royalties::{RoyaltiesDataKeys, Royalty};
use soroban_sdk::{Address, Env, Map};

pub fn bump_royalties(env: &Env) {
    env.storage().persistent().bump(
        &RoyaltiesDataKeys::Royalties,
        ROYALTIES_BUMP_CONSTANT_THRESHOLD,
        ROYALTIES_BUMP_CONSTANT,
    );
}

pub fn write_royalties(env: &Env, royalties: &Map<Address, Royalty>) {
    env.storage()
        .persistent()
        .set(&RoyaltiesDataKeys::Royalties, royalties)
}

pub fn get_royalties(env: &Env) -> Map<Address, Royalty> {
    env.storage()
        .persistent()
        .get::<RoyaltiesDataKeys, Map<Address, Royalty>>(&RoyaltiesDataKeys::Royalties)
        .unwrap()
}
