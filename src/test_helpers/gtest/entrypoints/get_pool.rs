use contracts::*;
use gstd::ActorId;
use gtest::*;

use io::*;
pub fn get_pool(
    invariant: &Program,
    token_0: ActorId,
    token_1: ActorId,
    fee_tier: FeeTier,
) -> Result<Pool, InvariantError> {
    let state: InvariantStateReply = invariant
        .read_state(InvariantStateQuery::GetPool(token_0, token_1, fee_tier))
        .expect("Failed to read state");
    if let InvariantStateReply::Pool(pool) = state {
        return Ok(pool);
    } else if let InvariantStateReply::QueryFailed(e) = state {
        return Err(e);
    } else {
        panic!("unexpected state {:?}", state);
    }
}
