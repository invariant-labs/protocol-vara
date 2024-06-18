#![no_std]
use io::InvariantConfig;
use sails_rtl::gstd::{gprogram, GStdExecContext};
mod invariant_service;
mod invariant_storage;
use invariant_service::InvariantService;
use invariant_storage::InvariantStorage;
pub use contracts::{
  FeeTier, FeeTiers, InvariantError, Pool, PoolKey, PoolKeys, Pools, Position, Positions, Tick,
  Tickmap, Ticks, UpdatePoolTick, TransferType, AwaitingTransfer,
};
use sails_rtl::{ActorId, MessageId};
use sails_rtl::{gstd::msg, String};
use gstd::exec::program_id;

//type ServiceOf<T> = <T as sails_rtl::gstd::services::Service>::Exposure;
pub fn signal_handler() {
    if program_id() == msg::source().into() {
        return;
    }

    let invariant = InvariantStorage::as_mut();
    let token: ActorId = msg::source().into();
    #[cfg(feature = "test")]
    let message_with_error = msg::reply_to().unwrap();
    #[cfg(not(feature = "test"))]
    let message_with_error: MessageId = msg::signal_from().unwrap().into();

    let (update_values, message_exists) = {
        let message_record = invariant
            .awaiting_transfers
            .get(&(message_with_error.into(), token));
        let update_values = message_record.and_then(
            |AwaitingTransfer {
                 transfer_type,
                 account,
                 amount,
             }| {
                // Only failure on withdrawal needs to stored, since in case of deposit failure the amount is not deducted from users account
                if matches!(transfer_type, TransferType::Withdrawal) {
                    Some((*account, token, *amount))
                } else {
                    None
                }
            },
        );

        (update_values, message_record.is_some())
    };

    if let Some((account, token, amount)) = update_values {
        if let Err(e) = invariant.increase_token_balance(&token, &account, amount) {
            gstd::debug!(
                "Failed to increase balance, {:?}, {:?}, {:?}, {:?}",
                account,
                &token,
                amount,
                e
            );
        }
    }

    if message_exists
        && invariant
            .awaiting_transfers
            .remove(&(message_with_error.into(), token))
            .is_none()
    {
        gstd::debug!(
            "Failed to remove transfer {:?}, {:?}",
            message_with_error,
            token
        );
    }

    gstd::debug!("Signal handling finished");
}

pub fn reply_handler() {
    let invariant = InvariantStorage::as_mut();
    let token: ActorId = msg::source().into();
    let message_with_error: MessageId = msg::reply_to().unwrap().into();

    let msg_data: Result<(String, String, bool), gstd::errors::Error> = msg::load();

    let (update_values, message_exists) = {
        let message = invariant
            .awaiting_transfers
            .get(&(message_with_error, token));

        let update_values = message.and_then(
            |AwaitingTransfer {
                 transfer_type,
                 account,
                 amount,
             }| {
                if let Ok((_, _, message_result)) = msg_data {
                    match transfer_type {
                        TransferType::Deposit => {
                            if message_result {
                                Some((*account, token, *amount))
                            } else {
                                None
                            }
                        }
                        TransferType::Withdrawal => {
                            if !message_result {
                                Some((*account, token, *amount))
                            } else {
                                None
                            }
                        }
                    }
                } else {
                    gstd::debug!("Invalid message");

                    None
                }
            },
        );

        (update_values, message.is_some())
    };

    if let Some((account, token, amount)) = update_values {
        if let Err(e) = invariant.increase_token_balance(&token, &account, amount) {
            gstd::debug!(
                "Failed to increase balance, {:?}, {:?}, {:?}, {:?}",
                account,
                &token,
                amount,
                e
            );
        }
    }

    if message_exists {
        if invariant
            .awaiting_transfers
            .remove(&(message_with_error, token))
            .is_none()
        {
            gstd::debug!(
                "Failed to remove transfer {:?}, {:?}",
                message_with_error,
                token
            );
        }
    }

    gstd::debug!("Reply handling finished");
}

// this is necessary to test custom handle_signal entrypoint with gtest
// since handle_signal appears to be handled in handle_reply
#[cfg(feature = "test")]
pub fn reply_and_signal_handler() {
    if msg::load::<(String, String, bool)>().is_ok() {
        reply_handler()
    } else if msg::load::<String>().is_ok() {
        signal_handler()
    } else {
        gstd::debug!("Unknown message type")
    }
}
pub struct InvariantProgram(());

#[cfg_attr(feature = "test", gprogram(handle_reply = reply_handler, handle_signal = signal_handler))]
#[cfg_attr(not(feature = "test"), gprogram(handle_reply = reply_handler, handle_signal = signal_handler))]
impl InvariantProgram {
    pub fn new(config: InvariantConfig) -> Self {
        InvariantService::<GStdExecContext>::seed(config);
        Self(())
    }

    pub fn service(&self) -> InvariantService<GStdExecContext> {
        InvariantService::new(GStdExecContext::new())
    }
}
