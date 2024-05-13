use super::PoolKey;
use crate::alloc::string::ToString;

use crate::types::{fee_growth::FeeGrowth, liquidity::Liquidity, token_amount::TokenAmount};

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(PartialEq, Default, Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub pool_key: PoolKey,
    pub liquidity: Liquidity,
    #[tsify(type = "bigint")]
    pub lower_tick_index: i32,
    #[tsify(type = "bigint")]
    pub upper_tick_index: i32,
    pub fee_growth_inside_x: FeeGrowth,
    pub fee_growth_inside_y: FeeGrowth,
    #[tsify(type = "bigint")]
    pub last_block_number: u64,
    pub tokens_owed_x: TokenAmount,
    pub tokens_owed_y: TokenAmount,
}
