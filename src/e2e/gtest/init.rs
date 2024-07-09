use crate::test_helpers::gtest::*;
use decimal::*;
use gtest::*;
use math::percentage::Percentage;

#[test]
fn test_init() {
    let sys = System::new();
    sys.init_logger();

    let _invariant = init_invariant(&sys, Percentage(100));
}
