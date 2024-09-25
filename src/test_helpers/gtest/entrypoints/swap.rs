use crate::send_request;
use crate::test_helpers::gtest::*;
use contracts::{pool_key, FeeTier, PoolKey};
use gtest::*;
use io::*;
use math::{
    liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};
use sails_rs::prelude::*;

pub fn swap(
    invariant: &Program,
    user: u64,
    pool_key: PoolKey,
    x_to_y: bool,
    amount: TokenAmount,
    by_amount_in: bool,
    sqrt_price_limit: SqrtPrice,
) -> RunResult {
    send_request!(
        program: invariant,
        user: user,
        service_name: "Service",
        action: "Swap",
        payload: (pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit)
    )
}
