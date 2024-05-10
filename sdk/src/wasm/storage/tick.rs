use crate::types::{fee_growth::FeeGrowth, liquidity::Liquidity, sqrt_price::SqrtPrice};
use decimal::*;

use crate::alloc::string::ToString;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Tick {
    #[tsify(type = "bigint")]
    pub index: i32,
    pub sign: bool,
    pub liquidity_change: Liquidity,
    pub liquidity_gross: Liquidity,
    pub sqrt_price: SqrtPrice,
    pub fee_growth_outside_x: FeeGrowth,
    pub fee_growth_outside_y: FeeGrowth,
    #[tsify(type = "bigint")]
    pub seconds_outside: u64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PositionTick {
    #[tsify(type = "bigint")]
    pub index: i32,
    pub fee_growth_outside_x: FeeGrowth,
    pub fee_growth_outside_y: FeeGrowth,
    #[tsify(type = "bigint")]
    pub seconds_outside: u64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityTick {
    #[tsify(type = "bigint")]
    pub index: i32,
    pub liquidity_change: Liquidity,
    pub sign: bool,
}

impl Default for Tick {
    fn default() -> Self {
        Tick {
            index: 0i32,
            sign: false,
            liquidity_change: Liquidity::new(0),
            liquidity_gross: Liquidity::new(0),
            sqrt_price: SqrtPrice::from_integer(1),
            fee_growth_outside_x: FeeGrowth::new(0),
            fee_growth_outside_y: FeeGrowth::new(0),
            seconds_outside: 0u64,
        }
    }
}
