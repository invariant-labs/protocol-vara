use crate::test_helpers::gtest::*;

use gtest::*;
use gstd::*;

#[test]
fn test_init() {
    let sys = System::new();
    sys.init_logger();

    let _invariant = init_invariant(&sys, 100);
}