use super::InvariantResult;
use crate::send_request;
use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use contracts::{FeeTier, PoolKey};
use gtest::*;
use io::*;
use math::{percentage::Percentage, sqrt_price::SqrtPrice};
use sails_rtl::ActorId;

pub fn create_pool(
    invariant: &Program,
    user: u64,
    token_0: impl Into<ActorId>,
    token_1: impl Into<ActorId>,
    fee_tier: FeeTier,
    init_sqrt_price: SqrtPrice,
    init_tick: i32,
) -> RunResult {
    send_request!(
        program: invariant,
        user: user,
        service_name: "Service",
        action: "CreatePool",
        payload: (token_0.into(), token_1.into(), fee_tier, init_sqrt_price, init_tick)
    )
}
