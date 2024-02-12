#![no_std]
pub mod collections;
pub mod storage;


use gmeta::{In, InOut, Metadata};
use gstd::{scale_info, ActorId};
use scale::{Encode, Decode};
use scale_info::TypeInfo;

pub struct InvariantMetadata;

impl Metadata for InvariantMetadata {
    type Init = In<InitInvariant>;
    type Handle = InOut<InvariantAction, InvariantEvent>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = ();
}

#[derive(Default, Encode, Decode, Clone, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InvariantState {
    pub config: InvariantConfig,
}

#[derive(Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
#[codec(crate = gstd::codec)]
pub struct InitInvariant {
    pub config: InvariantConfig,
}

#[derive(Decode, Encode, PartialEq, Eq, Clone, Copy, Debug, Default, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InvariantConfig {
    pub admin: ActorId,
    pub protocol_fee: u128,
}

#[derive(Clone, Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo)
)]
pub enum InvariantAction {
    ChangeProtocolFee(u128),
}

#[derive(Clone, Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo)
)]
pub enum InvariantEvent {
    ProtocolFeeChanged(u128),
}
