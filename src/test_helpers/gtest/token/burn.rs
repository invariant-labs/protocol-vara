use super::U256;
use gtest::*;
use sails_rtl::ActorId;

use crate::{send_request, test_helpers::gtest::PROGRAM_OWNER};

pub fn burn(token: &Program, account: impl Into<ActorId>, value: u128) -> RunResult {
    send_request!(program: token, user: PROGRAM_OWNER, service_name: "Admin", action: "Burn", payload: (account.into(), U256::from(value)))
}
