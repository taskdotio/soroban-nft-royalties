use crate::errors::SCErrors;
use crate::storage::core::CoreData;
use crate::storage::royalties::Royalty;
use crate::utils::balances::{bump_balance, get_balance};
use crate::utils::core::{bump_instance, is_initialized, write_core_data, write_token_metadata};
use crate::utils::royalties::{bump_royalties, write_royalties};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Map, String};
use soroban_token_sdk::metadata::TokenMetadata;

pub trait CollectibleTrait {
    fn initialize(
        env: Env,
        admin: Address,
        supply: u64,
        initial_price: u128,
        initial_asset: Address,
        name: String,
        symbol: String,
        royalties: Map<Address, Royalty>,
    );

    fn balance(env: Env, id: Address) -> u128;
}

#[contract]
pub struct CollectibleContract;

#[contractimpl]
impl CollectibleTrait for CollectibleContract {
    fn initialize(
        env: Env,
        admin: Address,
        supply: u64,
        initial_price: u128,
        initial_asset: Address,
        name: String,
        symbol: String,
        royalties: Map<Address, Royalty>,
    ) {
        if is_initialized(&env) {
            panic_with_error!(&env, &SCErrors::AlreadyInitialized);
        }

        write_core_data(
            &env,
            &CoreData {
                admin,
                supply,
                initial_price,
                initial_asset,
            },
        );

        write_token_metadata(
            &env,
            TokenMetadata {
                decimal: 0,
                name,
                symbol,
            },
        );

        write_royalties(&env, &royalties);

        bump_instance(&env);
        bump_royalties(&env);
    }

    fn balance(env: Env, id: Address) -> u128 {
        bump_balance(&env, &id);
        get_balance(&env, &id)
    }
}
