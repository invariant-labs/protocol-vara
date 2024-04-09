

use crate::test_helpers::consts::*;
use contracts::*;
use gtest::*;
use gstd::ActorId;
use io::*;

pub fn get_position(
    invariant: &Program,
    owner: ActorId,
    index: u32,
) -> Result<Position, InvariantError> {
    let state: InvariantStateReply = invariant
        .read_state(InvariantStateQuery::GetPosition(owner, index))
        .expect("Failed to read state");
    if let InvariantStateReply::Position(position) = state {
        return Ok(position);
    } else if let InvariantStateReply::QueryFailed(e) = state {
        return Err(e);
    } else {
        panic!("unexpected state {:?}", state);
    }
}
