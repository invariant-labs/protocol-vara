use crate::{
    send_query,
    test_helpers::{consts::*, gtest::PROGRAM_OWNER},
};
use contracts::*;
use gstd::{prelude::*, ActorId, Result};
use gtest::*;

use io::*;
pub fn get_all_positions(invariant: &Program, owner_id: ActorId) -> Vec<Position> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetAllPositions",
        payload: (owner_id),
        response_type: Vec<Position>
    )
}
