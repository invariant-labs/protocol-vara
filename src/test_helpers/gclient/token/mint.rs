use crate::test_helpers::{gclient::utils::ProgramId, gclient::utils::*};
use gclient::{EventListener, EventProcessor, GearApi};
use gstd::prelude::*;

#[allow(dead_code)]
pub async fn mint(
    api: &GearApi,
    listener: &mut EventListener,
    token: ProgramId,
    amount: u128,
) -> Result<(), String> {
    unimplemented!();
    // let message_id = send_message(
    //     api,
    //     token,
    //     FTAction::Mint(amount)
    // )
    // .await;
    // let res = listener
    //     .reply_bytes_on(message_id.into())
    //     .await
    //     .map_err(|e| e.to_string())?;
    // res.1?;

    // Ok(())
}
