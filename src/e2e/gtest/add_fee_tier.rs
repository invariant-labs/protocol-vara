use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gtest::*;
use math::percentage::Percentage;

#[test]
fn test_add_multiple_fee_tiers() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let first_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    let second_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 2).unwrap();
    let third_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, first_fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);
    let res = add_fee_tier(&invariant, ADMIN, second_fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);
    let res = add_fee_tier(&invariant, ADMIN, third_fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

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

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let first_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    let second_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, first_fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);
    let res = add_fee_tier(&invariant, ADMIN, second_fee_tier);
    res.assert_panicked_with(InvariantError::FeeTierAlreadyExist);
}

#[test]
fn test_add_fee_tier_not_admin() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let first_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, first_fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);
}

#[test]
fn test_add_fee_tier_zero_fee() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let fee_tier = FeeTier::new(Percentage::new(U128::from(0)), 1).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);
}
#[test]
fn test_add_fee_tier_tick_spacing_zero() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(2, 4),
        tick_spacing: 0,
    };

    add_fee_tier(&invariant, REGULAR_USER_1, fee_tier)
        .assert_panicked_with(InvariantError::InvalidTickSpacing);
}

#[test]
fn test_add_fee_tier_over_upper_bound_tick_spacing() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(2, 4),
        tick_spacing: 101,
    };

    add_fee_tier(&invariant, REGULAR_USER_1, fee_tier)
        .assert_panicked_with(InvariantError::InvalidTickSpacing);
}

#[test]
fn test_add_fee_tier_fee_above_limit() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(100)));

    let fee_tier = FeeTier::new(Percentage::from_integer(1), 10).unwrap();

    add_fee_tier(&invariant, REGULAR_USER_1, fee_tier)
        .assert_panicked_with(InvariantError::InvalidFee);
}
