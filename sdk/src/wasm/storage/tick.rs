use crate::types::{fee_growth::FeeGrowth, liquidity::Liquidity, sqrt_price::SqrtPrice};
use decimal::*;
use traceable_result::*;
use crate::Pool;
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Tsify, PartialEq, Eq)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityTick {
    #[tsify(type = "bigint")]
    pub index: i64,
    pub liquidity_change: Liquidity,
    pub sign: bool,
}

impl Default for Tick {
    fn default() -> Self {
        Tick {
            index: 0i32,
            sign: false,
            liquidity_change: Liquidity::from_integer(0),
            liquidity_gross: Liquidity::from_integer(0),
            sqrt_price: SqrtPrice::from_integer(1),
            fee_growth_outside_x: FeeGrowth::from_integer(0),
            fee_growth_outside_y: FeeGrowth::from_integer(0),
            seconds_outside: 0u64,
        }
    }
}

impl LiquidityTick {
    pub fn cross(&mut self, pool: &mut Pool) -> TrackableResult<()> {
        // When going to higher tick net_liquidity should be added and for going lower subtracted
        if (pool.current_tick_index >= self.index) ^ self.sign {
            pool.liquidity = pool
                .liquidity
                .checked_add(self.liquidity_change)
                .map_err(|_| err!("pool.liquidity + tick.liquidity_change overflow"))?;
        } else {
            pool.liquidity = pool
                .liquidity
                .checked_sub(self.liquidity_change)
                .map_err(|_| err!("pool.liquidity - tick.liquidity_change underflow"))?
        }

        Ok(())
    }
}