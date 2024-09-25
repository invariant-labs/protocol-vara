use crate::{send_query, test_helpers::gtest::*};
use contracts::*;
use gtest::*;
use sails_rs::prelude::*;

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
