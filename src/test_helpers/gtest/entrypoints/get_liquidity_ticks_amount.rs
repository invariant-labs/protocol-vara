use contracts::*;
use gtest::*;
use sails_rs::Vec;

use io::*;

use crate::{send_query, test_helpers::gtest::PROGRAM_OWNER};
pub fn get_liquidity_ticks_amount(invariant: &Program, pool_key: PoolKey) -> u32 {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetLiquidityTicksAmount",
        payload: (pool_key),
        response_type: u32
    )
}
