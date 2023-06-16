#![no_std]

use soroban_sdk::{contractimpl, Address, BytesN, Env, RawVal, Symbol, Vec, String, TryFromVal, vec};
//use soroban_sdk::env_val::TryFromVal;

pub struct Deployer;

#[contractimpl]
impl Deployer {
    /// Deploy the contract wasm and after deployment invoke the init function
    /// of the contract with the given arguments. Returns the contract ID and
    /// result of the init function.
    pub fn deploy(
        env: Env,
        salt: BytesN<32>,
        wasm_hash: BytesN<32>,
        admin: Address,
        decimal: u32,
        name: String,
        symbol: String,
        royaltyr: Address,
        royaltyp: u32
    ) -> (Address, RawVal) {
        // Convert the arguments to RawVal
        let admin_raw = RawVal::try_from_val(&env, &admin).unwrap();
        let decimal_raw = RawVal::try_from_val(&env, &decimal).unwrap();
        let name_raw = RawVal::try_from_val(&env, &name).unwrap();
        let symbol_raw = RawVal::try_from_val(&env, &symbol).unwrap();
        let royr_raw = RawVal::try_from_val(&env, &royaltyr).unwrap();
        let royp_raw = RawVal::try_from_val(&env, &royaltyp).unwrap();

        // Construct the init_args
        let init_args = vec![&env, admin_raw, decimal_raw, name_raw, symbol_raw, royr_raw, royp_raw];

        // Deploy the contract using the installed WASM code with given hash.
        let id = env
            .deployer()
            .with_current_contract(&salt)
            .deploy(&wasm_hash);
        // Invoke the init function with the given arguments.
        let res: RawVal = env.invoke_contract(&id, &Symbol::new(&env, "initialize"), init_args);
        // Return the contract ID of the deployed contract and the result of
        // invoking the init result.
        (id, res)
    }
}

//mod test;
