use super::U256;
use crate::{send_request, test_helpers::gtest::PROGRAM_OWNER};
use gstd::*;
use gtest::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Role {
    Admin,
    Burner,
    Minter,
}

#[must_use]
pub fn mint(token: &Program, account: impl Into<ActorId>, value: u128) -> RunResult {
    send_request!(token: token, user: PROGRAM_OWNER, service_name: "Admin", action: "Mint", payload: (account.into(), U256(value, 0u128)))
}
