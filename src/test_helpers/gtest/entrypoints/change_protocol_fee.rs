use super::InvariantResult;
use crate::send_request;
use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use contracts::{FeeTier, PoolKey};
use gtest::*;
use io::*;
use math::percentage::Percentage;
use sails_rtl::ActorId;

pub fn change_protocol_fee(invariant: &Program, user: u64, protocol_fee: Percentage) -> RunResult {
    send_request!(
        program: invariant,
        user: user,
        service_name: "Service",
        action: "ChangeProtocolFee",
        payload: (protocol_fee)
    )
}
