use crate::{send_request, test_helpers::gtest::*};
use contracts::{pool_key, FeeTier, PoolKey};
use gtest::*;
use io::*;
use math::{liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice};
use sails_rs::ActorId;

pub fn withdraw_protocol_fee(invariant: &Program, user: u64, pool_key: PoolKey) -> RunResult {
    send_request!(
        program: invariant,
        user: user,
        service_name: "Service",
        action: "WithdrawProtocolFee",
        payload: (pool_key)
    )
}
