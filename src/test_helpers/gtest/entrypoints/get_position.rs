use crate::{
    send_query,
    test_helpers::{consts::*, gtest::PROGRAM_OWNER},
};
use contracts::*;
use gtest::*;
use io::*;
use sails_rs::ActorId;

pub fn get_position(
    invariant: &Program,
    owner: ActorId,
    index: u32,
) -> Result<Position, InvariantError> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetPosition",
        payload: (owner, index),
        response_type: Result<Position, InvariantError>
    )
}
