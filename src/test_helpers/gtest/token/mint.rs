use crate::{send_request, test_helpers::gtest::PROGRAM_OWNER};
use gtest::*;
use sails_rs::prelude::*;

#[must_use]
pub fn mint(token: &Program, account: impl Into<ActorId>, value: U256) -> RunResult {
    send_request!(program: token, user: PROGRAM_OWNER, service_name: "Admin", action: "Mint", payload: (account.into(), value))
}
