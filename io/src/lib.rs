#![no_std]

use gmeta::{In, InOut, Metadata};
use gstd::{Decode, Encode, ActorId, TypeInfo, Vec};
pub struct InvariantMetadata;
use contracts::*;

impl Metadata for InvariantMetadata {
    type Init = In<InitInvariant>;
    type Handle = InOut<InvariantAction, InvariantEvent>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = InOut<InvariantStateQuery, InvariantState>;
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

#[derive(Clone, Decode, Encode, Debug, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantAction {
    ChangeProtocolFee(u128),
    AddFeeTier(FeeTier),
    RemoveFeeTier(FeeTier)
}

#[derive(Clone, Decode, Encode, Debug, PartialEq, Eq, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantEvent {
    ProtocolFeeChanged(u128),
    ActionFailed(InvariantError),
}

#[derive(Clone, Decode, Encode, Debug, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantStateQuery {
    FeeTierExist(FeeTier),
    GetFeeTiers,
    GetProtocolFee
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantState {
    ProtocolFee(u128),
    QueriedFeeTiers(Vec<FeeTier>),
    FeeTierExist(bool)
}

