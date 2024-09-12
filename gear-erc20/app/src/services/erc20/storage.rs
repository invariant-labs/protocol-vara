// TODO (sails): impl such macro
use super::utils::{AllowancesMap, BalancesMap};
use gstd::String;
use sails_rs::U256;

crate::declare_storage!(module: allowances, name: AllowancesStorage, ty: AllowancesMap);

impl AllowancesStorage {
    pub fn with_capacity(capacity: usize) -> Result<(), AllowancesMap> {
        Self::set(AllowancesMap::with_capacity(capacity))
    }

    pub fn default() -> Result<(), AllowancesMap> {
        Self::with_capacity(u16::MAX as usize)
    }
}

crate::declare_storage!(module: balances, name: BalancesStorage, ty: BalancesMap);

impl BalancesStorage {
    pub fn with_capacity(capacity: usize) -> Result<(), BalancesMap> {
        Self::set(BalancesMap::with_capacity(capacity))
    }

    pub fn default() -> Result<(), BalancesMap> {
        Self::with_capacity(u16::MAX as usize)
    }
}

pub struct Meta {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

crate::declare_storage!(module: meta, name: MetaStorage, ty: Meta);

impl MetaStorage {
    pub fn with_data(name: String, symbol: String, decimals: u8) -> Result<(), Meta> {
        Self::set(Meta {
            name,
            symbol,
            decimals,
        })
    }

    pub fn default() -> Result<(), Meta> {
        Self::with_data(String::from("Vara Network"), String::from("VARA"), 12)
    }

    pub fn name() -> String {
        Self::as_ref().name.clone()
    }

    pub fn symbol() -> String {
        Self::as_ref().symbol.clone()
    }

    pub fn decimals() -> u8 {
        Self::as_ref().decimals
    }
}

crate::declare_storage!(module: total_supply, name: TotalSupplyStorage, ty: U256);

impl TotalSupplyStorage {
    pub fn default() -> Result<(), U256> {
        Self::set(U256::zero())
    }

    pub fn get() -> U256 {
        *Self::as_ref()
    }
}

#[cfg(feature = "test")]
crate::declare_storage!(module: test_utils, name: TransferFailStorage, ty: bool);

#[cfg(feature = "test")]
impl TransferFailStorage {
    pub fn default() -> Result<(), bool> {
        Self::set(false)
    }

    pub fn get() -> bool {
        *Self::as_ref()
    }
}
