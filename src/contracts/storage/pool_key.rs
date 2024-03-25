use gstd::{Decode, Encode, TypeInfo, ActorId};
use crate::{InvariantError, FeeTier};
use math::percentage::Percentage;
use decimal::Decimal;

#[derive(Decode, Encode, TypeInfo, PartialEq, Eq, Clone, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct PoolKey {
  pub token_x: ActorId,
  pub token_y: ActorId,
  pub fee_tier: FeeTier,
}

impl Default for PoolKey {
  fn default() -> Self {
      Self {
          token_x: ActorId::from([0; 32]),
          token_y: ActorId::from([1; 32]),
          fee_tier: FeeTier {
              fee: Percentage::new(0),
              tick_spacing: 1,
          },
      }
  }
}

impl PoolKey {
  pub fn new(
      token_0: ActorId,
      token_1: ActorId,
      fee_tier: FeeTier,
  ) -> Result<Self, InvariantError> {
      if token_0 == token_1 {
          return Err(InvariantError::TokensAreSame);
      }

      if token_0 < token_1 {
          Ok(PoolKey {
              token_x: token_0,
              token_y: token_1,
              fee_tier,
          })
      } else {
          Ok(PoolKey {
              token_x: token_1,
              token_y: token_0,
              fee_tier,
          })
      }
  }
}