use fungible_token_io::*;
use gtest::*;

use crate::test_helpers::gtest::*;

pub fn increase_allowance(token: &Program, from: u64, to: u64, amount: u128) -> RunResult {
    let current_allowance = allowance(token, from, to);
    let res = token.send(
        from,
        FTAction::Approve {
            tx_id: None,
            to: to.into(),
            amount: amount + current_allowance,
        },
    );
    res
}
