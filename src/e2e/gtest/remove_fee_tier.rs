use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gstd::*;
use gtest::*;
use io::*;
use math::percentage::Percentage;
#[test]
fn test_remove_fee_tier() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, 100);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(ADMIN, InvariantAction::RemoveFeeTier(fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    assert!(!fee_tier_exists(&invariant, fee_tier));
}

#[test]
fn remove_not_existing_fee_tier() {
    let sys = System::new();
    sys.init_logger();

    let mut invariant = init_invariant(&sys, 100);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    let _res = invariant.send_and_assert_panic(
        ADMIN,
        InvariantAction::RemoveFeeTier(fee_tier),
        InvariantError::FeeTierNotFound,
    );
}

#[test]
fn test_remove_fee_tier_not_admin() {
    let sys = System::new();
    sys.init_logger();

    let mut invariant = init_invariant(&sys, 100);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let _res = invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::RemoveFeeTier(fee_tier),
        InvariantError::NotAdmin,
    );
}
