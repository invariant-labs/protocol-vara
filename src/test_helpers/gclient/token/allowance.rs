use gstd::ActorId;
use gclient::GearApi;
use fungible_token_io::*;

#[allow(dead_code)]
pub async fn allowance(
  api: &GearApi,
  token: impl Into<[u8; 32]> + gstd::Copy,
  from: impl Into<[u8; 32]> + gstd::Copy,
  signer: impl Into<[u8; 32]> + gstd::Copy,
)->u128 {
  let state = api
      .read_state::<IoFungibleToken>(token.into().into(), [].into())
      .await
      .expect("Failed to read state");
  let allowances = state
      .allowances
      .iter()
      .find(|(actor, _)| actor == &ActorId::from(from.into()));

  if let Some(allowances) = allowances {
    let allowance = allowances.1.iter()
        .find(|(actor, _amount)| actor == &ActorId::from(signer.into()))
        .unwrap_or(&(signer.into().into(), 0u128))
        .1;
    return allowance;
  } else {
    return 0;
  }

}