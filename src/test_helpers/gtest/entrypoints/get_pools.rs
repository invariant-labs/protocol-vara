use crate::test_helpers::consts::*;
use contracts::*;
use gstd::{prelude::*, Result};
use gtest::*;

use io::*;
pub fn get_pools(
    invariant: &Program,
    size: u8,
    offset: u16,
) -> Result<Vec<PoolKey>, InvariantError> {
    let state: InvariantStateReply = invariant
        .read_state(InvariantStateQuery::GetPools(size, offset))
        .expect("Failed to read state");

    if let InvariantStateReply::Pools(pools) = state {
        return Ok(pools);
    } else if let InvariantStateReply::QueryFailed(e) = state {
        return Err(e);
    } else {
        panic!("unexpected state {:?}", state);
    }
}
