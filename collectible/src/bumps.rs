/// We define all cases (instance, royalties, balances, etc) separated even doe they use the same value just in case we would like to change this in the future

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_CONSTANT: u32 = DAY_IN_LEDGERS * 28;
pub(crate) const INSTANCE_BUMP_CONSTANT_THRESHOLD: u32 = DAY_IN_LEDGERS * 14;

pub(crate) const ROYALTIES_BUMP_CONSTANT: u32 = DAY_IN_LEDGERS * 28;
pub(crate) const ROYALTIES_BUMP_CONSTANT_THRESHOLD: u32 = DAY_IN_LEDGERS * 14;

pub(crate) const BALANCES_BUMP_CONSTANT: u32 = DAY_IN_LEDGERS * 28;
pub(crate) const BALANCES_BUMP_CONSTANT_THRESHOLD: u32 = DAY_IN_LEDGERS * 14;

pub(crate) const ITEMS_BUMP_CONSTANT: u32 = DAY_IN_LEDGERS * 28;
pub(crate) const ITEMS_BUMP_CONSTANT_THRESHOLD: u32 = DAY_IN_LEDGERS * 14;
