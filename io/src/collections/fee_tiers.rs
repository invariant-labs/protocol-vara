use gstd::scale_info;
use scale::{Encode, Decode};
use scale_info::TypeInfo;
use gstd::Vec;
use crate::storage::fee_tier::FeeTier;

#[derive(Decode, Encode, TypeInfo, PartialEq, Eq, Clone, Debug, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct FeeTiers {
    fee_tiers: Vec<FeeTier>
}