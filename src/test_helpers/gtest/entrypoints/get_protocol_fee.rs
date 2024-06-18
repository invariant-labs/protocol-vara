use crate::send_query;
use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use gtest::*;
use io::*;
use math::percentage::Percentage;

pub fn get_protocol_fee(invariant: &Program) -> Percentage {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetProtocolFee",
        payload: (),
        response_type: Percentage
    )
}
