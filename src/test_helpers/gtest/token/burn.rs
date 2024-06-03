use super::U256;
use gstd::*;
use gtest::*;

use crate::{send_request, test_helpers::gtest::PROGRAM_OWNER};

pub fn burn(token: &Program, account: impl Into<ActorId>, value: u128) -> RunResult {
    send_request!(token: token, user: PROGRAM_OWNER, service_name: "Admin", action: "Burn", payload: (account.into(), U256::from(value)))
}
