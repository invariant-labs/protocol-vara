use crate::test_helpers::gclient::utils::*;
use contracts::{FeeTier, InvariantError};
use gclient::{EventListener, EventProcessor, GearApi};
use gstd::{codec::decode_from_bytes, prelude::*, ActorId};
use io::*;
use math::sqrt_price::SqrtPrice;

pub async fn create_pool(
    api: &GearApi,
    listener: &mut EventListener,
    invariant: ProgramId,
    token_0: impl Into<ActorId>,
    token_1: impl Into<ActorId>,
    fee_tier: FeeTier,
    init_sqrt_price: SqrtPrice,
    init_tick: i32,
    expected_error: Option<InvariantError>,
) {
    let message_id = send_message(
        api,
        invariant,
        InvariantAction::CreatePool {
            fee_tier,
            token_0: token_0.into(),
            token_1: token_1.into(),
            init_sqrt_price,
            init_tick,
        },
    )
    .await;
    let res = listener
        .reply_bytes_on(message_id.into())
        .await
        .expect("Failed to get reply");
    
    let bytes = res.1.expect("main panicked").into();
    match expected_error {
        Some(e) => {
            assert_eq!(
                decode_from_bytes::<InvariantEvent>(bytes).unwrap(),
                InvariantEvent::ActionFailed(e)
            );
        }
        None => {
          //Check that no event was emitted  
          let event = decode_from_bytes::<InvariantEvent>(bytes);
          if  event.is_err() { return; }

          assert!(false, "Unexpected event {:?}", event.unwrap());
        }
    }
}
