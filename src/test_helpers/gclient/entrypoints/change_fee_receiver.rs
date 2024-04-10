use crate::test_helpers::gclient::{get_pool, utils::*};
use contracts::{InvariantError, PoolKey};
use gclient::{EventListener, EventProcessor, GearApi};
use gstd::{codec::decode_from_bytes, prelude::*, ActorId};
use io::*;
#[allow(dead_code)]
pub async fn change_fee_receiver(
    api: &GearApi,
    listener: &mut EventListener,
    invariant: ProgramId,
    pool_key: PoolKey,
    fee_receiver: ActorId,
    expected_error: Option<InvariantError>,
) {
    let message_id = send_message(
        api,
        invariant,
        InvariantAction::ChangeFeeReceiver(pool_key, fee_receiver),
    )
    .await;
    let res = listener
        .reply_bytes_on(message_id.into())
        .await
        .expect("Failed to get reply");
    let bytes = res.1.unwrap().into();
    match expected_error {
        Some(e) => {
            assert_eq!(
                decode_from_bytes::<InvariantEvent>(bytes).unwrap(),
                InvariantEvent::ActionFailed(e)
            );
        }
        None => {
            let PoolKey {
                token_x,
                token_y,
                fee_tier,
            } = pool_key;
            let pool = get_pool(api, invariant, token_x, token_y, fee_tier, None)
                .await
                .expect("Failed to get pool");
            assert_eq!(pool.fee_receiver, fee_receiver);
        }
    }
}
