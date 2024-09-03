use contracts::*;
use gstd::Vec;
use gtest::*;
use io::*;
use sails_rs::ActorId;

use crate::{send_query, test_helpers::gtest::PROGRAM_OWNER};
pub fn get_position_with_associates(
    invariant: &Program,
    owner: impl Into<ActorId>,
    id: u32,
) -> Result<(Position, Pool, Tick, Tick), InvariantError> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetPositionWithAssociates",
        payload: (owner.into(), id),
        response_type: Result<(Position, Pool, Tick, Tick), InvariantError>
    )
}