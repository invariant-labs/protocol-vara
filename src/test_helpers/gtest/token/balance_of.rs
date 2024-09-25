use crate::{send_query, test_helpers::gtest::REGULAR_USER_1};
use gtest::*;
use sails_rs::prelude::*;

pub fn balance_of(token: &Program, account: u64) -> U256 {
    send_query!(
      program: token,
      user: REGULAR_USER_1,
      service_name: "Vft",
      action: "BalanceOf",
      payload: (ActorId::from(account)),
      response_type: U256
    )
}
