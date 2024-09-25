use sails_rs::prelude::*;

#[derive(Clone, Decode, Encode, TypeInfo, PartialEq, Eq, Debug)]
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
    ReplyHandlingFailed,
    InvalidVaraDepositAttempt,
    InvalidVaraWithdrawAttempt,
}

impl Into<String> for InvariantError {
    fn into(self) -> String {
        format!("InvariantError: {:?}", self)
    }
}
