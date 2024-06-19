use crate::{
    send_query,
    test_helpers::{consts::*, gtest::PROGRAM_OWNER},
};
use contracts::*;
use gstd::{prelude::*, Result};
use gtest::*;

use io::*;
pub fn get_pools(
    invariant: &Program,
    size: u8,
    offset: u16,
) -> Result<Vec<PoolKey>, InvariantError> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetPools",
        payload: (size, offset),
        response_type: Result<Vec<PoolKey>, InvariantError>
    )
}
