use decimal::*;
use gstd::{Decode, Encode, TypeInfo};

#[decimal(12)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct FixedPoint(pub u128);
