use gstd::scale_info;
use scale::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Clone, Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum InvariantError {
    NotAdmin,
    NotFeeReceiver,
    PoolAlreadyExist,
    PoolNotFound,
    TickAlreadyExist,
    InvalidTickIndexOrTickSpacing,
    PositionNotFound,
    TickNotFound,
    FeeTierNotFound,
    PoolKeyNotFound,
    AmountIsZero,
    WrongLimit,
    PriceLimitReached,
    NoGainSwap,
    InvalidTickSpacing,
    FeeTierAlreadyExist,
    PoolKeyAlreadyExist,
    UnauthorizedFeeReceiver,
    ZeroLiquidity,
    TransferError,
    TokensAreSame,
    AmountUnderMinimumAmountOut,
    InvalidFee,
    NotEmptyTickDeinitialization,
    InvalidInitTick,
    InvalidInitSqrtPrice,
}
