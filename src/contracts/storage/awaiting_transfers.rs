use math::token_amount::TokenAmount;
use sails_rtl::ActorId;

#[derive(Debug, Clone)]
pub struct AwaitingTransfer {
    pub account: ActorId,
    pub amount: TokenAmount,
    pub transfer_type: TransferType,
}

#[derive(Debug, Clone, Copy)]
pub enum TransferType {
    Deposit,
    Withdrawal,
}