use crate::send_query;
use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use contracts::*;
use gstd::prelude::*;
use gtest::*;

use io::*;
pub fn get_fee_tiers(invariant: &Program) -> Vec<FeeTier> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetFeeTiers",
        payload: (),
        response_type: Vec<FeeTier>
    )
}
