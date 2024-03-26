use gclient::GearApi;
use gstd::prelude::*;

pub type MessageId = [u8; 32];
pub type UserId = [u8; 32];
pub type ProgramId = [u8; 32];

pub async fn send_message(
    api: &GearApi,
    program: impl Into<[u8; 32]> + gstd::Copy,
    message: impl Encode,
) -> MessageId {
    let gas_info = api
        .calculate_handle_gas(
            None,
            program.into().into(),
            message.encode().clone(),
            0,
            true,
        )
        .await
        .expect("Failed to send message");

    let (message_id, _hash) = api
        .send_message(program.into().into(), message, gas_info.burned * 2, 0)
        .await
        .expect("Failed to send message");
    <[u8; 32]>::from(message_id)
}

pub fn get_api_user_id (api: &GearApi) -> UserId {
    let user_id = api.account_id();
    <[u8; 32]>::from(user_id.clone())
}