use decimal::*;
use gstd::*;

#[decimal(28)]
// #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
// #[derive(Default, TypeInfo, Clone, Decode, Encode)]
#[derive(Default, Debug, Clone, Copy)]
pub struct SqrtPrice {
    pub v: u128
}