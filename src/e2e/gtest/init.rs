use crate::test_helpers::gtest::*;

use gstd::*;
use gtest::*;

#[test]
fn test_init() {
    let sys = System::new();
    sys.init_logger();

    let _invariant = init_invariant(&sys, 100);
}
