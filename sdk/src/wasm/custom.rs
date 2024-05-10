use crate::{
    storage::{pool_key::PoolKey, tick::Tick},
    types::{
        fee_growth::calculate_fee_growth_inside,
        fee_growth::FeeGrowth,
        liquidity::Liquidity,
        sqrt_price::{get_max_tick, SqrtPrice},
        token_amount::TokenAmount,
    },
    MAX_TICK,
};
use serde::{Deserialize, Serialize};
use traceable_result::{function, location, ok_or_mark_trace, trace, TrackableResult};
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use wasm_wrapper::wasm_wrapper;

#[derive(PartialEq, Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SwapHop {
    pub pool_key: PoolKey,
    pub x_to_y: bool,
}

#[derive(Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResult {
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub target_sqrt_price: SqrtPrice,
    pub ticks: Vec<Tick>,
}

#[derive(Debug, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmounts {
    pub x: TokenAmount,
    pub y: TokenAmount,
}

#[wasm_wrapper("_calculateFee")]
pub fn calculate_fee(
    lower_tick_index: i32,
    lower_tick_fee_growth_outside_x: FeeGrowth,
    lower_tick_fee_growth_outside_y: FeeGrowth,
    upper_tick_index: i32,
    upper_tick_fee_growth_outside_x: FeeGrowth,
    upper_tick_fee_growth_outside_y: FeeGrowth,
    pool_current_tick_index: i32,
    pool_fee_growth_global_x: FeeGrowth,
    pool_fee_growth_global_y: FeeGrowth,
    position_fee_growth_inside_x: FeeGrowth,
    position_fee_growth_inside_y: FeeGrowth,
    position_liquidity: Liquidity,
) -> TrackableResult<(TokenAmount, TokenAmount)> {
    let (fee_growth_inside_x, fee_growth_inside_y) = calculate_fee_growth_inside(
        lower_tick_index,
        lower_tick_fee_growth_outside_x,
        lower_tick_fee_growth_outside_y,
        upper_tick_index,
        upper_tick_fee_growth_outside_x,
        upper_tick_fee_growth_outside_y,
        pool_current_tick_index,
        pool_fee_growth_global_x,
        pool_fee_growth_global_y,
    );

    let tokens_owed_x = ok_or_mark_trace!(fee_growth_inside_x
        .unchecked_sub(position_fee_growth_inside_x)
        .to_fee(position_liquidity))?;
    let tokens_owed_y = ok_or_mark_trace!(fee_growth_inside_y
        .unchecked_sub(position_fee_growth_inside_y)
        .to_fee(position_liquidity))?;

    Ok((tokens_owed_x, tokens_owed_y))
}

#[wasm_wrapper]
pub fn is_token_x(token_candidate: String, token_to_compare: String) -> TrackableResult<bool> {
    Ok(token_candidate < token_to_compare)
}

#[wasm_wrapper("isValidTick")]
pub fn check_tick_to_sqrt_price_relationship(
    tick_index: i32,
    tick_spacing: u16,
    sqrt_price: SqrtPrice,
) -> TrackableResult<bool> {
    if tick_index + tick_spacing as i32 > MAX_TICK {
        let max_tick = get_max_tick(tick_spacing);
        let max_sqrt_price = ok_or_mark_trace!(SqrtPrice::from_tick(max_tick))?;
        if sqrt_price != max_sqrt_price {
            return Ok(false);
        }
    } else {
        let lower_bound = ok_or_mark_trace!(SqrtPrice::from_tick(tick_index))?;
        let upper_bound =
            ok_or_mark_trace!(SqrtPrice::from_tick(tick_index + tick_spacing as i32))?;
        if sqrt_price >= upper_bound || sqrt_price < lower_bound {
            return Ok(false);
        }
    }
    Ok(true)
}
