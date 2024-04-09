use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use fungible_token_io::*;
use gstd::*;
use gtest::*;
#[test]
fn test_init_tokens() {
    let sys = System::new();
    sys.init_logger();

    let (token_x, token_y) = init_tokens(&sys);
    assert!(!token_x
        .send(REGULAR_USER_1, FTAction::Mint(1000))
        .main_failed());
    assert!(!token_y
        .send(REGULAR_USER_2, FTAction::Mint(1000))
        .main_failed());
}
