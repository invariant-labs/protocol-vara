use decimal::*;
use alloc::string::ToString;

#[decimal(12)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct FixedPoint(pub u128);
