use super::U256;
use crate::{send_request, test_helpers::gtest::*};
use gtest::*;

pub fn increase_allowance(token: &Program, owner: u64, spender: u64, amount: u128) -> RunResult {
    let current_allowance = allowance(token, owner, spender);

    set_allowance(token, owner, spender, current_allowance + amount)
}

pub fn set_allowance(token: &Program, owner: u64, spender: u64, amount: u128) -> RunResult {
    send_request!(
        program: token,
        user: owner,
        service_name: "Erc20",
        action: "Approve",
        payload: (ActorId::from(spender), U256::from(amount))
    )
}
