use crate::test_helpers::gtest::*;
use decimal::*;
use gtest::*;
use math::percentage::Percentage;

pub fn init_slippage_invariant_and_tokens(sys: &System) -> (Program<'_>, Program<'_>, Program<'_>) {
    let protocol_fee = Percentage::from_scale(1, 2);

    let invariant = init_invariant(sys, protocol_fee);

    let (token_x, token_y) = init_tokens(sys);

    (invariant, token_x, token_y)
}
