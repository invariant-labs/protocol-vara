use crate::test_helpers::gclient::utils::*;
use contracts::PoolKey;
use gclient::GearApi;
use gstd::prelude::*;
use io::*;

pub async fn is_tick_initialized(
    api: &GearApi,
    invariant: ProgramId,
    pool_key: PoolKey,
    index: i32,
) -> bool {
    let payload = InvariantStateQuery::IsTickInitialized(pool_key, index).encode();
    let event = api
        .read_state::<InvariantStateReply>(invariant.into(), payload)
        .await
        .expect("Failed to read state");
    if let InvariantStateReply::IsTickInitialized(is_tick_initialized) = event {
        return is_tick_initialized;
    }
    panic!("Unexpected event {:?}", event);
}
