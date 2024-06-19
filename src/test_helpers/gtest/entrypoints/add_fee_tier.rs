use crate::send_request;
use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use contracts::FeeTier;
use gtest::*;
use io::*;
use math::percentage::Percentage;

use super::InvariantResult;

pub fn add_fee_tier(invariant: &Program, user: u64, fee_tier: FeeTier) -> RunResult {
    send_request!(
        program: invariant,
        user: user,
        service_name: "Service",
        action: "AddFeeTier",
        payload: (fee_tier)
    )
}
