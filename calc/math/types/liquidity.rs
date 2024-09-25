use decimal::*;
use sails_rs::prelude::*;

#[decimal(5, U512)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Encode, Decode, TypeInfo)]
pub struct Liquidity(pub U256);
