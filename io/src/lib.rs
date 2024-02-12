<<<<<<< HEAD
use gmeta::Metadata;
pub mod collections;
pub mod storage;
=======
#![no_std]
>>>>>>> 2a63e3ff5b4a5afeb4e53932685ee506cf310de9

use gmeta::{In, InOut, Metadata};
use gstd::ActorId;
use parity_scale_codec::{Decode, Encode};
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
pub struct InitInvariant {
    pub config: InvariantConfig,
}

#[derive(Decode, Encode, TypeInfo, PartialEq, Eq, Clone, Copy, Debug, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InvariantConfig {
    pub admin: ActorId,
    pub protocol_fee: u128,
}

#[derive(Clone, Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantAction {
    ChangeProtocolFee(u128),
}

#[derive(Clone, Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantEvent {
    ProtocolFeeChanged(u128),
}
