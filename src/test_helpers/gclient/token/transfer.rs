use crate::test_helpers::{consts::TOKEN_PATH, gclient::utils::ProgramId, gclient::utils::*};
use fungible_token_io::*;
use gclient::{EventListener, EventProcessor, GearApi};
use gstd::fmt::Error;
use gstd::prelude::*;

pub async fn transfer(
    api: &GearApi,
    listener: &mut EventListener,
    token: ProgramId,
    from: impl Into<[u8; 32]> + Copy,
    to: impl Into<[u8; 32]> + Copy,
    amount: u128
) -> Result<(), String> {
    let message_id = send_message(
        api,
        token,
        FTAction::Transfer {
            tx_id: None,
            from: from.into().into(),
            to: to.into().into(),
            amount,
        },
    )
    .await;
    let res = listener
        .reply_bytes_on(message_id.into())
        .await
        .map_err(|e| e.to_string())?;
    res.1?;

    Ok(())
}
