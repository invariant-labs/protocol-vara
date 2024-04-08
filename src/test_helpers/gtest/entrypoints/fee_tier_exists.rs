use contracts::*;
use gtest::*;

use io::*;
pub fn fee_tier_exists(
    invariant: &Program,
    fee_tier: FeeTier,
) -> bool {
    let state: InvariantStateReply = invariant
        .read_state(InvariantStateQuery::FeeTierExist(fee_tier))
        .expect("Failed to read state");
    if let InvariantStateReply::FeeTierExist(exists) = state {
        return exists;
    } else {
        panic!("unexpected state {:?}", state);
    }
}
