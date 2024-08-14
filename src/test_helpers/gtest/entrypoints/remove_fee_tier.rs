use super::InvariantResult;
use crate::send_request;
use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use contracts::{pool_key, FeeTier, PoolKey};
use gtest::*;
use io::*;
use math::{liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice};
use sails_rs::ActorId;

pub fn remove_fee_tier(invariant: &Program, user: u64, fee_tier: FeeTier) -> RunResult {
    send_request!(
        program: invariant,
        user: user,
        service_name: "Service",
        action: "RemoveFeeTier",
        payload: (fee_tier)
    )
}
