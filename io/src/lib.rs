#![no_std]

use contracts::*;
use gmeta::{In, InOut, Metadata};
use gstd::{ActorId, Decode, Encode, TypeInfo, Vec};
use math::{
    percentage::Percentage,
    token_amount::TokenAmount,
    types::{liquidity::Liquidity, sqrt_price::SqrtPrice},
};

pub struct InvariantMetadata;

impl Metadata for InvariantMetadata {
    type Init = In<InitInvariant>;
    type Handle = InOut<InvariantAction, InvariantEvent>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = InOut<InvariantStateQuery, InvariantStateReply>;
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
    pub protocol_fee: Percentage,
}

#[derive(Clone, Decode, Encode, Debug, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantAction {
    ChangeProtocolFee(Percentage),
    AddFeeTier(FeeTier),
    RemoveFeeTier(FeeTier),
    CreatePool {
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
    },
    RemovePosition {
        position_id: u32,
    },
    TransferPosition {
        index: u32,
        receiver: ActorId,
    },
    Swap {
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    },
    Quote {
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    },
    QuoteRoute {
        amount_in: TokenAmount,
        swaps: Vec<SwapHop>,
    },
    ClaimFee {
        position_id: u32,
    },
    WithdrawProtocolFee(PoolKey),
    ClaimLostTokens {
        token: ActorId,
    },
}

#[derive(Clone, Decode, Encode, Debug, PartialEq, Eq, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantEvent {
    ActionFailed(InvariantError),
    ProtocolFeeChanged(Percentage),
    PositionCreatedReturn(Position),
    PositionCreatedEvent {
        block_timestamp: u64,
        address: ActorId,
        pool_key: PoolKey,
        liquidity_delta: Liquidity,
        lower_tick: i32,
        upper_tick: i32,
        current_sqrt_price: SqrtPrice,
    },
    PositionRemovedReturn(TokenAmount, TokenAmount),
    PositionRemovedEvent {
        block_timestamp: u64,
        caller: ActorId,
        pool_key: PoolKey,
        liquidity: Liquidity,
        lower_tick_index: i32,
        upper_tick_index: i32,
        sqrt_price: SqrtPrice,
    },
    CrossTickEvent {
        timestamp: u64,
        address: ActorId,
        pool: PoolKey,
        indexes: Vec<i32>,
    },
    SwapEvent {
        timestamp: u64,
        address: ActorId,
        pool: PoolKey,
        amount_in: TokenAmount,
        amount_out: TokenAmount,
        fee: TokenAmount,
        start_sqrt_price: SqrtPrice,
        target_sqrt_price: SqrtPrice,
        x_to_y: bool,
    },
    SwapReturn(CalculateSwapResult),
    Quote(QuoteResult),
    QuoteRoute(TokenAmount),
    ClaimFee(TokenAmount, TokenAmount),
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
    GetTick(PoolKey, i32),
    IsTickInitialized(PoolKey, i32),
    GetAllPositions(ActorId),
    GetUserBalances(ActorId),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum InvariantStateReply {
    QueryFailed(InvariantError),
    ProtocolFee(Percentage),
    QueriedFeeTiers(Vec<FeeTier>),
    FeeTierExist(bool),
    Pool(Pool),
    Pools(Vec<PoolKey>),
    Position(Position),
    Positions(Vec<Position>),
    Tick(Tick),
    IsTickInitialized(bool),
    UserBalances(Vec<(ActorId, TokenAmount)>),
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
