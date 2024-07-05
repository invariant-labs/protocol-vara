use contracts::*;
use gstd::Vec;
use gtest::*;

use io::*;

use crate::{send_query, test_helpers::gtest::PROGRAM_OWNER};
pub fn get_liquidity_ticks(
    invariant: &Program,
    pool_key: PoolKey,
) -> Vec<LiquidityTick> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetLiquidityTicks",
        payload: (pool_key),
        response_type: Vec<LiquidityTick>
    )
}
