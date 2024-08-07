use crate::{send_query, test_helpers::gtest::REGULAR_USER_1};
use gtest::*;
use sails_rtl::ActorId;
use decimal::U256;

pub fn balance_of(token: &Program, account: u64) -> U256 {
    send_query!(
      program: token,
      user: REGULAR_USER_1,
      service_name: "Erc20",
      action: "BalanceOf",
      payload: (ActorId::from(account)),
      response_type: U256
    )
}
