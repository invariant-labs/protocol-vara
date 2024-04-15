use crate::test_helpers::consts::*;
use contracts::*;
use gstd::{prelude::*, ActorId, Result};
use gtest::*;

use io::*;
pub fn get_all_positions(invariant: &Program, owner_id: ActorId) -> Vec<Position> {
    let state: InvariantStateReply = invariant
        .read_state(InvariantStateQuery::GetAllPositions(owner_id))
        .expect("Failed to read state");

    if let InvariantStateReply::Positions(positions) = state {
        return positions;
    } else {
        panic!("unexpected state {:?}", state);
    }
}
