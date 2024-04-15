use crate::test_helpers::gclient::utils::*;
use contracts::InvariantError;
use gclient::{EventListener, EventProcessor, GearApi};
use gstd::{codec::decode_from_bytes, prelude::*};
use io::*;
use math::token_amount::TokenAmount;
#[allow(dead_code)]
pub async fn remove_position(
    api: &GearApi,
    listener: &mut EventListener,
    invariant: ProgramId,
    position_index: u32,
) -> Result<(TokenAmount, TokenAmount), InvariantError> {
    let message_id = send_message(
        api,
        invariant,
        InvariantAction::RemovePosition {position_id: position_index},
    )
    .await;
    let res = listener
        .reply_bytes_on(message_id.into())
        .await
        .expect("Failed to get reply");

    let bytes = res.1.unwrap().into();

    let event = decode_from_bytes::<InvariantEvent>(bytes).unwrap();
    match event {
        InvariantEvent::PositionRemovedReturn(token_x_amount, token_y_amount) => {
            return Ok((token_x_amount, token_y_amount));
        }
        InvariantEvent::ActionFailed(e) => {
            return Err(e);
        }
    _=> panic!("Unexpected event {:?}", event) 
  }
}
