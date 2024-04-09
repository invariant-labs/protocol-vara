use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use gtest::*;
use io::*;

#[allow(dead_code)]
pub fn get_protocol_fee(
    invariant: &Program,
) -> u128 {
    let state: InvariantStateReply = invariant
        .read_state(InvariantStateQuery::GetProtocolFee)
        .expect("Failed to read state");
    if let InvariantStateReply::ProtocolFee(fee) = state {
        return fee;
    } else {
        panic!("unexpected state {:?}", state);
    }
}
