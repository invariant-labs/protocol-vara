use crate::consts::*;
use crate::types::{liquidity::*, percentage::*, sqrt_price::*, token_amount::*};
use core::convert::TryInto;
use decimal::*;
use js_sys::BigInt;
use serde::{Deserialize, Serialize};
use traceable_result::*;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use wasm_wrapper::*;

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct AmountDeltaResult {
    pub x: TokenAmount,
    pub y: TokenAmount,
    pub update_liquidity: bool,
}
#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SwapResult {
    pub next_sqrt_price: SqrtPrice,
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub fee_amount: TokenAmount,
}

#[wasm_wrapper]
pub fn compute_swap_step(
    current_sqrt_price: SqrtPrice,
    target_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    amount: TokenAmount,
    by_amount_in: bool,
    fee: Percentage,
) -> TrackableResult<SwapResult> {
    if liquidity.is_zero() {
        return Ok(SwapResult {
            next_sqrt_price: target_sqrt_price,
            amount_in: TokenAmount(0),
            amount_out: TokenAmount(0),
            fee_amount: TokenAmount(0),
        });
    }

    let x_to_y = current_sqrt_price >= target_sqrt_price;
    let next_sqrt_price: SqrtPrice;
    let (mut amount_in, mut amount_out) = (TokenAmount(0), TokenAmount(0));

    if by_amount_in {
        let amount_after_fee = amount.big_mul(Percentage::from_integer(1u8) - fee);

        amount_in = ok_or_mark_trace!(if x_to_y {
            get_delta_x(target_sqrt_price, current_sqrt_price, liquidity, true)
        } else {
            get_delta_y(current_sqrt_price, target_sqrt_price, liquidity, true)
        })?;
        // if target sqrt_price was hit it will be the next sqrt_price
        if amount_after_fee >= amount_in {
            next_sqrt_price = target_sqrt_price
        } else {
            next_sqrt_price = ok_or_mark_trace!(get_next_sqrt_price_from_input(
                current_sqrt_price,
                liquidity,
                amount_after_fee,
                x_to_y,
            ))?;
        };
    } else {
        amount_out = ok_or_mark_trace!(if x_to_y {
            get_delta_y(target_sqrt_price, current_sqrt_price, liquidity, false)
        } else {
            get_delta_x(current_sqrt_price, target_sqrt_price, liquidity, false)
        })?;

        if amount >= amount_out {
            next_sqrt_price = target_sqrt_price
        } else {
            next_sqrt_price = ok_or_mark_trace!(get_next_sqrt_price_from_output(
                current_sqrt_price,
                liquidity,
                amount,
                x_to_y
            ))?;
        }
    }

    let not_max = target_sqrt_price != next_sqrt_price;

    if x_to_y {
        if not_max || !by_amount_in {
            amount_in = ok_or_mark_trace!(get_delta_x(
                next_sqrt_price,
                current_sqrt_price,
                liquidity,
                true
            ))?
        };
        if not_max || by_amount_in {
            amount_out = ok_or_mark_trace!(get_delta_y(
                next_sqrt_price,
                current_sqrt_price,
                liquidity,
                false
            ))?
        }
    } else {
        if not_max || !by_amount_in {
            amount_in = ok_or_mark_trace!(get_delta_y(
                current_sqrt_price,
                next_sqrt_price,
                liquidity,
                true
            ))?
        };
        if not_max || by_amount_in {
            amount_out = ok_or_mark_trace!(get_delta_x(
                current_sqrt_price,
                next_sqrt_price,
                liquidity,
                false
            ))?
        };
    }

    // Amount out can not exceed amount
    if !by_amount_in && amount_out > amount {
        amount_out = amount;
    }

    let fee_amount = if by_amount_in && next_sqrt_price != target_sqrt_price {
        amount - amount_in
    } else {
        amount_in.big_mul_up(fee)
    };

    Ok(SwapResult {
        next_sqrt_price,
        amount_in,
        amount_out,
        fee_amount,
    })
}

#[wasm_wrapper]
pub fn get_delta_x(
    sqrt_price_a: SqrtPrice,
    sqrt_price_b: SqrtPrice,
    liquidity: Liquidity,
    rounding_up: bool,
) -> TrackableResult<TokenAmount> {
    let delta_price: SqrtPrice = if sqrt_price_a > sqrt_price_b {
        sqrt_price_a - sqrt_price_b
    } else {
        sqrt_price_b - sqrt_price_a
    };
    let nominator = delta_price.big_mul_to_value(liquidity);

    ok_or_mark_trace!(match rounding_up {
        true => SqrtPrice::big_div_values_to_token_up(
            nominator,
            sqrt_price_a.big_mul_to_value(sqrt_price_b),
        ),
        false => SqrtPrice::big_div_values_to_token(
            nominator,
            sqrt_price_a.big_mul_to_value_up(sqrt_price_b),
        ),
    })
}

