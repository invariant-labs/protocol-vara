use decimal::*;
use sails_rs::prelude::*;
#[decimal(12, U256)]
#[derive(
    Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Encode, Decode, TypeInfo, Hash,
)]
pub struct Percentage(pub u128);
