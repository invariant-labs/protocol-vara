use decimal::*;
#[allow(unused_imports)]
use gstd::ToString;
use gstd::{Decode, Encode, TypeInfo};

#[decimal(12, U256)]
#[derive(
    Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Encode, Decode, TypeInfo, Hash,
)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Percentage(pub u128);
