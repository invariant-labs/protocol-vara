#![no_std]

use gmeta::{In, InOut, Metadata};
use gstd::{Decode, Encode, ActorId, TypeInfo, Vec};
use math::types::{sqrt_price::SqrtPrice, liquidity::Liquidity};
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
    RemoveFeeTier(FeeTier),
    CreatePool{
        token_0: ActorId,
        token_1: ActorId,
        fee_tier: FeeTier,
        init_sqrt_price: SqrtPrice,
        init_tick: i32,
    },
    ChangeFeeReceiver(PoolKey, ActorId),
    CreatePosition {
        pool_key: PoolKey,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_delta: Liquidity,
        slippage_limit_lower: SqrtPrice,
        slippage_limit_upper: SqrtPrice,
    }
}

#[derive(Clone, Decode, Encode, Debug, PartialEq, Eq, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantEvent {
    ProtocolFeeChanged(u128),
    PositionCreated(Position),
    ActionFailed(InvariantError),
}

#[derive(Clone, Decode, Encode, Debug, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantStateQuery {
    FeeTierExist(FeeTier),
    GetFeeTiers,
    GetProtocolFee,
    GetPool(ActorId, ActorId, FeeTier),
    GetPools(u8, u16),
    GetPosition(ActorId, u32),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantState {
    ProtocolFee(u128),
    QueriedFeeTiers(Vec<FeeTier>),
    FeeTierExist(bool),
    Pool(Pool),
    Pools(Vec<PoolKey>),
    Position(Position),
    QueryFailed(InvariantError),
}

