use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use contracts::*;
use gstd::prelude::*;
use gtest::*;

use io::*;
pub fn get_fee_tiers(
    invariant: &Program,
) -> Vec<FeeTier> {
    let state: InvariantStateReply = invariant
        .read_state(InvariantStateQuery::GetFeeTiers)
        .expect("Failed to read state");
    if let InvariantStateReply::QueriedFeeTiers(fee_tiers) = state {
        return fee_tiers;
    } else {
        panic!("unexpected state {:?}", state);
    }
}
