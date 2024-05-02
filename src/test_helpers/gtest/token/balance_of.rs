use gtest::*;
use fungible_token_io::*;

pub fn balance_of(
  token: &Program,
  account: u64,
) -> u128 {
  let state: IoFungibleToken = token
      .read_state(FTStateQuery::FullState)
      .expect("Failed to read state");
  state.balances.iter().find(|(k, _)| *k == account.into()).unwrap_or(&(account.into(), 0)).1
}