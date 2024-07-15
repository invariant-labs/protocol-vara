use crate::alloc::string::{String, ToString};
use crate::types::{
    fee_growth::FeeGrowth, liquidity::Liquidity, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};
use crate::*;
use decimal::*;
use serde::{Deserialize, Serialize};
use traceable_result::*;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Pool {
    pub liquidity: Liquidity,
    pub sqrt_price: SqrtPrice,
    #[tsify(type = "bigint")]
    pub current_tick_index: i64,
    pub fee_growth_global_x: FeeGrowth,
    pub fee_growth_global_y: FeeGrowth,
    pub fee_protocol_token_x: TokenAmount,
    pub fee_protocol_token_y: TokenAmount,
    #[tsify(type = "bigint")]
    pub start_timestamp: u64,
    #[tsify(type = "bigint")]
    pub last_timestamp: u64,
    #[tsify(type = "string")]
    pub fee_receiver: String,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UpdatePoolTick {
    NoTick,
    TickInitialized(LiquidityTick),
    TickUninitialized(i64),
}
impl Pool {
    pub fn update_tick(
        &mut self,
        result: SwapResult,
        swap_limit: SqrtPrice,
        tick: &mut UpdatePoolTick,
        mut remaining_amount: TokenAmount,
        by_amount_in: bool,
        x_to_y: bool,
        fee_tier: FeeTier,
    ) -> TrackableResult<(TokenAmount, TokenAmount, bool)> {
        let mut has_crossed = false;
        let mut total_amount = TokenAmount::new(U256::from(0));

        if UpdatePoolTick::NoTick == *tick || swap_limit != result.next_sqrt_price {
            self.current_tick_index = unwrap!(get_tick_at_sqrt_price(
                result.next_sqrt_price,
                fee_tier.tick_spacing as u16
            )) as i64;

            return Ok((total_amount, remaining_amount, has_crossed));
        };

        let is_enough_amount_to_cross = unwrap!(is_enough_amount_to_change_price(
            remaining_amount,
            result.next_sqrt_price,
            self.liquidity,
            fee_tier.fee,
            by_amount_in,
            x_to_y,
        ));

        let tick_index = match tick {
            UpdatePoolTick::TickInitialized(tick) => {
                if !x_to_y || is_enough_amount_to_cross {
                    tick.cross(self)?;
                    has_crossed = true;
                } else if !remaining_amount.is_zero() {
                    if by_amount_in {
                        total_amount = remaining_amount;
                    }
                    remaining_amount = TokenAmount::new(U256::from(0));
                }

                tick.index
            }
            UpdatePoolTick::TickUninitialized(index) => *index,
            _ => unreachable!(),
        };

        self.current_tick_index = if x_to_y && is_enough_amount_to_cross {
            tick_index - fee_tier.tick_spacing as i64
        } else {
            tick_index
        };

        Ok((total_amount, remaining_amount, has_crossed))
    }

    pub fn update_liquidity(
        &mut self,
        liquidity_delta: Liquidity,
        liquidity_sign: bool,
        upper_tick: i32,
        lower_tick: i32,
    ) -> TrackableResult<(TokenAmount, TokenAmount)> {
        let (x, y, update_liquidity) = ok_or_mark_trace!(calculate_amount_delta(
            self.current_tick_index as i32,
            self.sqrt_price,
            liquidity_delta,
            liquidity_sign,
            upper_tick,
            lower_tick,
        ))?;

        if !update_liquidity {
            return Ok((x, y));
        }

        if liquidity_sign {
            self.liquidity = self
                .liquidity
                .checked_add(liquidity_delta)
                .map_err(|_| err!("update_liquidity: liquidity + liquidity_delta overflow"))?;
            Ok((x, y))
        } else {
            self.liquidity = self
                .liquidity
                .checked_sub(liquidity_delta)
                .map_err(|_| err!("update_liquidity: liquidity - liquidity_delta underflow"))?;
            Ok((x, y))
        }
    }
}
