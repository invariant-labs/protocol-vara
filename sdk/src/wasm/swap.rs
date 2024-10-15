use crate::clamm::compute_swap_step;
use crate::sqrt_price::{get_max_tick, get_min_tick, SqrtPrice};
use crate::token_amount::TokenAmount;
use crate::{
    FeeTier, LiquidityTick, Pool, SimulateSwapResult, Tickmap, UpdatePoolTick,
    MAX_SQRT_PRICE, MIN_SQRT_PRICE,
};
use crate::consts::MAX_SWAP_STEPS;
use decimal::*;
use traceable_result::TrackableResult;
use traceable_result::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_wrapper::wasm_wrapper;

type LiquidityTicks = Vec<LiquidityTick>;

#[wasm_wrapper("_simulateInvariantSwap")]
pub fn simulate_invariant_swap(
    tickmap: Tickmap,
    fee_tier: FeeTier,
    mut pool: Pool,
    ticks: LiquidityTicks,
    x_to_y: bool,
    amount: TokenAmount,
    by_amount_in: bool,
    sqrt_price_limit: SqrtPrice,
) -> TrackableResult<SimulateSwapResult> {
    if amount.is_zero() {
        return Err(err!("Amount is zero"));
    }

    if x_to_y {
        if pool.sqrt_price <= sqrt_price_limit
            || sqrt_price_limit > SqrtPrice::new(MAX_SQRT_PRICE.into())
        {
            return Err(err!("Wrong limit"));
        }
    } else if pool.sqrt_price >= sqrt_price_limit
        || sqrt_price_limit < SqrtPrice::new(MIN_SQRT_PRICE.into())
    {
        return Err(err!("Wrong limit"));
    }

    let tick_limit = if x_to_y {
        get_min_tick(fee_tier.tick_spacing as u16)?
    } else {
        get_max_tick(fee_tier.tick_spacing as u16)?
    };

    let start_sqrt_price = pool.sqrt_price;

    let mut swap_step_number = 0;
    let mut max_swap_steps_reached = false;
    let mut global_insufficient_liquidity = false;
    let mut state_outdated = false;

    let mut crossed_ticks: Vec<LiquidityTick> = vec![];
    let mut remaining_amount = amount;
    let mut total_amount_in = TokenAmount(U256::zero());
    let mut total_amount_out = TokenAmount(U256::zero());
    let mut total_fee_amount = TokenAmount(U256::zero());

    while !remaining_amount.is_zero() {
        let closer_limit = tickmap.get_closer_limit(
            sqrt_price_limit,
            x_to_y,
            pool.current_tick_index as i32,
            fee_tier.tick_spacing as u16,
        );
        let (swap_limit, limiting_tick) = if let Ok(closer_limit) = closer_limit {
            closer_limit
        } else {
            global_insufficient_liquidity = true;
            break;
        };

        let result = compute_swap_step(
            pool.sqrt_price,
            swap_limit,
            pool.liquidity,
            remaining_amount,
            by_amount_in,
            fee_tier.fee,
        )?;
        swap_step_number += 1;

        // make remaining amount smaller
        if by_amount_in {
            remaining_amount -= result.amount_in + result.fee_amount;
        } else {
            remaining_amount -= result.amount_out;
        }

        total_fee_amount += result.fee_amount;

        pool.sqrt_price = result.next_sqrt_price;

        total_amount_in += result.amount_in + result.fee_amount;
        total_amount_out += result.amount_out;

        // Fail if price would go over swap limit
        if pool.sqrt_price == sqrt_price_limit && !remaining_amount.is_zero() {
            global_insufficient_liquidity = true;
            break;
        }

        let mut tick_update = {
            if let Some((tick_index, is_initialized)) = limiting_tick {
                if is_initialized {
                    let tick = ticks.iter().find(|t| t.index as i32 == tick_index);

                    match tick {
                        Some(tick) => UpdatePoolTick::TickInitialized(*tick),
                        None => {
                            state_outdated = true;

                            break;
                        }
                    }
                } else {
                    UpdatePoolTick::TickUninitialized(tick_index as i64)
                }
            } else {
                UpdatePoolTick::NoTick
            }
        };

        let tick_update_return = pool.update_tick(
            result,
            swap_limit,
            &mut tick_update,
            remaining_amount,
            by_amount_in,
            x_to_y,
            fee_tier,
        );
        let (amount_to_add, amount_after_tick_update, has_crossed) =
            if let Ok(tick_update_return) = tick_update_return {
                tick_update_return
            } else {
                state_outdated = true;
                break;
            };

        remaining_amount = amount_after_tick_update;
        total_amount_in += amount_to_add;

        if let UpdatePoolTick::TickInitialized(tick) = tick_update {
            if has_crossed {
                crossed_ticks.push(tick);
            }
        }

        let reached_tick_limit = match x_to_y {
            true => pool.current_tick_index <= tick_limit as i64,
            false => pool.current_tick_index >= tick_limit as i64,
        };

        if reached_tick_limit {
            global_insufficient_liquidity = true;
            break;
        }

        if swap_step_number > MAX_SWAP_STEPS {
            max_swap_steps_reached = true;
            break;
        }
    }

    return Ok(SimulateSwapResult {
        amount_in: total_amount_in,
        amount_out: total_amount_out,
        start_sqrt_price,
        target_sqrt_price: pool.sqrt_price,
        fee: total_fee_amount,
        crossed_ticks,
        global_insufficient_liquidity,
        max_swap_steps_reached,
        state_outdated
    });
}
