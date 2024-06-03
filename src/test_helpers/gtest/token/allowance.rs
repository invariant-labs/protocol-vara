use crate::{send_query, test_helpers::gtest::REGULAR_USER_1};
use gtest::*;

use super::U256;

pub fn allowance(token: &Program, owner: u64, spender: u64) -> u128 {
    send_query!(
      token: token,
      user: REGULAR_USER_1,
      service_name: "Erc20",
      action: "Allowance",
      payload: (ActorId::from(owner), ActorId::from(spender)),
      response_type: U256
    )
    .0
}
