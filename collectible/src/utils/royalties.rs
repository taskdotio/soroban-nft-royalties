use crate::bumps::{ROYALTIES_BUMP_CONSTANT, ROYALTIES_BUMP_CONSTANT_THRESHOLD};
use crate::storage::royalties::{RoyaltiesDataKeys, Royalty};
use soroban_sdk::{Env, Vec};

pub fn bump_royalties(env: &Env) {
    env.storage().persistent().extend_ttl(
        &RoyaltiesDataKeys::Royalties,
        ROYALTIES_BUMP_CONSTANT_THRESHOLD,
        ROYALTIES_BUMP_CONSTANT,
    );
}

pub fn write_royalties(env: &Env, royalties: &Vec<Royalty>) {
    env.storage()
        .persistent()
        .set(&RoyaltiesDataKeys::Royalties, royalties)
}

pub fn get_royalties(env: &Env) -> Vec<Royalty> {
    env.storage()
        .persistent()
        .get(&RoyaltiesDataKeys::Royalties)
        .unwrap()
}
