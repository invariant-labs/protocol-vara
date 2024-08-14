use contracts::*;
use gtest::*;
use sails_rs::{ActorId, Vec};

use io::*;
use math::{percentage::Percentage, token_amount::TokenAmount};

use crate::{send_query, test_helpers::gtest::PROGRAM_OWNER};
pub fn get_user_balances(invariant: &Program, user: u64) -> Vec<(ActorId, TokenAmount)> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "GetUserBalances",
        payload: (ActorId::from(user)),
        response_type: Vec<(ActorId, TokenAmount)>
    )
}
