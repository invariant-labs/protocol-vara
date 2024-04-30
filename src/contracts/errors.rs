use gstd::{Decode, Encode, TypeInfo, String, prelude::*};

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
  RecoverableTransferError,
  UnrecoverableTransferError,
  TransferError,
  TokensAreSame,
  AmountUnderMinimumAmountOut,
  InvalidFee,
  NotEmptyTickDeinitialization,
  InvalidInitTick,
  InvalidInitSqrtPrice,
  NotEnoughGasToExecute,
  TickLimitReached,
  InvalidTickIndex,
  NoBalanceForTheToken,
  FailedToChangeTokenBalance,
}

impl Into<String>for InvariantError {
  fn into(self) -> String {
      format!("InvariantError: {:?}", self)
  }
}
 