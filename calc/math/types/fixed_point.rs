use decimal::*;
use sails_rs::prelude::*;

#[decimal(12, U192)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Encode, Decode, TypeInfo)]
pub struct FixedPoint(pub u128);
