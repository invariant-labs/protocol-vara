use gtest::*;
use sails_rs::prelude::*;

use crate::{send_request, test_helpers::gtest::PROGRAM_OWNER};

pub fn burn(token: &Program, account: impl Into<ActorId>, value: U256) -> RunResult {
    send_request!(program: token, user: PROGRAM_OWNER, service_name: "Vft", action: "Burn", payload: (account.into(), value))
}
