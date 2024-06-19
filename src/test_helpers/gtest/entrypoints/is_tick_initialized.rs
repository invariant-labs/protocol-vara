use contracts::*;
use gtest::*;

use io::*;
use math::percentage::Percentage;

use crate::{send_query, test_helpers::gtest::PROGRAM_OWNER};
pub fn is_tick_initialized(invariant: &Program, pool_key: PoolKey, index: i32) -> bool {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "IsTickInitialized",
        payload: (pool_key, index),
        response_type: bool
    )
}
