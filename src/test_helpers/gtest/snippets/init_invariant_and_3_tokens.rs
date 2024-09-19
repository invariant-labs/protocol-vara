use crate::test_helpers::gtest::*;
use decimal::*;
use gtest::*;
use math::percentage::Percentage;

pub fn init_invariant_and_3_tokens(
    sys: &System,
) -> (Program<'_>, Program<'_>, Program<'_>, Program<'_>) {
    let protocol_fee = Percentage::from_scale(1, 2);

    let invariant = init_invariant(sys, protocol_fee);

    let (token_x, token_y, token_z) = (
        init_token(sys, TOKEN_X_ID),
        init_token(sys, TOKEN_Y_ID),
        init_token(sys, TOKEN_Z_ID),
    );

    (invariant, token_x, token_y, token_z)
}
