use super::init_tokens;
use crate::test_helpers::gtest::*;
use gtest::*;

pub fn init_tokens_with_mint(
    sys: &System,
    initial_balances: (u128, u128),
) -> (Program<'_>, Program<'_>) {
    let (token_x, token_y) = init_tokens(&sys);
    mint(&token_x, REGULAR_USER_1, initial_balances.0).assert_success();
    mint(&token_x, REGULAR_USER_2, initial_balances.0).assert_success();

    mint(&token_y, REGULAR_USER_1, initial_balances.1).assert_success();
    mint(&token_y, REGULAR_USER_2, initial_balances.1).assert_success();

    (token_x, token_y)
}
