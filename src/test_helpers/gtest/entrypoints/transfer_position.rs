use super::InvariantResult;
use crate::send_request;
use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use contracts::{pool_key, FeeTier, PoolKey};
use gtest::*;
use io::*;
use math::{liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice};
use sails_rs::ActorId;

pub fn transfer_position(
    invariant: &Program,
    user: u64,
    index: u32,
    receiver: impl Into<ActorId>,
) -> RunResult {
    send_request!(
        program: invariant,
        user: user,
        service_name: "Service",
        action: "TransferPosition",
        payload: (index, receiver.into())
    )
}
