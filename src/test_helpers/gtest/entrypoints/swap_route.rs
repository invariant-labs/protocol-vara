use crate::send_request;
use crate::test_helpers::gtest::*;
use contracts::{pool_key, FeeTier, PoolKey};
use gtest::*;
use io::*;
use math::{
    liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};
use sails_rs::prelude::*;

pub fn swap_route(
    invariant: &Program,
    user: u64,
    amount_in: TokenAmount,
    expected_token_amount: TokenAmount,
    slippage: Percentage,
    swaps: Vec<SwapHop>,
) -> RunResult {
    send_request!(
        program: invariant,
        user: user,
        service_name: "Service",
        action: "SwapRoute",
        payload: (amount_in, expected_token_amount, slippage, swaps)
    )
}
