#![allow(dead_code)]
#![allow(unused_imports)]

pub mod allowance;
pub mod balance_of;
pub mod burn;
pub mod increase_allowance;
pub mod init_tokens;
pub mod init_tokens_with_mint;
pub mod mint;
pub mod set_transfer_fail;

pub use allowance::*;
pub use balance_of::*;
pub use burn::*;
pub use increase_allowance::*;
pub use init_tokens::*;
pub use init_tokens_with_mint::*;
pub use mint::*;
pub use set_transfer_fail::*;

use gstd::*;
// temporary workaround for U256
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub(self) struct U256(pub u128, pub u128);

impl From<u128> for U256 {
    fn from(num: u128) -> U256 {
        U256(num, 0)
    }
}

impl From<U256> for u128 {
    fn from(num: U256) -> u128 {
        num.0
    }
}
