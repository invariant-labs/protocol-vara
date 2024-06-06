use crate::{send_request, test_helpers::gtest::*};
use gtest::*;

#[must_use]
pub fn set_transfer_fail(token: &Program, flag: bool) -> RunResult {
    send_request!(token: token, user: PROGRAM_OWNER, service_name: "Erc20", action: "SetFailTransfer", payload: (flag))
}