use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SCErrors {
    UnexpectedError = 0,
    AlreadyInitialized = 1,
    ItemHasNotBeenMinted = 2,
    ItemNumberIsInvalid = 3,
    ItemIsNotForSale = 4,
    ItemWasAlreadyMinted = 5,
}
