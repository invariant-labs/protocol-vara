#![no_std]
use contracts::*;
use sails_rtl::{ActorId, Decode, Encode, TypeInfo, Vec};
use math::{
    percentage::Percentage,
    token_amount::TokenAmount,
    types::{liquidity::Liquidity, sqrt_price::SqrtPrice},
};

#[derive(Decode, Encode, TypeInfo, PartialEq, Eq, Clone, Copy, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InvariantConfig {
    pub admin: ActorId,
    pub protocol_fee: Percentage,
}
impl Default for InvariantConfig {
    fn default() -> Self {
        Self {
            admin: ActorId::from(0),
            protocol_fee: Percentage::default(),
        }
    }
}

#[derive(Clone, Decode, Encode, Debug, PartialEq, Eq, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantEvent {
    PositionCreatedEvent {
        timestamp: u64,
        address: ActorId,
        pool_key: PoolKey,
        liquidity_delta: Liquidity,
        lower_tick: i32,
        upper_tick: i32,
        current_sqrt_price: SqrtPrice,
    },
    PositionRemovedEvent {
        timestamp: u64,
        address: ActorId,
        pool_key: PoolKey,
        liquidity: Liquidity,
        lower_tick_index: i32,
        upper_tick_index: i32,
        sqrt_price: SqrtPrice,
    },
    CrossTickEvent {
        timestamp: u64,
        address: ActorId,
        pool_key: PoolKey,
        indexes: Vec<i32>,
    },
    SwapEvent {
        timestamp: u64,
        address: ActorId,
        pool_key: PoolKey,
        amount_in: TokenAmount,
        amount_out: TokenAmount,
        fee: TokenAmount,
        start_sqrt_price: SqrtPrice,
        target_sqrt_price: SqrtPrice,
        x_to_y: bool,
    },
}

#[derive(Decode, Default, Encode, Clone, Debug, PartialEq, Eq, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct CalculateSwapResult {
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub start_sqrt_price: SqrtPrice,
    pub target_sqrt_price: SqrtPrice,
    pub fee: TokenAmount,
    pub pool: Pool,
    pub ticks: Vec<Tick>,
}

#[derive(Decode, Default, Encode, Clone, Debug, PartialEq, Eq, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct SwapHop {
    pub pool_key: PoolKey,
    pub x_to_y: bool,
}

#[derive(Decode, Default, Encode, Clone, Debug, PartialEq, Eq, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct QuoteResult {
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub target_sqrt_price: SqrtPrice,
    pub ticks: Vec<Tick>,
}
