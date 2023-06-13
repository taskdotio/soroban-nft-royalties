#![no_std]
use soroban_sdk::{contracterror, contractimpl, contracttype, token, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    ContractAlreadyInitialized = 1,
    ContractNotInitialized = 2,
    InvalidAuth = 3,
    InvalidArguments = 4,
}

#[contracttype]
#[derive(Clone)]
pub enum StorageKey {
    Issuer,            // Address
    Asset,       // Address
    Recipient,        // Address
    Pct, // u32
}

pub struct RoyaltyIndexContract;
pub trait RoyaltyIndexTrait {
    fn set(e: Env, issuer: Address, asset: Address, recipient: Address, pct: u32) -> Result<(), Error>;
    fn get(e: Env) -> Result<(Address, Address, Address, u32), Error>;
}

#[contractimpl]
impl RoyaltyIndexTrait for RoyaltyIndexContract {

    fn set(
            e: Env,
            issuer: Address, //the issuer of the token
            asset: Address, // the id of the sac for the token
            recipient: Address, // the recipient of the royalty
            pct: u32, // the percentage of the royalty
    ) -> Result<(), Error> {
    let assetkey = StorageKey::Asset;
    if e.storage().has(&assetkey) {
        return Err(Error::ContractAlreadyInitialized);
    }

    issuer.require_auth();
    //verify the inputs here
    e.storage().set(&StorageKey::Issuer, &issuer);
    e.storage().set(&StorageKey::Asset, &asset);
    e.storage().set(&StorageKey::Recipient, &recipient);
    e.storage().set(&StorageKey::Pct, &pct);
    Ok(())
    }

    fn get(e: Env) -> Result<(Address, Address, Address, u32), Error> {
        let assetkey = StorageKey::Asset;
        if !e.storage().has(&assetkey) {
            return Err(Error::ContractNotInitialized);
        }

        let issuer: Address = e.storage().get(&StorageKey::Issuer).unwrap().unwrap();
        let asset: Address = e.storage().get(&StorageKey::Asset).unwrap().unwrap();
        let recipient: Address = e.storage().get(&StorageKey::Recipient).unwrap().unwrap();
        let pct: u32 = e.storage().get(&StorageKey::Pct).unwrap().unwrap();
        Ok((issuer, asset, recipient, pct))
    }

}
