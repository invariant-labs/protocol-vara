use contracts::*;
use gtest::*;
use sails_rs::Vec;

use io::*;
use sails_rs::ActorId;

use crate::{send_query, test_helpers::gtest::PROGRAM_OWNER};
pub fn get_positions(
    invariant: &Program,
    owner_id: impl Into<ActorId>,
    size: u32,
    offset: u32,
) -> Result<(Vec<(Pool, Vec<(Position, u32)>)>, u32), InvariantError> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetPositions",
        payload: (owner_id.into(), size, offset),
        response_type: Result<(Vec<(Pool, Vec<(Position, u32)>)>, u32), InvariantError>
    )
}
