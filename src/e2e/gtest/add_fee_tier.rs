use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gstd::*;
use gtest::*;
use io::*;
use math::percentage::Percentage;

#[test]
fn test_add_multiple_fee_tiers() {
    let sys = System::new();
    sys.init_logger();

    let mut invariant = init_invariant(&sys, Percentage(100));

    let first_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    let second_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 2).unwrap();
    let third_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(first_fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));
    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(second_fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));
    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(third_fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let fee_tiers = get_fee_tiers(&invariant);
    assert_eq!(
        fee_tiers,
        vec![first_fee_tier, second_fee_tier, third_fee_tier]
    );

    assert!(fee_tier_exists(&invariant, first_fee_tier));
    assert!(fee_tier_exists(&invariant, second_fee_tier));
    assert!(fee_tier_exists(&invariant, third_fee_tier));
}

#[test]
fn test_add_existing_fee_tier() {
    let sys = System::new();
    sys.init_logger();

    let mut invariant = init_invariant(&sys, Percentage(100));

    let first_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    let second_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(first_fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));
    let _res = invariant.send_and_assert_panic(
        ADMIN,
        InvariantAction::AddFeeTier(second_fee_tier),
        InvariantError::FeeTierAlreadyExist,
    );
}

#[test]
fn test_add_fee_tier_not_admin() {
    let sys = System::new();
    sys.init_logger();

    let mut invariant = init_invariant(&sys, Percentage(100));

    let first_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();

    let _res = invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::AddFeeTier(first_fee_tier),
        InvariantError::NotAdmin,
    );
}

#[test]
fn test_add_fee_tier_zero_fee() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

    let res = invariant.send(
        ADMIN,
        InvariantAction::AddFeeTier(fee_tier),
    );
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]))
}
#[test]
fn test_add_fee_tier_tick_spacing_zero() {
  let sys = System::new();
  sys.init_logger();

  let mut invariant = init_invariant(&sys, Percentage(100));

  let fee_tier = FeeTier {
    fee: Percentage::from_scale(2, 4),
    tick_spacing: 0,
  };

  let _res = invariant.send_and_assert_panic(
      REGULAR_USER_1,
      InvariantAction::AddFeeTier(fee_tier),
      InvariantError::InvalidTickSpacing,
  );
}

#[test]
fn test_add_fee_tier_over_upper_bound_tick_spacing() {
  let sys = System::new();
  sys.init_logger();

  let mut invariant = init_invariant(&sys, Percentage(100));

  let fee_tier = FeeTier {
    fee: Percentage::from_scale(2, 4),
    tick_spacing: 101,
  };

  let _res = invariant.send_and_assert_panic(
      REGULAR_USER_1,
      InvariantAction::AddFeeTier(fee_tier),
      InvariantError::InvalidTickSpacing,
  );
}

#[test]
fn test_add_fee_tier_fee_above_limit() {
  let sys = System::new();
  sys.init_logger();

  let mut invariant = init_invariant(&sys, Percentage(100));

  let fee_tier = FeeTier::new(Percentage::from_integer(1), 10).unwrap();

  let _res = invariant.send_and_assert_panic(
      REGULAR_USER_1,
      InvariantAction::AddFeeTier(fee_tier),
      InvariantError::InvalidFee,
  );
}