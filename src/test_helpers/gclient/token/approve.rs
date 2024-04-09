use crate::test_helpers::{gclient::utils::ProgramId, gclient::utils::*};
use fungible_token_io::*;
use gclient::{EventListener, EventProcessor, GearApi};
use gstd::prelude::*;

#[allow(dead_code)]
pub async fn approve(
    api: &GearApi,
    listener: &mut EventListener,
    token: ProgramId,
    to: impl Into<[u8; 32]> + Copy,
    amount: u128
) -> Result<(), String> {
    let message_id = send_message(
        api,
        token,
        FTAction::Approve {
            tx_id: None,
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
