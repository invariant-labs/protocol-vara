use contracts::*;
use gtest::*;

use io::*;
pub fn get_tick(
    invariant: &Program,
    pool_key: PoolKey,
    index: i32,
) -> Result<Tick, InvariantError> {
    let state: InvariantStateReply = invariant
        .read_state(InvariantStateQuery::GetTick(pool_key, index))
        .expect("Failed to read state");
    if let InvariantStateReply::Tick(tick) = state {
        return Ok(tick);
    } else if let InvariantStateReply::QueryFailed(e) = state {
        return Err(e);
    } else {
        panic!("unexpected state {:?}", state);
    }
}