#[wasm_wrapper]
pub fn get_delta_y(
    sqrt_price_a: SqrtPrice,
    sqrt_price_b: SqrtPrice,
    liquidity: Liquidity,
    rounding_up: bool,
) -> TrackableResult<TokenAmount> {
    let delta: SqrtPrice = if sqrt_price_a > sqrt_price_b {
        sqrt_price_a - sqrt_price_b
    } else {
        sqrt_price_b - sqrt_price_a
    };

    let delta_y = match rounding_up {
        true => delta
            .big_mul_to_value_up(liquidity)
            .checked_add(SqrtPrice::almost_one())
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(SqrtPrice::one())
            .ok_or_else(|| err!(TrackableError::DIV))?,
        false => delta
            .big_mul_to_value(liquidity)
            .checked_div(SqrtPrice::one())
            .ok_or_else(|| err!(TrackableError::DIV))?,
    };

    Ok(TokenAmount(delta_y.try_into().map_err(|_| {
        err!(TrackableError::cast::<TokenAmount>().as_str())
    })?))
}

#[wasm_wrapper]
pub fn get_next_sqrt_price_from_input(
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    amount: TokenAmount,
    x_to_y: bool,
) -> TrackableResult<SqrtPrice> {
    let result = if x_to_y {
        // add x to pool, decrease sqrt_price
        get_next_sqrt_price_x_up(starting_sqrt_price, liquidity, amount, true)
    } else {
        // add y to pool, increase sqrt_price
        get_next_sqrt_price_y_down(starting_sqrt_price, liquidity, amount, true)
    };
    ok_or_mark_trace!(result)
}

#[wasm_wrapper]
pub fn get_next_sqrt_price_from_output(
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    amount: TokenAmount,
    x_to_y: bool,
) -> TrackableResult<SqrtPrice> {
    let result = if x_to_y {
        // remove y from pool, decrease sqrt_price
        get_next_sqrt_price_y_down(starting_sqrt_price, liquidity, amount, false)
    } else {
        // remove x from pool, increase sqrt_price
        get_next_sqrt_price_x_up(starting_sqrt_price, liquidity, amount, false)
    };
    ok_or_mark_trace!(result)
}

#[wasm_wrapper]
pub fn get_next_sqrt_price_x_up(
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    x: TokenAmount,
    add_x: bool,
) -> TrackableResult<SqrtPrice> {
    if x.is_zero() {
        return Ok(starting_sqrt_price);
    };
    let price_delta = ok_or_mark_trace!(SqrtPrice::checked_from_decimal_to_value(liquidity)
        .map_err(|_| err!("extending liquidity overflow")))?;

    let denominator = ok_or_mark_trace!(match add_x {
        true => price_delta.checked_add(starting_sqrt_price.big_mul_to_value(x)),
        false => price_delta.checked_sub(starting_sqrt_price.big_mul_to_value(x)),
    }
    .ok_or_else(|| err!("big_liquidity -/+ sqrt_price * x")))?; // never should be triggered

    ok_or_mark_trace!(SqrtPrice::checked_big_div_values_up(
        starting_sqrt_price.big_mul_to_value_up(liquidity),
        denominator
    ))
}

#[wasm_wrapper]
pub fn get_next_sqrt_price_y_down(
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    y: TokenAmount,
    add_y: bool,
) -> TrackableResult<SqrtPrice> {
    let numerator: U256 = from_result!(SqrtPrice::checked_from_decimal_to_value(y))?;

    let denominator: U256 = SqrtPrice::checked_from_decimal_to_value(liquidity)
        .map_err(|_| err!("extending liquidity overflow"))?;

    if add_y {
        let quotient =
            ok_or_mark_trace!(SqrtPrice::checked_big_div_values(numerator, denominator))?;
        from_result!(starting_sqrt_price.checked_add(quotient))
    } else {
        let quotient =
            ok_or_mark_trace!(SqrtPrice::checked_big_div_values_up(numerator, denominator))?;
        from_result!(starting_sqrt_price.checked_sub(quotient))
    }
}

