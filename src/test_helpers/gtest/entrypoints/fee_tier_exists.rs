use contracts::*;
use gtest::*;
use io::*;

use crate::{send_query, test_helpers::gtest::PROGRAM_OWNER};

#[allow(dead_code)]
pub fn fee_tier_exists(invariant: &Program, fee_tier: FeeTier) -> bool {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "FeeTierExists",
        payload: (fee_tier),
        response_type: bool
    )
}
