use decimal::*;
use sails_rs::prelude::*;

#[decimal(0, U512)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Encode, Decode, TypeInfo)]
pub struct TokenAmount(pub U256);
