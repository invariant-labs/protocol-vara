use decimal::*;

#[allow(unused_imports)]
use gstd::ToString;
use gstd::{Decode, Encode, TypeInfo};

#[decimal(0, U512)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct TokenAmount(pub U256);
