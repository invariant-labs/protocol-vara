use crate::{
    send_query,
    test_helpers::{consts::*, gtest::PROGRAM_OWNER},
};
use contracts::*;
use gstd::{prelude::*, Result};
use gtest::*;

use io::*;
pub fn get_pool_keys(
    invariant: &Program,
    size: u16,
    offset: u16,
) -> (Vec<PoolKey>, u16) {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetPoolKeys",
        payload: (size, offset),
        response_type: (Vec<PoolKey>, u16)
    )
}
