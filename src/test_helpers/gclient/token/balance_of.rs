use gstd::ActorId;
use gclient::GearApi;
use fungible_token_io::*;

pub async fn balance_of(
  api: &GearApi,
  token: impl Into<[u8; 32]> + gstd::Copy,
  account: impl Into<[u8; 32]> + gstd::Copy,
)->u128 {
  let state = api
      .read_state::<IoFungibleToken>(token.into().into(), [].into())
      .await
      .expect("Failed to read state");
  let balance = state
      .balances
      .iter()
      .find(|(actor, _amount)| actor == &ActorId::from(account.into()))
      .unwrap_or(&(account.into().into(), 0u128))
      .1;

  balance
}