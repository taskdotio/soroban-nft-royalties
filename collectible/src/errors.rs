use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SCErrors {
    AlreadyInitialized = 0,
    ItemHasNotBeenMinted = 1,
    ItemNumberIsInvalid = 2,
}
