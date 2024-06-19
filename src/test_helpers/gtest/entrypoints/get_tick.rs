use contracts::*;
use gtest::*;

use io::*;

use crate::{send_query, test_helpers::gtest::PROGRAM_OWNER};
pub fn get_tick(
    invariant: &Program,
    pool_key: PoolKey,
    index: i32,
) -> Result<Tick, InvariantError> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetTick",
        payload: (pool_key, index),
        response_type: Result<Tick, InvariantError>
    )
}
