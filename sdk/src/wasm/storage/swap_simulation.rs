use crate::*;
use crate::{token_amount::TokenAmount, sqrt_price::SqrtPrice};
use tsify::Tsify;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Default, Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SimulateSwapResult {
  pub amount_in: TokenAmount,
  pub amount_out: TokenAmount,
  pub fee: TokenAmount,
  pub start_sqrt_price: SqrtPrice,
  pub target_sqrt_price: SqrtPrice,
  pub crossed_ticks: Vec<LiquidityTick>,
  pub global_insufficient_liquidity: bool,
  pub max_swap_steps_reached: bool,
  pub state_outdated: bool,
}

