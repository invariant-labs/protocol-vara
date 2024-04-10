use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use gstd::*;
use gtest::*;
use io::*;

#[test]
fn test_change_protocol_fee() {
    let sys = System::new();
    let invariant = init_invariant(&sys, 0);
    let res = invariant.send(ADMIN, InvariantAction::ChangeProtocolFee(1));
    assert!(res.events_eq(vec![TestEvent::invariant_response(
        ADMIN,
        InvariantEvent::ProtocolFeeChanged(1)
    )]));
    assert_eq!(get_protocol_fee(&invariant), 1);
}

#[test]
fn test_change_protocol_fee_not_admin() {
    let sys = System::new();
    let mut invariant = init_invariant(&sys, 0);
    let _res = invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::ChangeProtocolFee(1),
        InvariantError::NotAdmin,
    );

    assert_eq!(get_protocol_fee(&invariant), 0);
}
