use soroban_sdk::{contract, contractimpl, Env};

pub trait CollectibleTrait {
    fn init(env: Env);
}

#[contract]
pub struct CollectibleContract;

#[contractimpl]
impl CollectibleTrait for CollectibleContract {
    fn init(env: Env) {}
}
