use crate::test_helpers::gclient::utils::*;
use contracts::{InvariantError, PoolKey};
use gclient::{EventListener, EventProcessor, GearApi};
use gstd::{codec::decode_from_bytes, prelude::*};
use io::*;
use math::{liquidity::Liquidity, sqrt_price::SqrtPrice};

pub async fn create_position(
    api: &GearApi,
    listener: &mut EventListener,
    invariant: ProgramId,
    pool_key: PoolKey,
    lower_tick: i32,
    upper_tick: i32,
    liquidity_delta: Liquidity,
    slippage_limit_lower: SqrtPrice,
    slippage_limit_upper: SqrtPrice,
    expected_error: Option<InvariantError>,
) {
    let message_id = send_message(
        api,
        invariant,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick,
            upper_tick,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
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
            let event = decode_from_bytes::<InvariantEvent>(bytes).unwrap();
            if let InvariantEvent::PositionCreatedReturn(_position) = &event {
                return;
            }

            assert!(false, "Unexpected event {:?}", &event);
        }
    }
}
