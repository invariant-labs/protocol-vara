use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use gtest::*;
#[test]
fn test_init_tokens() {
    let sys = System::new();
    sys.init_logger();

    let (token_x, token_y) = init_tokens(&sys);
    mint(&token_x, REGULAR_USER_1, 1000).assert_success();
    mint(&token_y, REGULAR_USER_1, 1000).assert_success();
}
