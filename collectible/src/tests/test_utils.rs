#![cfg(test)]

use crate::contract::{CollectibleContract, CollectibleContractClient};
use crate::storage::core::TokenMetadata;
use crate::storage::royalties::Royalty;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, Address, Env, String, Vec};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

fn create_token_contract<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        TokenClient::new(e, &contract_address),
        TokenAdminClient::new(e, &contract_address),
    )
}

pub struct TestData<'a> {
    pub admin: Address,
    pub supply: u64,
    pub initial_price: u128,
    pub initial_seller: Address,
    pub token_metadata: TokenMetadata,
    pub default_royalties: Vec<Royalty>,
    pub platform_royalty: Royalty,
    pub creator_royalty: Royalty,
    pub charity_royalty: Royalty,

    pub usd_token_admin: Address,
    pub usd_token_client: TokenClient<'a>,
    pub usd_token_admin_client: TokenAdminClient<'a>,
    pub eur_token_admin: Address,
    pub eur_token_client: TokenClient<'a>,
    pub eur_token_admin_client: TokenAdminClient<'a>,

    pub contract_client: CollectibleContractClient<'a>,
}

pub fn create_test_data(env: &Env) -> TestData {
    let admin: Address = Address::random(&env);
    let supply: u64 = 150u64;

    let mut default_royalties: Vec<Royalty> = Vec::new(&env);
    let platform_royalty: Royalty = Royalty {
        name: String::from_slice(&env, "ThePlatform"),
        address: Address::random(&env),
        first_sale: true,
        percentage: 0_0100000,
    };
    let creator_royalty: Royalty = Royalty {
        name: String::from_slice(&env, "TheCreator"),
        address: Address::random(&env),
        first_sale: false,
        percentage: 0_0300000,
    };
    let charity_royalty: Royalty = Royalty {
        name: String::from_slice(&env, "TheCharity"),
        address: Address::random(&env),
        first_sale: false,
        percentage: 0_0200000,
    };

    default_royalties.push_back(platform_royalty.clone());
    default_royalties.push_back(creator_royalty.clone());
    default_royalties.push_back(charity_royalty.clone());

    let usd_token_admin: Address = Address::random(&env);
    let (usd_token_client, usd_token_admin_client) = create_token_contract(&env, &usd_token_admin);
    let eur_token_admin: Address = Address::random(&env);
    let (eur_token_client, eur_token_admin_client) = create_token_contract(&env, &eur_token_admin);

    let contract_client =
        CollectibleContractClient::new(&env, &env.register_contract(None, CollectibleContract));

    let initial_price: u128 = 19_9900000u128;
    let initial_seller: Address = Address::random(&env);

    TestData {
        admin,
        supply,
        initial_price,
        initial_seller,
        token_metadata: TokenMetadata {
            name: String::from_slice(&env, "GoldMiners"),
            symbol: String::from_slice(&env, "GMS"),
            metadata_uri: String::from_slice(&env, "https://kjgutsr.dfghuexvhj.net/userdata/GDVT45B2WLFKQS3XB5MUYHV3WCGEX5W2QPDLBOAIPC3MWHATI34VOULF.jpg"),
        },
        default_royalties,
        platform_royalty,
        creator_royalty,
        charity_royalty,
        usd_token_admin,
        usd_token_client,
        usd_token_admin_client,
        eur_token_admin,
        eur_token_client,
        eur_token_admin_client,
        contract_client,
    }
}

pub fn init_with_test_data(test_data: &TestData) {
    test_data.contract_client.init(
        &test_data.admin,
        &test_data.supply,
        &test_data.initial_price,
        &test_data.initial_seller,
        &test_data.usd_token_client.address,
        &test_data.token_metadata.name,
        &test_data.token_metadata.symbol,
        &test_data.token_metadata.metadata_uri,
        &test_data.default_royalties,
    );
}
