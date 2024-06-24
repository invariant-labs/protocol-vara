#![no_std]
extern crate alloc;
#[cfg(test)]
mod e2e;
#[cfg(test)]
mod test_helpers;

use io::InvariantConfig;
use sails_rtl::gstd::{gprogram, GStdExecContext};
mod invariant_service;
mod invariant_storage;
pub use contracts::{
    AwaitingTransfer, FeeTier, FeeTiers, InvariantError, Pool, PoolKey, PoolKeys, Pools, Position,
    Positions, Tick, Tickmap, Ticks, TransferType, UpdatePoolTick,
};
use gstd::exec::program_id;
use invariant_service::InvariantService;
use invariant_storage::InvariantStorage;
use sails_rtl::{gstd::msg, String};
use sails_rtl::{ActorId, MessageId};

pub fn handle_panic() {
    if program_id() == msg::source().into() {
        return;
    }

    let invariant = InvariantStorage::as_mut();
    let token: ActorId = msg::source().into();
    let message_with_error = msg::reply_to().unwrap();

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

    gstd::debug!("Panic handling finished");
}

pub fn handle_valid_reply() {
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

pub fn reply_handler() {
    // message is a valid reply
    if msg::load::<(String, String, bool)>().is_ok() {
        handle_valid_reply()
    // message is a valid panic
    } else if msg::load::<String>().is_ok() {
        handle_panic()
    } else {
        gstd::debug!("Unknown message type");
    }
}
pub struct InvariantProgram(());

#[gprogram(handle_reply = reply_handler)]
impl InvariantProgram {
    pub fn new(config: InvariantConfig) -> Self {
        InvariantService::<GStdExecContext>::seed(config);
        Self(())
    }

    pub fn service(&self) -> InvariantService<GStdExecContext> {
        InvariantService::new(GStdExecContext::new())
    }
}
