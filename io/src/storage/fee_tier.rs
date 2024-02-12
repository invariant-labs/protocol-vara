use gstd::scale_info;
use scale::{Encode, Decode};
use scale_info::TypeInfo;

#[derive(Decode, Encode, TypeInfo, PartialEq, Eq, Clone, Copy, Debug, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct FeeTier {
    pub fee: u64,
    pub tick_spacing: u16,
}