#[wasm_wrapper]
pub fn calculate_amount_delta(
    current_tick_index: i32,
    current_sqrt_price: SqrtPrice,
    liquidity_delta: Liquidity,
    liquidity_sign: bool,
    upper_tick: i32,
    lower_tick: i32,
) -> TrackableResult<(TokenAmount, TokenAmount, bool)> {
    if upper_tick < lower_tick {
        return Err(err!("upper_tick is not greater than lower_tick"));
    }
    let mut amount_x = TokenAmount(0);
    let mut amount_y = TokenAmount(0);
    let mut update_liquidity = false;

    if current_tick_index < lower_tick {
        amount_x = ok_or_mark_trace!(get_delta_x(
            ok_or_mark_trace!(SqrtPrice::from_tick(lower_tick))?,
            ok_or_mark_trace!(SqrtPrice::from_tick(upper_tick))?,
            liquidity_delta,
            liquidity_sign,
        ))?;
    } else if current_tick_index < upper_tick {
        amount_x = ok_or_mark_trace!(get_delta_x(
            current_sqrt_price,
            ok_or_mark_trace!(SqrtPrice::from_tick(upper_tick))?,
            liquidity_delta,
            liquidity_sign,
        ))?;
        amount_y = ok_or_mark_trace!(get_delta_y(
            ok_or_mark_trace!(SqrtPrice::from_tick(lower_tick))?,
            current_sqrt_price,
            liquidity_delta,
            liquidity_sign,
        ))?;
        update_liquidity = true;
    } else {
        amount_y = ok_or_mark_trace!(get_delta_y(
            ok_or_mark_trace!(SqrtPrice::from_tick(lower_tick))?,
            ok_or_mark_trace!(SqrtPrice::from_tick(upper_tick))?,
            liquidity_delta,
            liquidity_sign,
        ))?;
    }

    Ok((amount_x, amount_y, update_liquidity))
}

#[wasm_wrapper]
pub fn is_enough_amount_to_change_price(
    amount: TokenAmount,
    starting_sqrt_price: SqrtPrice,
    liquidity: Liquidity,
    fee: Percentage,
    by_amount_in: bool,
    x_to_y: bool,
) -> TrackableResult<bool> {
    if liquidity.is_zero() {
        return Ok(true);
    }

    let next_sqrt_price = ok_or_mark_trace!(if by_amount_in {
        let amount_after_fee = amount.big_mul(Percentage::from_integer(1) - fee);
        get_next_sqrt_price_from_input(starting_sqrt_price, liquidity, amount_after_fee, x_to_y)
    } else {
        get_next_sqrt_price_from_output(starting_sqrt_price, liquidity, amount, x_to_y)
    })?;

    Ok(starting_sqrt_price.ne(&next_sqrt_price))
}

#[wasm_wrapper]
pub fn calculate_max_liquidity_per_tick(tick_spacing: u16) -> Liquidity {
    const MAX_TICKS_AMOUNT_SQRT_PRICE_LIMITED: u128 = 2 * MAX_TICK as u128 + 1;
    let ticks_amount_spacing_limited = MAX_TICKS_AMOUNT_SQRT_PRICE_LIMITED / tick_spacing as u128;
    Liquidity::new(Liquidity::max_instance().get() / ticks_amount_spacing_limited)
}

#[wasm_wrapper]
pub fn check_ticks(tick_lower: i32, tick_upper: i32, tick_spacing: u16) -> TrackableResult<()> {
    if tick_lower > tick_upper {
        return Err(err!("tick_lower > tick_upper"));
    }
    ok_or_mark_trace!(check_tick(tick_lower, tick_spacing))?;
    ok_or_mark_trace!(check_tick(tick_upper, tick_spacing))?;

    Ok(())
}

#[wasm_wrapper]
pub fn check_tick(tick_index: i32, tick_spacing: u16) -> TrackableResult<()> {
    let (min_tick, max_tick) = (get_min_tick(tick_spacing), get_max_tick(tick_spacing));
    let tick_spacing = tick_spacing as i32;
    if tick_index % tick_spacing != 0 {
        return Err(err!("InvalidTickSpacing"));
    }
    if tick_index > max_tick || tick_index < min_tick {
        return Err(err!("InvalidTickIndex"));
    }

    Ok(())
}

#[wasm_wrapper]
pub fn calculate_min_amount_out(
    expected_amount_out: TokenAmount,
    slippage: Percentage,
) -> TokenAmount {
    expected_amount_out.big_mul_up(Percentage::from_integer(1u8) - slippage)
}
