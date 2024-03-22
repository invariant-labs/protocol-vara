use gstd::{Decode, Encode, TypeInfo};

#[derive(Clone, Decode, Encode, TypeInfo, PartialEq, Eq, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
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