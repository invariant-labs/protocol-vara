use fungible_token_io::*;
use gtest::*;

pub fn allowance(token: &Program, from: u64, to: u64) -> u128 {
    let state: IoFungibleToken = token.read_state(()).expect("Failed to read state");
    let allowances = state.allowances.iter().find(|(k, _)| *k == from.into());

    if let Some(allowances) = allowances {
        return allowances
            .1
            .iter()
            .find(|(k, _)| *k == to.into())
            .unwrap_or(&(to.into(), 0))
            .1;
    }
    0
}
