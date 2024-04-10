use crate::test_helpers::gclient::utils::*;
use contracts::{InvariantError, PoolKey, Tick};
use gclient::GearApi;
use gstd::prelude::*;
use io::*;

#[allow(dead_code)]
pub async fn get_tick(
    api: &GearApi,
    invariant: ProgramId,
    pool_key: PoolKey,
    index: i32,
    expected_error: Option<InvariantError>,
) -> Option<Tick> {
    let payload = InvariantStateQuery::GetTick(pool_key, index).encode();
    let state = api
        .read_state::<InvariantStateReply>(invariant.into(), payload)
        .await
        .expect("Failed to read state");
    match expected_error {
        Some(e) => {
            assert_eq!(state, InvariantStateReply::QueryFailed(e));
            return None;
        }
        None => {
            if let InvariantStateReply::Tick(tick) = state {
                return tick.into();
            }
            panic!("Unexpected state {:?}", state);
        }
    }
}
