use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gtest::*;
use math::percentage::Percentage;
#[test]
fn test_remove_fee_tier() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    remove_fee_tier(&invariant, ADMIN, fee_tier)
        .assert_single_event()
        .assert_empty()
        .assert_to(ADMIN);

    assert!(!fee_tier_exists(&invariant, fee_tier));
}

#[test]
fn remove_not_existing_fee_tier() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    remove_fee_tier(&invariant, ADMIN, fee_tier)
        .assert_panicked_with(InvariantError::FeeTierNotFound);
}

#[test]
fn test_remove_fee_tier_not_admin() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    remove_fee_tier(&invariant, REGULAR_USER_1, fee_tier)
        .assert_panicked_with(InvariantError::NotAdmin);
}
