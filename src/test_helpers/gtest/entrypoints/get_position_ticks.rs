use contracts::*;
use gstd::Vec;
use gtest::*;
use io::*;
use sails_rs::ActorId;

use crate::{send_query, test_helpers::gtest::PROGRAM_OWNER};
pub fn get_position_ticks(
    invariant: &Program,
    owner: impl Into<ActorId>,
    offset: u32,
) -> Vec<PositionTick> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetPositionTicks",
        payload: (owner.into(), offset),
        response_type: Vec<PositionTick>
    )
}
