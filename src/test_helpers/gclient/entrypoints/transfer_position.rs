use crate::test_helpers::gclient::utils::*;
use contracts::InvariantError;
use gclient::{EventListener, EventProcessor, GearApi};
use gstd::{codec::decode_from_bytes, prelude::*};
use io::*;
#[allow(dead_code)]
pub async fn transfer_position(
    api: &GearApi,
    listener: &mut EventListener,
    invariant: ProgramId,
    position_index: u32,
    receiver: UserId,
    expected_error: Option<InvariantError>,
) {
    let message_id = send_message(
        api,
        invariant,
        InvariantAction::TransferPosition {
            index: position_index,
            receiver: receiver.into(),
        },
    )
    .await;
    let res = listener
        .reply_bytes_on(message_id.into())
        .await
        .expect("Failed to get reply");
  
    let bytes = res.1.unwrap().into();

    let event = decode_from_bytes::<InvariantEvent>(bytes);
    if let Some(e) = expected_error {
        let event = event.unwrap();

        if event == InvariantEvent::ActionFailed(e) {
            return;
        }
        panic!("Unexpected event {:?}", event)
    } else {
      //Check that no event was emitted
      if  event.is_err() { return; }

      assert!(false, "Unexpected event {:?}", event.unwrap());
    }
}
