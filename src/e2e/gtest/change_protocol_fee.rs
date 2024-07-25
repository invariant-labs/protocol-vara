use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;
use contracts::*;
use gtest::*;
use math::percentage::Percentage;

#[test]
fn test_change_protocol_fee() {
    let sys = System::new();
    let invariant = init_invariant(&sys, Percentage(0));
    let res = change_protocol_fee(&invariant, ADMIN, Percentage(1));
    res.assert_single_event().assert_to(ADMIN).assert_empty();

    assert_eq!(get_protocol_fee(&invariant), Percentage(1));
}

#[test]
fn test_change_protocol_fee_not_admin() {
    let sys = System::new();
    let invariant = init_invariant(&sys, Percentage(0));

    change_protocol_fee(&invariant, REGULAR_USER_1, Percentage(1))
        .assert_panicked_with(InvariantError::NotAdmin);

    assert_eq!(get_protocol_fee(&invariant), Percentage(0));
}
