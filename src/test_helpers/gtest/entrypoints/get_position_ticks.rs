use crate::{send_query, test_helpers::gtest::*};
use contracts::*;
use gtest::*;
use io::*;
use sails_rs::prelude::*;

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
