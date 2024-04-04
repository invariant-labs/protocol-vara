use crate::test_helpers::{gclient::utils::ProgramId, consts::TOKEN_PATH};
use gclient::{EventListener, EventProcessor, GearApi};
use gstd::prelude::*;
use fungible_token_io::InitConfig;


pub async fn init_token(api: &GearApi, listener: &mut EventListener, init: InitConfig) -> ProgramId {
  let init_payload = init.encode();

  let gas_info = api
      .calculate_upload_gas(
          None,
          gclient::code_from_os(TOKEN_PATH).expect("failed to get code"),
          init_payload.clone(),
          0,
          true,
      )
      .await
      .expect("failed to calculate gas");

  let (message_id, program_id, _hash) = api
      .upload_program_bytes_by_path(
          TOKEN_PATH,
          gclient::now_micros().to_le_bytes(),
          init_payload,
          gas_info.burned * 2,
          0,
      )
      .await
      .expect("Failed to upload escrow program");
  assert!(
      listener
          .message_processed(message_id)
          .await
          .expect("failed to get message status")
          .succeed(),
      "Failed to create escrow program"
  );
  <[u8; 32]>::from(program_id)
}
