use contracts::*;
use gtest::*;

use io::*;
pub fn is_tick_initialized(invariant: &Program, pool_key: PoolKey, index: i32) -> bool {
    let state: InvariantStateReply = invariant
        .read_state(InvariantStateQuery::IsTickInitialized(pool_key, index))
        .expect("Failed to read state");
    if let InvariantStateReply::IsTickInitialized(is_tick_initialized) = state {
        return is_tick_initialized;
    } else {
        panic!("unexpected state {:?}", state);
    }
}
