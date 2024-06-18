use super::InvariantResult;
use crate::send_request;
use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use contracts::{pool_key, FeeTier, PoolKey};
use gtest::*;
use io::*;
use math::{liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice};
use sails_rtl::ActorId;

pub fn create_position(
    invariant: &Program,
    user: u64,
    pool_key: PoolKey,
    lower_tick: i32,
    upper_tick: i32,
    liquidity_delta: Liquidity,
    slippage_limit_lower: SqrtPrice,
    slippage_limit_upper: SqrtPrice,
) -> RunResult {
    send_request!(
        program: invariant,
        user: user,
        service_name: "Service",
        action: "CreatePosition",
        payload: (pool_key, lower_tick, upper_tick, liquidity_delta, slippage_limit_lower, slippage_limit_upper)
    )
}
