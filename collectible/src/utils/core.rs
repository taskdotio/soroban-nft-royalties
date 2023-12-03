use crate::storage::core::{CoreData, CoreDataKeys};
use soroban_sdk::{token, Address, Env};
use soroban_token_sdk::metadata::TokenMetadata;

use crate::bumps::{INSTANCE_BUMP_CONSTANT, INSTANCE_BUMP_CONSTANT_THRESHOLD};

pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .bump(INSTANCE_BUMP_CONSTANT_THRESHOLD, INSTANCE_BUMP_CONSTANT);
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&CoreDataKeys::CoreData)
}

pub fn write_core_data(env: &Env, core_data: &CoreData) {
    env.storage()
        .instance()
        .set(&CoreDataKeys::CoreData, core_data);
}

pub fn get_core_data(env: &Env) -> CoreData {
    env.storage()
        .instance()
        .get(&CoreDataKeys::CoreData)
        .unwrap()
}

pub fn write_token_metadata(env: &Env, token_metadata: TokenMetadata) {
    env.storage()
        .instance()
        .set(&CoreDataKeys::TokenMetadata, &token_metadata);
}

pub fn collection_currency<'a>(env: &Env, core_data: &CoreData) -> token::Client<'a> {
    token::Client::new(&env, &core_data.collection_currency)
}
