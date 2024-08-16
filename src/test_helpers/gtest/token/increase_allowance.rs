use crate::{send_request, test_helpers::gtest::*};
use gtest::*;
use decimal::U256;

pub fn increase_allowance(token: &Program, owner: u64, spender: u64, amount: U256) -> RunResult {
    let current_allowance = allowance(token, owner, spender);

    set_allowance(token, owner, spender, current_allowance + amount)
}

pub fn set_allowance(token: &Program, owner: u64, spender: u64, amount: U256) -> RunResult {
    send_request!(
        program: token,
        user: owner,
        service_name: "Vft",
        action: "Approve",
        payload: (ActorId::from(spender), amount)
    )
}
