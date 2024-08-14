use super::InvariantResult;
use crate::send_request;
use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use contracts::{FeeTier, PoolKey};
use gtest::*;
use io::*;
use math::percentage::Percentage;
use sails_rs::ActorId;

pub fn change_fee_receiver(
    invariant: &Program,
    user: u64,
    pool_key: PoolKey,
    fee_receiver: ActorId,
) -> RunResult {
    send_request!(
        program: invariant,
        user: user,
        service_name: "Service",
        action: "ChangeFeeReceiver",
        payload: (pool_key, fee_receiver)
    )
}
