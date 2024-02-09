use gmeta::{In, InOut, Metadata};
use gstd::ActorId;
use scale::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Decode, Encode, TypeInfo, PartialEq, Eq, Clone, Copy, Debug, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct FeeTier {
    pub admin: ActorId,
    pub protocol_fee: u128,
}
