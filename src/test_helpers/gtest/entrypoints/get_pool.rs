use contracts::*;
use gtest::*;
use sails_rtl::ActorId;

use io::*;

use crate::{send_query, test_helpers::gtest::PROGRAM_OWNER};
pub fn get_pool(
    invariant: &Program,
    token_0: ActorId,
    token_1: ActorId,
    fee_tier: FeeTier,
) -> Result<Pool, InvariantError> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetPool",
        payload: (token_0, token_1, fee_tier),
        response_type: Result<Pool, InvariantError>
    )
}
