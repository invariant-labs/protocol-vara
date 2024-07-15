use contracts::*;
use gstd::Vec;
use gtest::*;

use io::*;

use crate::{send_query, test_helpers::gtest::PROGRAM_OWNER};
pub fn get_tickmap(
    invariant: &Program,
    pool_key: PoolKey,
) -> Vec<(u16, u64)> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetTickmap",
        payload: (pool_key),
        response_type: Vec<(u16, u64)>
    )
}
