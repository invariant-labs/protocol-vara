use crate::test_helpers::gtest::*;
use gtest::*;
use math::percentage::Percentage;
use decimal::*;

pub fn init_slippage_invariant_and_tokens(sys: &System) -> (Program<'_>, Program<'_>, Program<'_>) {
    let protocol_fee = Percentage::from_scale(1, 2);

    let invariant = init_invariant(sys, protocol_fee.0 as u128);

    let (token_x, token_y) = init_tokens(sys);

    (invariant, token_x, token_y)
}
