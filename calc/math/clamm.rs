use decimal::*;
use traceable_result::*;

use crate::consts::*;
use crate::types::{liquidity::*, percentage::*, sqrt_price::*, token_amount::*};

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct SwapResult {
    pub next_sqrt_price: SqrtPrice,
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub fee_amount: TokenAmount,
}

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

fn get_next_sqrt_price_from_input(
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

fn get_next_sqrt_price_from_output(
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

fn get_next_sqrt_price_y_down(
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

pub fn calculate_max_liquidity_per_tick(tick_spacing: u16) -> Liquidity {
    const MAX_TICKS_AMOUNT_SQRT_PRICE_LIMITED: u128 = 2 * MAX_TICK as u128 + 1;
    let ticks_amount_spacing_limited = MAX_TICKS_AMOUNT_SQRT_PRICE_LIMITED / tick_spacing as u128;
    Liquidity::new(Liquidity::max_instance().get() / ticks_amount_spacing_limited)
}

pub fn check_ticks(tick_lower: i32, tick_upper: i32, tick_spacing: u16) -> TrackableResult<()> {
    if tick_lower > tick_upper {
        return Err(err!("tick_lower > tick_upper"));
    }
    ok_or_mark_trace!(check_tick(tick_lower, tick_spacing))?;
    ok_or_mark_trace!(check_tick(tick_upper, tick_spacing))?;

    Ok(())
}

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

pub fn calculate_min_amount_out(
    expected_amount_out: TokenAmount,
    slippage: Percentage,
) -> TokenAmount {
    expected_amount_out.big_mul_up(Percentage::from_integer(1u8) - slippage)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_calculate_min_amount_out() {
        // 0% fee
        {
            let expected_amount_out = TokenAmount(100);
            let slippage = Percentage::from_integer(0);
            let result = calculate_min_amount_out(expected_amount_out, slippage);
            assert_eq!(result, TokenAmount(100));
        }
        // 0.1% fee
        {
            let expected_amount_out = TokenAmount(100);
            let slippage = Percentage::from_scale(1, 3);
            let result = calculate_min_amount_out(expected_amount_out, slippage);
            assert_eq!(result, TokenAmount(100));
        }
        // 0.9% fee
        {
            let expected_amount_out = TokenAmount(123);
            let slippage = Percentage::from_scale(9, 3);
            let result = calculate_min_amount_out(expected_amount_out, slippage);
            assert_eq!(result, TokenAmount(122));
        }
        // 1% fee
        {
            let expected_amount_out = TokenAmount(100);
            let slippage = Percentage::from_scale(1, 2);
            let result = calculate_min_amount_out(expected_amount_out, slippage);
            assert_eq!(result, TokenAmount(99));
        }
        // 3% fee
        {
            let expected_amount_out = TokenAmount(100);
            let slippage = Percentage::from_scale(3, 2);
            let result = calculate_min_amount_out(expected_amount_out, slippage);
            assert_eq!(result, TokenAmount(97));
        }
        // 5% fee
        {
            let expected_amount_out = TokenAmount(100);
            let slippage = Percentage::from_scale(5, 2);
            let result = calculate_min_amount_out(expected_amount_out, slippage);
            assert_eq!(result, TokenAmount(95));
        }
        // 10% fee
        {
            let expected_amount_out = TokenAmount(100);
            let slippage = Percentage::from_scale(1, 1);
            let result = calculate_min_amount_out(expected_amount_out, slippage);
            assert_eq!(result, TokenAmount(90));
        }
        // 20% fee
        {
            let expected_amount_out = TokenAmount(100);
            let slippage = Percentage::from_scale(2, 1);
            let result = calculate_min_amount_out(expected_amount_out, slippage);
            assert_eq!(result, TokenAmount(80));
        }
        // 50% fee
        {
            let expected_amount_out = TokenAmount(100);
            let slippage = Percentage::from_scale(5, 1);
            let result = calculate_min_amount_out(expected_amount_out, slippage);
            assert_eq!(result, TokenAmount(50));
        }
        // 100% fee
        {
            let expected_amount_out = TokenAmount(100);
            let slippage = Percentage::from_integer(1);
            let result = calculate_min_amount_out(expected_amount_out, slippage);
            assert_eq!(result, TokenAmount(0));
        }
    }

    #[test]
    fn test_domain_calculate_min_amount_out() {
        let min_amount = TokenAmount(0);
        let max_amount = TokenAmount::max_instance();
        let min_fee = Percentage::new(0);
        let max_fee = Percentage::from_integer(1);
        // min amount min fee
        {
            let result = calculate_min_amount_out(min_amount, min_fee);
            assert_eq!(result, TokenAmount(0));
        }
        // min amount max fee
        {
            let result = calculate_min_amount_out(min_amount, max_fee);
            assert_eq!(result, TokenAmount(0));
        }
        // max amount max fee
        {
            let result = calculate_min_amount_out(max_amount, max_fee);
            assert_eq!(result, TokenAmount(0));
        }
        // max amount min fee
        {
            let result = calculate_min_amount_out(max_amount, min_fee);
            assert_eq!(result, max_amount);
        }
    }

    #[test]
    fn test_domain_get_next_sqrt_price_from_input() {
        let max_liquidity = Liquidity::max_instance();
        let min_liquidity = Liquidity::new(1);
        let max_amount = TokenAmount::max_instance();
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let almost_min_sqrt_price = min_sqrt_price + SqrtPrice::new(1);
        let almost_max_sqrt_price = max_sqrt_price - SqrtPrice::new(1);

        // max result, increase sqrt_price case
        {
            // get_next_sqrt_price_y_down
            let result = get_next_sqrt_price_from_input(
                almost_max_sqrt_price,
                max_liquidity,
                TokenAmount(600000000),
                false,
            )
            .unwrap();

            assert_eq!(result, SqrtPrice::new(65535383934512647000000000000));
        }
        // min result, decrease sqrt_price case
        {
            // get_next_sqrt_price_x_up
            let result = get_next_sqrt_price_from_input(
                almost_min_sqrt_price,
                max_liquidity,
                TokenAmount(2000000000000000000),
                true,
            )
            .unwrap();

            assert_eq!(result, SqrtPrice::new(15258932000000000000));
        }
        // amount == 0
        {
            let result =
                get_next_sqrt_price_from_input(min_sqrt_price, max_liquidity, TokenAmount(0), true)
                    .unwrap();

            assert_eq!(result, min_sqrt_price);
        }
        // liquidity == 0
        {
            let result = get_next_sqrt_price_from_input(
                min_sqrt_price,
                Liquidity::new(0),
                TokenAmount(20),
                true,
            )
            .unwrap();

            assert_eq!(result, SqrtPrice::new(0));
        }
        // error handling
        {
            let (_, cause, stack) =
                get_next_sqrt_price_from_input(max_sqrt_price, min_liquidity, max_amount, false)
                    .unwrap_err()
                    .get();
            assert_eq!(cause, "multiplication overflow");
            assert_eq!(stack.len(), 3);
        }
    }

    #[test]
    fn test_domain_get_next_sqrt_price_from_output() {
        let max_liquidity = Liquidity::max_instance();
        let min_liquidity = Liquidity::new(1);
        let max_amount = TokenAmount::max_instance();
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let almost_min_sqrt_price = min_sqrt_price + SqrtPrice::new(1);
        let almost_max_sqrt_price = max_sqrt_price - SqrtPrice::new(1);

        // max result, increase sqrt_price case
        {
            // get_next_sqrt_price_x_up
            let result = get_next_sqrt_price_from_output(
                almost_max_sqrt_price,
                max_liquidity,
                TokenAmount(1),
                false,
            )
            .unwrap();

            assert_eq!(result, SqrtPrice::new(65535383934512647000000000012));
        }
        // min result, decrease sqrt_price case
        {
            // get_next_sqrt_price_y_down
            let result = get_next_sqrt_price_from_output(
                almost_min_sqrt_price,
                max_liquidity,
                TokenAmount(1),
                true,
            )
            .unwrap();

            assert_eq!(result, SqrtPrice::new(15258932000000000000));
        }
        // amount == 0
        {
            let result = get_next_sqrt_price_from_output(
                min_sqrt_price,
                max_liquidity,
                TokenAmount(0),
                true,
            )
            .unwrap();

            assert_eq!(result, min_sqrt_price);
        }
        // liquidity == 0
        {
            let (_, cause, stack) = get_next_sqrt_price_from_output(
                min_sqrt_price,
                Liquidity::new(0),
                TokenAmount(20),
                true,
            )
            .unwrap_err()
            .get();

            assert_eq!(cause, "subtraction underflow");
            assert_eq!(stack.len(), 3);
        }
        // error handling
        {
            let (_, cause, stack) =
                get_next_sqrt_price_from_output(max_sqrt_price, min_liquidity, max_amount, false)
                    .unwrap_err()
                    .get();
            assert_eq!(cause, "big_liquidity -/+ sqrt_price * x");
            assert_eq!(stack.len(), 3);
        }
    }

    #[test]
    fn test_compute_swap_step() {
        // VALIDATE BASE SAMPLES
        // one token by amount in
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let target = SqrtPrice::new(1004987562112089027021926);
            let liquidity = Liquidity::from_integer(2000);
            let amount = TokenAmount(1);
            let fee = Percentage::from_scale(6, 4);

            let result =
                compute_swap_step(sqrt_price, target, liquidity, amount, true, fee).unwrap();

            let expected_result = SwapResult {
                next_sqrt_price: sqrt_price,
                amount_in: TokenAmount(0),
                amount_out: TokenAmount(0),
                fee_amount: TokenAmount(1),
            };
            assert_eq!(result, expected_result)
        }
        // amount out capped at target sqrt_price
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let target = SqrtPrice::new(1004987562112089027021926);
            let liquidity = Liquidity::from_integer(2000);
            let amount = TokenAmount(20);
            let fee = Percentage::from_scale(6, 4);

            let result_in =
                compute_swap_step(sqrt_price, target, liquidity, amount, true, fee).unwrap();
            let result_out =
                compute_swap_step(sqrt_price, target, liquidity, amount, false, fee).unwrap();

            let expected_result = SwapResult {
                next_sqrt_price: target,
                amount_in: TokenAmount(10),
                amount_out: TokenAmount(9),
                fee_amount: TokenAmount(1),
            };
            assert_eq!(result_in, expected_result);
            assert_eq!(result_out, expected_result);
        }
        // amount in not capped
        {
            let sqrt_price = SqrtPrice::from_scale(101, 2);
            let target = SqrtPrice::from_integer(10);
            let liquidity = Liquidity::from_integer(300000000);
            let amount = TokenAmount(1000000);
            let fee = Percentage::from_scale(6, 4);

            let result =
                compute_swap_step(sqrt_price, target, liquidity, amount, true, fee).unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: SqrtPrice::new(1_013_331_333_333_333_333_333_333),
                amount_in: TokenAmount(999400),
                amount_out: TokenAmount(976487), // ((1.013331333333 - 1.01) * 300000000) / (1.013331333333 * 1.01)
                fee_amount: TokenAmount(600),
            };
            assert_eq!(result, expected_result)
        }
        // amount out not capped
        {
            let sqrt_price = SqrtPrice::from_integer(101);
            let target = SqrtPrice::from_integer(100);
            let liquidity = Liquidity::from_integer(5000000000000u128);
            let amount = TokenAmount(2000000);
            let fee = Percentage::from_scale(6, 4);

            let result =
                compute_swap_step(sqrt_price, target, liquidity, amount, false, fee).unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: SqrtPrice::new(100_999_999_600_000_000_000_000_000),
                amount_in: TokenAmount(197), // (5000000000000 * (101 - 100.9999996)) /  (101 * 100.9999996)
                amount_out: amount,
                fee_amount: TokenAmount(1),
            };
            assert_eq!(result, expected_result)
        }
        // empty swap step when sqrt_price is at tick
        {
            let current_sqrt_price = SqrtPrice::new(999500149965_000000000000);
            let target_sqrt_price = SqrtPrice::new(999500149965_000000000000);

            let liquidity = Liquidity::new(20_006_000_000_000);
            let amount = TokenAmount(1_000_000);
            let by_amount_in = true;
            let fee = Percentage::from_scale(6, 4); // 0.0006 -> 0.06%

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: current_sqrt_price,
                amount_in: TokenAmount(0),
                amount_out: TokenAmount(0),
                fee_amount: TokenAmount(0),
            };
            assert_eq!(result, expected_result)
        }
        // if liquidity is high, small amount in should not push sqrt_price
        {
            let current_sqrt_price = SqrtPrice::from_scale(999500149965u128, 12);
            let target_sqrt_price = SqrtPrice::from_scale(1999500149965u128, 12);
            let liquidity = Liquidity::from_integer(100_000000000000_000000000000u128);
            let amount = TokenAmount(10);
            let by_amount_in = true;
            let fee = Percentage::from_scale(6, 4); // 0.0006 -> 0.06%

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: current_sqrt_price,
                amount_in: TokenAmount(0),
                amount_out: TokenAmount(0),
                fee_amount: TokenAmount(10),
            };
            assert_eq!(result, expected_result)
        }
        // amount_in > u64 for swap to target sqrt_price and when liquidity > 2^64
        {
            let current_sqrt_price = SqrtPrice::from_integer(1);
            let target_sqrt_price = SqrtPrice::from_scale(100005, 5); // 1.00005
            let liquidity = Liquidity::from_integer(368944000000_000000000000u128);
            let amount = TokenAmount(1);
            let by_amount_in = true;
            let fee = Percentage::from_scale(6, 4); // 0.0006 -> 0.06%

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: current_sqrt_price,
                amount_in: TokenAmount(0),
                amount_out: TokenAmount(0),
                fee_amount: TokenAmount(1),
            };
            assert_eq!(result, expected_result)
        }
        // amount_out > u64 for swap to target sqrt_price and when liquidity > 2^64
        {
            let current_sqrt_price = SqrtPrice::from_integer(1);
            let target_sqrt_price = SqrtPrice::from_scale(100005, 5); // 1.00005
            let liquidity = Liquidity::from_integer(368944000000_000000000000u128);
            let amount = TokenAmount(1);
            let by_amount_in = false;
            let fee = Percentage::from_scale(6, 4); // 0.0006 -> 0.06%

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: SqrtPrice::new(1_000000000000_000000000003),
                amount_in: TokenAmount(2),
                amount_out: TokenAmount(1),
                fee_amount: TokenAmount(1),
            };
            assert_eq!(result, expected_result)
        }
        // liquidity is zero and by amount_in should skip to target sqrt_price
        {
            let current_sqrt_price = SqrtPrice::from_integer(1);
            let target_sqrt_price = SqrtPrice::from_scale(100005, 5); // 1.00005
            let liquidity = Liquidity::new(0);
            let amount = TokenAmount(100000);
            let by_amount_in = true;
            let fee = Percentage::from_scale(6, 4); // 0.0006 -> 0.06%

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: target_sqrt_price,
                amount_in: TokenAmount(0),
                amount_out: TokenAmount(0),
                fee_amount: TokenAmount(0),
            };
            assert_eq!(result, expected_result)
        }
        // liquidity is zero and by amount_out should skip to target sqrt_price
        {
            let current_sqrt_price = SqrtPrice::from_integer(1);
            let target_sqrt_price = SqrtPrice::from_scale(100005, 5); // 1.00005
            let liquidity = Liquidity::new(0);
            let amount = TokenAmount(100000);
            let by_amount_in = false;
            let fee = Percentage::from_scale(6, 4); // 0.0006 -> 0.06%

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: target_sqrt_price,
                amount_in: TokenAmount(0),
                amount_out: TokenAmount(0),
                fee_amount: TokenAmount(0),
            };
            assert_eq!(result, expected_result)
        }
        // normal swap step but fee is set to 0
        {
            let current_sqrt_price = SqrtPrice::from_scale(99995, 5); // 0.99995
            let target_sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(50000000);
            let amount = TokenAmount(1000);
            let by_amount_in = true;
            let fee = Percentage::new(0);

            let result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                amount,
                by_amount_in,
                fee,
            )
            .unwrap();
            let expected_result = SwapResult {
                next_sqrt_price: SqrtPrice::from_scale(99997, 5),
                amount_in: TokenAmount(1000),
                amount_out: TokenAmount(1000),
                fee_amount: TokenAmount(0),
            };
            assert_eq!(result, expected_result)
        }
        // by_amount_out and x_to_y edge cases
        {
            let target_sqrt_price = SqrtPrice::from_tick(-10).unwrap();
            let current_sqrt_price = target_sqrt_price + SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(340282366920938463463374607u128);
            let one_token = TokenAmount(1);
            let tokens_with_same_output = TokenAmount(85);
            let zero_token = TokenAmount(0);
            let by_amount_in = false;
            let max_fee = Percentage::from_scale(9, 1);
            let min_fee = Percentage::from_integer(0);

            let one_token_result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                one_token,
                by_amount_in,
                max_fee,
            )
            .unwrap();
            let tokens_with_same_output_result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                tokens_with_same_output,
                by_amount_in,
                max_fee,
            )
            .unwrap();
            let zero_token_result = compute_swap_step(
                current_sqrt_price,
                target_sqrt_price,
                liquidity,
                zero_token,
                by_amount_in,
                min_fee,
            )
            .unwrap();
            /*
                86x -> [1, 85]y
                rounding due to sqrt_price accuracy
                it does not matter if you want 1 or 85 y tokens, will take you the same input amount
            */
            let expected_one_token_result = SwapResult {
                next_sqrt_price: current_sqrt_price - SqrtPrice::new(1),
                amount_in: TokenAmount(86),
                amount_out: TokenAmount(1),
                fee_amount: TokenAmount(78),
            };
            let expected_tokens_with_same_output_result = SwapResult {
                next_sqrt_price: current_sqrt_price - SqrtPrice::new(1),
                amount_in: TokenAmount(86),
                amount_out: TokenAmount(85),
                fee_amount: TokenAmount(78),
            };
            let expected_zero_token_result = SwapResult {
                next_sqrt_price: current_sqrt_price,
                amount_in: TokenAmount(0),
                amount_out: TokenAmount(0),
                fee_amount: TokenAmount(0),
            };
            assert_eq!(one_token_result, expected_one_token_result);
            assert_eq!(
                tokens_with_same_output_result,
                expected_tokens_with_same_output_result
            );
            assert_eq!(zero_token_result, expected_zero_token_result);
        }
    }

    #[test]
    fn test_domain_compute_swap_step() {
        let one_sqrt_price = SqrtPrice::from_integer(1);
        let two_sqrt_price = SqrtPrice::from_integer(2);
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        let one_liquidity = Liquidity::from_integer(1);
        let max_liquidity = Liquidity::max_instance();
        let max_amount = TokenAmount::max_instance();
        let max_amount_not_reached_target_sqrt_price = TokenAmount(TokenAmount::max_value() - 1);
        let max_fee = Percentage::from_integer(1);
        let min_fee = Percentage::new(0);

        // 100% fee | max_amount
        {
            let result = compute_swap_step(
                one_sqrt_price,
                two_sqrt_price,
                one_liquidity,
                max_amount,
                true,
                max_fee,
            )
            .unwrap();
            assert_eq!(
                result,
                SwapResult {
                    next_sqrt_price: SqrtPrice::from_integer(1),
                    amount_in: TokenAmount(0),
                    amount_out: TokenAmount(0),
                    fee_amount: max_amount,
                }
            )
        }
        // 0% fee | max_amount | max_liquidity | sqrt_price slice
        {
            let result = compute_swap_step(
                one_sqrt_price,
                two_sqrt_price,
                max_liquidity,
                max_amount,
                true,
                min_fee,
            )
            .unwrap();

            assert_eq!(
                result,
                SwapResult {
                    next_sqrt_price: SqrtPrice::new(2000000000000000000000000),
                    amount_in: TokenAmount(340282366920938463463374607431769),
                    amount_out: TokenAmount(170141183460469231731687303715884),
                    fee_amount: TokenAmount(0)
                }
            )
        }
        // by_amount_in == true || close to target_sqrt_price but not reached
        {
            let big_liquidity = Liquidity::from_integer(100_000_000_000_000u128);
            let amount_pushing_sqrt_price_to_target = TokenAmount(100000000000000);

            let result = compute_swap_step(
                one_sqrt_price,
                two_sqrt_price,
                big_liquidity,
                amount_pushing_sqrt_price_to_target - TokenAmount(1),
                true,
                min_fee,
            )
            .unwrap();
            assert_eq!(
                result,
                SwapResult {
                    next_sqrt_price: SqrtPrice::new(1999999999999990000000000),
                    amount_in: TokenAmount(99999999999999),
                    amount_out: TokenAmount(49999999999999),
                    fee_amount: TokenAmount(0)
                }
            )
        }
        // maximize fee_amount || close to target_sqrt_price but not reached
        {
            let expected_result = SwapResult {
                next_sqrt_price: SqrtPrice::new(1000018999999999999999999),
                amount_in: TokenAmount(6465364971497830805463835175),
                amount_out: TokenAmount(6465242131897324756293472063),
                fee_amount: TokenAmount(340282366914473098491876776626304376280),
            };

            let result = compute_swap_step(
                one_sqrt_price,
                two_sqrt_price,
                max_liquidity,
                TokenAmount::max_instance(),
                true,
                max_fee - Percentage::new(19),
            )
            .unwrap();
            assert_eq!(result, expected_result)
        }
        // get_next_sqrt_price_from_input -> get_next_sqrt_price_x_up
        {
            // by_amount_in == true
            // x_to_y == true => current_sqrt_price >= target_sqrt_price == true

            let result = compute_swap_step(
                max_sqrt_price,
                min_sqrt_price,
                max_liquidity,
                max_amount_not_reached_target_sqrt_price,
                true,
                min_fee,
            )
            .unwrap();

            assert_eq!(
                result,
                SwapResult {
                    next_sqrt_price: SqrtPrice::new(15258932000000000000),
                    amount_in: TokenAmount(22300536291904886527033853306200674438),
                    amount_out: TokenAmount(22300535557116062863569555195614450423),
                    fee_amount: TokenAmount(0)
                }
            )
        }

        // get_next_sqrt_price_from_input -> get_next_sqrt_price_y_down
        {
            // by_amount_in == true
            // x_to_y == false => current_sqrt_price >= target_sqrt_price == false

            // 1. scale - maximize amount_after_fee => (max_amount, min_fee) && not reached target
            {
                let result = compute_swap_step(
                    min_sqrt_price,
                    max_sqrt_price,
                    max_liquidity,
                    max_amount_not_reached_target_sqrt_price,
                    true,
                    min_fee,
                )
                .unwrap();

                assert_eq!(
                    result,
                    SwapResult {
                        next_sqrt_price: SqrtPrice::new(65535383934512647000000000000),
                        amount_in: TokenAmount(22300535557116062863569555195614450424),
                        amount_out: TokenAmount(22300536291904886527033853306200674437),
                        fee_amount: TokenAmount(0)
                    }
                )
            }
            // 2. checked_big_div - no possible to trigger from compute_swap_step
            {
                let min_overflow_token_amount = TokenAmount::new(340282366920939);
                let result = compute_swap_step(
                    min_sqrt_price,
                    max_sqrt_price,
                    one_liquidity - Liquidity::new(1),
                    min_overflow_token_amount - TokenAmount(1),
                    true,
                    min_fee,
                )
                .unwrap();
                assert_eq!(
                    result,
                    SwapResult {
                        next_sqrt_price: max_sqrt_price,
                        amount_in: TokenAmount(65536),
                        amount_out: TokenAmount(65535),
                        fee_amount: TokenAmount(0),
                    }
                )
            }
        }
        // get_next_sqrt_price_from_output -> get_next_sqrt_price_x_up
        {
            // by_amount_in == false
            // x_to_y == false => current_sqrt_price >= target_sqrt_price == false
            // TRY TO UNWRAP IN SUBTRACTION

            // min_sqrt_price different at maximum amount
            {
                let min_diff = 232_826_265_438_719_159_684u128;
                let result = compute_swap_step(
                    max_sqrt_price - SqrtPrice::new(min_diff),
                    max_sqrt_price,
                    max_liquidity,
                    TokenAmount(TokenAmount::max_value() - 1),
                    false,
                    min_fee,
                )
                .unwrap();

                assert_eq!(
                    result,
                    SwapResult {
                        next_sqrt_price: SqrtPrice::new(65535383934512647000000000000),
                        amount_in: TokenAmount(79226672684850046813853155300),
                        amount_out: TokenAmount(18446744073709551615),
                        fee_amount: TokenAmount(0)
                    }
                )
                // assert_eq!(cause, "multiplication overflow");
                // assert_eq!(stack.len(), 4);
            }
            // min sqrt_price different at maximum amount
            {
                let result = compute_swap_step(
                    min_sqrt_price,
                    max_sqrt_price,
                    Liquidity::from_integer(281_477_613_507_675u128),
                    TokenAmount(TokenAmount::max_value() - 1),
                    false,
                    min_fee,
                )
                .unwrap();

                assert_eq!(
                    result,
                    SwapResult {
                        next_sqrt_price: SqrtPrice::new(65535383934512647000000000000),
                        amount_in: TokenAmount(18446743465900796471),
                        amount_out: TokenAmount(18446744073709559494),
                        fee_amount: TokenAmount(0)
                    }
                );
            }
            // min token change
            {
                let result = compute_swap_step(
                    max_sqrt_price - SqrtPrice::from_integer(1),
                    max_sqrt_price,
                    Liquidity::from_integer(10_000_000_000u128),
                    TokenAmount(1),
                    false,
                    min_fee,
                )
                .unwrap();

                assert_eq!(
                    result,
                    SwapResult {
                        next_sqrt_price: SqrtPrice::new(65534813412874974599766965330u128),
                        amount_in: TokenAmount(4294783624),
                        amount_out: TokenAmount(1),
                        fee_amount: TokenAmount(0),
                    }
                );
            }
        }
        // maximizalize amount_out, by_amount_in == false
        {
            let result = compute_swap_step(
                max_sqrt_price,
                min_sqrt_price,
                max_liquidity,
                max_amount,
                false,
                min_fee,
            )
            .unwrap();

            assert_eq!(
                result,
                SwapResult {
                    next_sqrt_price: SqrtPrice::new(15258932000000000000),
                    amount_in: TokenAmount(22300536291904886527033853306200674438),
                    amount_out: TokenAmount(22300535557116062863569555195614450423),
                    fee_amount: TokenAmount(0)
                }
            )
        }
    }

    #[test]
    fn test_get_next_sqrt_price_y_down() {
        // VALIDATE BASE SAMPLES
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(1);
            let y = TokenAmount(1);

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, true).unwrap();

            assert_eq!(result, SqrtPrice::from_integer(2));
        }
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(2);
            let y = TokenAmount(3);

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, true).unwrap();

            assert_eq!(result, SqrtPrice::from_scale(25, 1));
        }
        {
            let sqrt_price = SqrtPrice::from_integer(2);
            let liquidity = Liquidity::from_integer(3);
            let y = TokenAmount(5);

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, true).unwrap();

            assert_eq!(
                result,
                SqrtPrice::from_integer(11).big_div(SqrtPrice::from_integer(3))
            );
        }
        {
            let sqrt_price = SqrtPrice::from_integer(24234);
            let liquidity = Liquidity::from_integer(3000);
            let y = TokenAmount(5000);

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, true).unwrap();

            assert_eq!(
                result,
                SqrtPrice::from_integer(72707).big_div(SqrtPrice::from_integer(3))
            );
        }
        // bool = false
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(2);
            let y = TokenAmount(1);

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, false).unwrap();

            assert_eq!(result, SqrtPrice::from_scale(5, 1));
        }
        {
            let sqrt_price = SqrtPrice::from_integer(100_000);
            let liquidity = Liquidity::from_integer(500_000_000);
            let y = TokenAmount(4_000);

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, false).unwrap();
            assert_eq!(
                result,
                SqrtPrice::new(99_999_999_992_000_000_000_000_000_000)
            );
        }
        {
            let sqrt_price = SqrtPrice::from_integer(3);
            let liquidity = Liquidity::from_integer(222);
            let y = TokenAmount(37);

            let result = get_next_sqrt_price_y_down(sqrt_price, liquidity, y, false).unwrap();

            // expected 2.833333333333
            // real     2.999999999999833...
            assert_eq!(result, SqrtPrice::new(2_833_333_333_333_333_333_333_333));
        }
    }

    #[test]
    fn test_domain_get_next_sqrt_price_y_down() {
        let min_y = TokenAmount::new(1);
        let max_y = TokenAmount::max_instance();
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let almost_min_sqrt_price = min_sqrt_price + SqrtPrice::new(1);
        let almost_max_sqrt_price = max_sqrt_price - SqrtPrice::new(1);
        let min_sqrt_price_outside_domain = SqrtPrice::new(1);
        let min_liquidity = Liquidity::new(1);
        let max_liquidity: Liquidity = Liquidity::max_instance();
        // let min_overflow_token_y = TokenAmount::new(340282366920939);
        let min_overflow_token_y = TokenAmount::new(340282366920940);
        // let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let one_liquidity: Liquidity = Liquidity::from_integer(1);

        // Max token y is 2^96 to not cause intermediate overflow
        let min_y_overflow_decimal_extenstion = TokenAmount::new(1 << 96);

        // min value inside domain
        {
            // increases min_sqrt_price
            {
                let target_sqrt_price =
                     // get_next_sqrt_price_y_down(min_sqrt_price, max_liquidity, min_y, true).unwrap();
                     get_next_sqrt_price_y_down(min_sqrt_price, max_liquidity, min_y + TokenAmount(600000000), true).unwrap();

                assert_eq!(target_sqrt_price, SqrtPrice::new(15258932000000000001));
            }
            // decreases almost_min_sqrt_price
            {
                let target_sqrt_price =
                    get_next_sqrt_price_y_down(almost_min_sqrt_price, max_liquidity, min_y, false)
                        .unwrap();

                assert_eq!(target_sqrt_price, SqrtPrice::new(15258932000000000000));
            }
        }
        // max value inside domain
        {
            // decreases max_sqrt_price
            {
                let target_sqrt_price =
                    get_next_sqrt_price_y_down(max_sqrt_price, max_liquidity, min_y, false)
                        .unwrap();

                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(65535383934512646999999999999)
                );
            }
            // increases almost_max_sqrt_price
            {
                let target_sqrt_price: SqrtPrice = get_next_sqrt_price_y_down(
                    almost_max_sqrt_price,
                    max_liquidity,
                    min_y + TokenAmount(600000000),
                    true,
                )
                .unwrap();

                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(65535383934512647000000000000)
                );
            }
        }
        // extension TokenAmount to SqrtPrice decimal overflow
        {
            {
                let (_, cause, stack) =
                    get_next_sqrt_price_y_down(max_sqrt_price, min_liquidity, max_y, true)
                        .unwrap_err()
                        .get();
                assert_eq!(cause, "multiplication overflow");
                assert_eq!(stack.len(), 2);
            }
            {
                let (_, cause, stack) = get_next_sqrt_price_y_down(
                    min_sqrt_price_outside_domain,
                    min_liquidity,
                    max_y,
                    false,
                )
                .unwrap_err()
                .get();
                assert_eq!(
                    cause,
                    "conversion to math::types::sqrt_price::SqrtPrice type failed"
                );
                assert_eq!(stack.len(), 2);
            }
        }
        // overflow in sqrt_price difference
        {
            {
                let result = get_next_sqrt_price_y_down(
                    max_sqrt_price,
                    one_liquidity,
                    min_overflow_token_y - TokenAmount(2),
                    true,
                )
                .unwrap_err();
                let (_, cause, stack) = result.get();
                assert_eq!(cause, "checked_add: (self + rhs) additional overflow");
                assert_eq!(stack.len(), 1);
            }
            {
                let result = get_next_sqrt_price_y_down(
                    min_sqrt_price_outside_domain,
                    one_liquidity,
                    min_overflow_token_y - TokenAmount(2),
                    false,
                )
                .unwrap_err();
                let (_, cause, stack) = result.get();
                assert_eq!(cause, "checked_sub: (self - rhs) subtraction underflow");
                assert_eq!(stack.len(), 1);
            }
        }

        // quotient overflow
        // max params to max result
        // unwrap_err on result
        // min liq max amou, maxsqrt_price
        // min sqrt highest underflow
        {
            {
                let irrelevant_sqrt_price = SqrtPrice::new(1);
                let irrelevant_liquidity: Liquidity = Liquidity::from_integer(1);

                {
                    let (_, cause, stack) = get_next_sqrt_price_y_down(
                        irrelevant_sqrt_price,
                        irrelevant_liquidity,
                        min_y_overflow_decimal_extenstion,
                        true,
                    )
                    .unwrap_err()
                    .get();
                    assert_eq!(
                        cause,
                        "conversion to math::types::sqrt_price::SqrtPrice type failed"
                    );
                    assert_eq!(stack.len(), 2);
                }
                {
                    let (_, cause, stack) = get_next_sqrt_price_y_down(
                        irrelevant_sqrt_price,
                        irrelevant_liquidity,
                        min_y_overflow_decimal_extenstion,
                        false,
                    )
                    .unwrap_err()
                    .get();
                    assert_eq!(
                        cause,
                        "conversion to math::types::sqrt_price::SqrtPrice type failed"
                    );
                    assert_eq!(stack.len(), 2);
                }
            }
        }
        // y_max
        {
            {
                let (_, cause, stack) =
                    get_next_sqrt_price_y_down(min_sqrt_price, max_liquidity, max_y, true)
                        .unwrap_err()
                        .get();
                assert_eq!(cause, "multiplication overflow");
                assert_eq!(stack.len(), 2);
            }
        }

        // L == 0
        {
            {
                let (_, cause, stack) =
                    get_next_sqrt_price_y_down(min_sqrt_price, Liquidity::new(0), min_y, true)
                        .unwrap_err()
                        .get();
                assert_eq!(cause, "division overflow or division by zero");
                assert_eq!(stack.len(), 2);
            }
        }
        // TokenAmount is zero
        {
            {
                let target_sqrt_price =
                    get_next_sqrt_price_y_down(min_sqrt_price, max_liquidity, TokenAmount(0), true)
                        .unwrap();

                assert_eq!(target_sqrt_price, min_sqrt_price);
            }
        }
    }

    #[test]
    fn test_get_delta_x() {
        // validate base samples
        // zero at zero liquidity
        {
            let result = get_delta_x(
                SqrtPrice::from_integer(1u8),
                SqrtPrice::from_integer(1u8),
                Liquidity::new(0),
                false,
            )
            .unwrap();
            assert_eq!(result, TokenAmount(0));
        }
        // equal at equal liquidity
        {
            let result = get_delta_x(
                SqrtPrice::from_integer(1u8),
                SqrtPrice::from_integer(2u8),
                Liquidity::from_integer(2u8),
                false,
            )
            .unwrap();
            assert_eq!(result, TokenAmount(1));
        }
        // complex
        {
            let sqrt_price_a = SqrtPrice::new(234_878_324_943_782_000_000_000_000);
            let sqrt_price_b = SqrtPrice::new(87_854_456_421_658_000_000_000_000);
            let liquidity = Liquidity::new(983_983_249_092);

            let result_down = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, false).unwrap();
            let result_up = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, true).unwrap();

            // 7010.8199533068819376891841727789301497024557314488455622925765280
            assert_eq!(result_down, TokenAmount(7010));
            assert_eq!(result_up, TokenAmount(7011));
        }
        // big
        {
            let sqrt_price_a = SqrtPrice::from_integer(1u8);
            let sqrt_price_b = SqrtPrice::from_scale(5u8, 1);
            let liquidity = Liquidity::from_integer(2u128.pow(64) - 1);

            let result_down = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, false).unwrap();
            let result_up = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, true).unwrap();

            assert_eq!(result_down, TokenAmount::from_decimal(liquidity));
            assert_eq!(result_up, TokenAmount::from_decimal(liquidity));
        }
        // no more overflow after extending the type to U320 in intermediate operations
        {
            let sqrt_price_a = SqrtPrice::from_integer(1u8);
            let sqrt_price_b = SqrtPrice::from_scale(5u8, 1);
            let liquidity = Liquidity::max_instance();

            let result_down = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, false);
            let result_up = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, true);

            assert!(result_down.is_ok());
            assert!(result_up.is_ok());
        }
        // huge liquidity
        {
            let sqrt_price_a = SqrtPrice::from_integer(1u8);
            let sqrt_price_b = SqrtPrice::new(SqrtPrice::one()) + SqrtPrice::new(1000000);
            let liquidity = Liquidity::from_integer(2u128.pow(80));

            let result_down = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, false);
            let result_up = get_delta_x(sqrt_price_a, sqrt_price_b, liquidity, true);

            assert!(result_down.is_ok());
            assert!(result_up.is_ok());
        }
    }

    #[test]
    fn test_domain_get_delta_x() {
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        // let almost_max_sqrt_price = SqrtPrice::from_tick(MAX_TICK - 1);
        let almost_min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK + 1).unwrap();

        let max_liquidity = Liquidity::max_instance();
        let min_liquidity = Liquidity::new(1);

        // maximize delta_sqrt_price and liquidity
        {
            {
                let result =
                    get_delta_x(max_sqrt_price, min_sqrt_price, max_liquidity, true).unwrap();

                assert_eq!(result, TokenAmount(22300536291904886527033853306200674438));
            }
            {
                let result =
                    get_delta_x(max_sqrt_price, min_sqrt_price, max_liquidity, false).unwrap();

                assert_eq!(result, TokenAmount(22300536291904886527033853306200674437))
            }
        }
        {
            {
                let result =
                    get_delta_x(max_sqrt_price, min_sqrt_price, min_liquidity, true).unwrap();

                assert_eq!(result, TokenAmount(1));
            }
            {
                let result =
                    get_delta_x(max_sqrt_price, min_sqrt_price, min_liquidity, false).unwrap();

                assert_eq!(result, TokenAmount(0))
            }
        }
        // minimize denominator on maximize liquidity which fit into TokenAmount
        {
            {
                let result =
                    get_delta_x(min_sqrt_price, almost_min_sqrt_price, max_liquidity, true)
                        .unwrap();
                assert_eq!(TokenAmount(1115049101222874255702226300908362), result);
            }
            {
                let result =
                    get_delta_x(min_sqrt_price, almost_min_sqrt_price, max_liquidity, false)
                        .unwrap();
                assert_eq!(TokenAmount(1115049101222874255702226300908361), result);
            }
        }
        // minimize denominator on minimize liquidity which fits into TokenAmount
        {
            {
                let result =
                    get_delta_x(min_sqrt_price, almost_min_sqrt_price, min_liquidity, true)
                        .unwrap();
                assert_eq!(TokenAmount(1), result);
            }
            {
                let result =
                    get_delta_x(min_sqrt_price, almost_min_sqrt_price, min_liquidity, false)
                        .unwrap();
                assert_eq!(TokenAmount(0), result);
            }
        }

        {
            let search_limit = 256;
            let tick_spacing = 100;
            let max_search_limit = MAX_TICK - (search_limit * tick_spacing);
            let min_search_sqrt_price = SqrtPrice::from_tick(max_search_limit).unwrap();
            let liquidity = Liquidity::max_instance();

            let result =
                get_delta_x(max_sqrt_price, min_search_sqrt_price, liquidity, true).unwrap();
            /*
                    search_limit 256 * tick_spacing (max 100)
                    MAX_TICK <-> MAX_TICK - (serach_limit * tick_spacing)
                    sqrt(1.0001^MAX_TICK) * 10^24 -> sqrt(1.0001^(MAX_TICK - SEARCH_LIMIT * MAX_TICK_SPACING)) * 10^24

                    MAX_TICK - SEARCH_LIMIT * MAX_TICK_SPACING = 196218
                    ceil(log2(max_sqrt_price)) < 96
                    ceil(log2(min_search_price)) < 94

                    max_nominator = (sqrt(1.0001)^221818 - sqrt(1.0001)^196218) * 10^24 * 2^128 / 10^6
                    max_nominator < 2^204
                    max_nominator_intermediate = (sqrt(1.0001)^221818 - sqrt(1.0001)^196218) * 10^24 * 2^128
                    max_nominator < 2^224

                    denominator = (sqrt(1.0001)^221818 - sqrt(1.0001)^196218) * 10^24
                    denominator = 2^96

                    max_big_div_values_to_token_up = ((max_nominator * SqrtPrice::one() + denominaotr - 1) / denominaotr + SqrtPrice::almost_one()) / SqrtPrice::one()
                    max_big_div_values_to_token_up = ((2^204 * 10^24 + 2^96 - 1) / 2^96 + 10^24) / 10^24
                    max_big_div_values_to_token_up < 2^108

                    max_big_div_values_to_token_up_indermediate = (max_nominator * SqrtPrice::one() + denominaotr
                    max_big_div_values_to_token_up_indermediate = 2^204 * 10^24 + 2^96
                    max_big_div_values_to_token_up_indermediate < 2^284 <-- no more overflow  after adding U320
            */
            assert_eq!(result, TokenAmount(13481455942966627077028504118))
        }
        {
            let almost_max_sqrt_price = max_sqrt_price.checked_sub(SqrtPrice::new(1)).unwrap(); // max_sqrt_price.checked_sub(min_step).unwrap();
            let almost_min_sqrt_price = min_sqrt_price.checked_add(SqrtPrice::new(1)).unwrap(); //min_sqrt_price.checked_add(min_step).unwrap();

            // max_sqrt_price -> max_sqrt_price - 10^-24  /  max liqudiity
            {
                let result =
                    get_delta_x(max_sqrt_price, almost_max_sqrt_price, max_liquidity, true)
                        .unwrap();
                assert_eq!(TokenAmount(1), result);
            }

            // min_sqrt_price -> min_sqrt_price + 10^-24 / max liqudity

            {
                let result =
                    get_delta_x(min_sqrt_price, almost_min_sqrt_price, max_liquidity, true)
                        .unwrap();
                assert_eq!(TokenAmount(1461474256330471373), result);
            }
        }
        // liquidity is zero
        {
            {
                let result =
                    get_delta_x(max_sqrt_price, min_sqrt_price, Liquidity::new(0), true).unwrap();
                assert_eq!(TokenAmount(0), result);
            }
            {
                let result =
                    get_delta_x(max_sqrt_price, min_sqrt_price, Liquidity::new(0), false).unwrap();
                assert_eq!(TokenAmount(0), result);
            }
        }
    }
    #[test]
    fn test_get_delta_y() {
        // base samples
        // zero at zero liquidity
        {
            let result = get_delta_y(
                SqrtPrice::from_integer(1),
                SqrtPrice::from_integer(1),
                Liquidity::new(0),
                false,
            )
            .unwrap();
            assert_eq!(result, TokenAmount(0));
        }
        // equal at equal liquidity
        {
            let result = get_delta_y(
                SqrtPrice::from_integer(1),
                SqrtPrice::from_integer(2),
                Liquidity::from_integer(2),
                false,
            )
            .unwrap();
            assert_eq!(result, TokenAmount(2));
        }
        // big numbers
        {
            let sqrt_price_a = SqrtPrice::new(234_878_324_943_782_000_000_000_000);
            let sqrt_price_b = SqrtPrice::new(87_854_456_421_658_000_000_000_000);
            let liquidity = Liquidity::new(983_983_249_092);

            let result_down = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, false).unwrap();
            let result_up = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, true).unwrap();

            // 144669023.842474597804911408
            assert_eq!(result_down, TokenAmount(144669023));
            assert_eq!(result_up, TokenAmount(144669024));
        }
        // big
        {
            let sqrt_price_a = SqrtPrice::from_integer(1u8);
            let sqrt_price_b = SqrtPrice::from_integer(2u8);
            let liquidity = Liquidity::from_integer(2u128.pow(64) - 1);

            let result_down = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, false).unwrap();
            let result_up = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, true).unwrap();

            assert_eq!(result_down, TokenAmount::from_decimal(liquidity));
            assert_eq!(result_up, TokenAmount::from_decimal(liquidity));
        }
        // overflow
        {
            let sqrt_price_a = SqrtPrice::from_integer(1u8);
            let sqrt_price_b = SqrtPrice::max_instance();
            let liquidity = Liquidity::max_instance();

            let result_down = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, false);
            let result_up = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, true);

            assert!(result_down.is_err());
            assert!(result_up.is_err());
        }
        // huge liquidity
        {
            let sqrt_price_a = SqrtPrice::from_integer(1u8);
            let sqrt_price_b = SqrtPrice::new(SqrtPrice::one()) + SqrtPrice::new(1000000);
            let liquidity = Liquidity::from_integer(2u128.pow(80));

            let result_down = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, false);
            let result_up = get_delta_y(sqrt_price_a, sqrt_price_b, liquidity, true);

            assert!(result_down.is_ok());
            assert!(result_up.is_ok());
        }
    }

    #[test]
    fn test_domain_get_delta_y() {
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        let max_liquidity = Liquidity::max_instance();
        let min_liquidity = Liquidity::new(1);
        // maximize delta_sqrt_price and liquidity
        {
            {
                let result =
                    get_delta_y(max_sqrt_price, min_sqrt_price, max_liquidity, true).unwrap();
                assert_eq!(result, TokenAmount(22300535557116062863569555195614450424));
            }
            {
                let result =
                    get_delta_y(max_sqrt_price, min_sqrt_price, max_liquidity, false).unwrap();
                assert_eq!(result, TokenAmount(22300535557116062863569555195614450423));
            }
            // can be zero
            {
                let result = get_delta_y(
                    max_sqrt_price,
                    SqrtPrice::new(max_sqrt_price.get() - 1),
                    min_liquidity,
                    false,
                )
                .unwrap();
                assert_eq!(result, TokenAmount(0));
            }
        }
        // liquidity is zero
        {
            let result =
                get_delta_y(max_sqrt_price, min_sqrt_price, Liquidity::new(0), true).unwrap();
            assert_eq!(result, TokenAmount(0));
        }
        {
            let result = get_delta_y(max_sqrt_price, max_sqrt_price, max_liquidity, true).unwrap();
            assert_eq!(result, TokenAmount(0));
        }
    }

    #[test]
    fn test_get_next_sqrt_price_x_up() {
        // basic samples
        // Add
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(1);
            let x = TokenAmount(1);

            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, true);

            assert_eq!(result.unwrap(), SqrtPrice::from_scale(5, 1));
        }
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(2);
            let x = TokenAmount(3);

            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, true);

            assert_eq!(result.unwrap(), SqrtPrice::from_scale(4, 1));
        }
        {
            let sqrt_price = SqrtPrice::from_integer(2);
            let liquidity = Liquidity::from_integer(3);
            let x = TokenAmount(5);

            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, true);

            assert_eq!(
                result.unwrap(),
                SqrtPrice::new(461538461538461538461539) // rounded up Decimal::from_integer(6).div(Decimal::from_integer(13))
            );
        }
        {
            let sqrt_price = SqrtPrice::from_integer(24234);
            let liquidity = Liquidity::from_integer(3000);
            let x = TokenAmount(5000);

            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, true);

            assert_eq!(
                result.unwrap(),
                SqrtPrice::new(599985145205615112277488) // rounded up Decimal::from_integer(24234).div(Decimal::from_integer(40391))
            );
        }
        // Subtract
        {
            let sqrt_price = SqrtPrice::from_integer(1);
            let liquidity = Liquidity::from_integer(2);
            let x = TokenAmount(1);

            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, false);

            assert_eq!(result.unwrap(), SqrtPrice::from_integer(2));
        }
        {
            let sqrt_price = SqrtPrice::from_integer(100_000);
            let liquidity = Liquidity::from_integer(500_000_000);
            let x = TokenAmount(4_000);

            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, false);

            assert_eq!(result.unwrap(), SqrtPrice::from_integer(500_000));
        }
        {
            let sqrt_price = SqrtPrice::new(3_333333333333333333333333);
            let liquidity = Liquidity::new(222_222222);
            let x = TokenAmount(37);

            // expected 7.490636713462104974072145
            // real     7.4906367134621049740721443...
            let result = get_next_sqrt_price_x_up(sqrt_price, liquidity, x, false);

            assert_eq!(result.unwrap(), SqrtPrice::new(7490636713462104974072145));
        }
    }

    #[test]
    fn test_domain_get_next_sqrt_price_x_up() {
        // DOMAIN:
        let max_liquidity = Liquidity::max_instance();
        let min_liquidity = Liquidity::new(1);
        // let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK);
        let max_x = TokenAmount::max_instance();
        let min_x = TokenAmount::new(1);
        let min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let almost_min_sqrt_price = min_sqrt_price + SqrtPrice::new(1);
        let almost_max_sqrt_price = max_sqrt_price - SqrtPrice::new(1);
        // min value inside domain
        {
            // increases min_sqrt_price
            {
                let target_sqrt_price =
                              // get_next_sqrt_price_y_down(min_sqrt_price, max_liquidity, min_x, true).unwrap();
                              get_next_sqrt_price_x_up(min_sqrt_price, max_liquidity, TokenAmount(600000000), false).unwrap();

                assert_eq!(target_sqrt_price, SqrtPrice::new(15258932000000000001));
            }
            // decreases almost_min_sqrt_price
            {
                let target_sqrt_price = get_next_sqrt_price_x_up(
                    almost_min_sqrt_price,
                    max_liquidity,
                    TokenAmount(2000000000000000000),
                    true,
                )
                .unwrap();

                assert_eq!(target_sqrt_price, SqrtPrice::new(15258932000000000000));
            }
        }
        // max value inside domain
        {
            // decreases max_sqrt_price
            {
                let target_sqrt_price =
                    get_next_sqrt_price_x_up(max_sqrt_price, max_liquidity, min_x, true).unwrap();

                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(65535383934512646999999999988)
                );
            }
            // increases almost_max_sqrt_price
            {
                let target_sqrt_price: SqrtPrice =
                    get_next_sqrt_price_x_up(almost_max_sqrt_price, max_liquidity, min_x, false)
                        .unwrap();

                assert_eq!(
                    target_sqrt_price,
                    SqrtPrice::new(65535383934512647000000000012)
                );
            }
        }
        {
            let result =
                get_next_sqrt_price_x_up(max_sqrt_price, max_liquidity, max_x, true).unwrap();

            assert_eq!(result, SqrtPrice::new(999999999984741068));
        }
        // subtraction underflow (not possible from upper-level function)
        {
            let (_, cause, stack) =
                get_next_sqrt_price_x_up(max_sqrt_price, min_liquidity, max_x, false)
                    .unwrap_err()
                    .get();

            assert_eq!(cause, "big_liquidity -/+ sqrt_price * x");
            assert_eq!(stack.len(), 2);
        }
        // max possible result test
        {
            let result =
                get_next_sqrt_price_x_up(max_sqrt_price, max_liquidity, min_x, true).unwrap();

            assert_eq!(result, SqrtPrice::new(65535383934512646999999999988));
        }
        // Liquidity is zero
        {
            let result =
                get_next_sqrt_price_x_up(max_sqrt_price, Liquidity::new(0), min_x, true).unwrap();

            assert_eq!(result, SqrtPrice::new(0));
        }
        // Amount is zero
        {
            let result =
                get_next_sqrt_price_x_up(max_sqrt_price, max_liquidity, TokenAmount(0), true)
                    .unwrap();

            assert_eq!(result, SqrtPrice::new(65535383934512647000000000000));
        }
    }

    #[test]
    fn test_is_enough_amount_to_push_price() {
        // Validate traceable error
        let min_liquidity = Liquidity::new(1);
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let min_fee = Percentage::from_integer(0);
        {
            let (_, cause, stack) = is_enough_amount_to_change_price(
                TokenAmount::max_instance(),
                max_sqrt_price,
                min_liquidity,
                min_fee,
                false,
                false,
            )
            .unwrap_err()
            .get();

            assert_eq!(cause, "big_liquidity -/+ sqrt_price * x");
            assert_eq!(stack.len(), 4);
        }
    }

    #[test]
    fn test_domain_is_enoguh_amount_to_push_price() {
        let min_liquidity = Liquidity::new(1);
        let zero_liquidity = Liquidity::new(0);
        let max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
        let min_fee = Percentage::from_integer(0);
        let max_fee = Percentage::from_integer(1);
        let max_amount = TokenAmount::max_instance();
        let min_amount = TokenAmount(1);

        // Percentage Max
        {
            let (_, cause, stack) = is_enough_amount_to_change_price(
                min_amount,
                max_sqrt_price,
                min_liquidity,
                max_fee,
                false,
                false,
            )
            .unwrap_err()
            .get();

            assert_eq!(cause, "big_liquidity -/+ sqrt_price * x");
            assert_eq!(stack.len(), 4);
        }

        // Liquidity is 0
        {
            let result = is_enough_amount_to_change_price(
                max_amount,
                max_sqrt_price,
                zero_liquidity,
                max_fee,
                false,
                false,
            )
            .unwrap();
            assert!(result)
        }
        // Amount Min
        {
            let (_, cause, stack) = is_enough_amount_to_change_price(
                min_amount,
                max_sqrt_price,
                min_liquidity,
                min_fee,
                false,
                false,
            )
            .unwrap_err()
            .get();

            assert_eq!(cause, "big_liquidity -/+ sqrt_price * x");
            assert_eq!(stack.len(), 4);
        }
        // Amount Max
        {
            let (_, cause, stack) = is_enough_amount_to_change_price(
                max_amount,
                max_sqrt_price,
                min_liquidity,
                min_fee,
                false,
                false,
            )
            .unwrap_err()
            .get();

            assert_eq!(cause, "big_liquidity -/+ sqrt_price * x");
            assert_eq!(stack.len(), 4);
        }
    }

    #[test]
    fn test_calculate_amount_delta() {
        // current tick between lower tick and upper tick
        {
            let current_tick_index = 2;
            let current_sqrt_price = SqrtPrice::new(1_000_140_000_000_000_000_000_000);

            let liquidity_delta = Liquidity::from_integer(5_000_000);
            let liquidity_sign = true;
            let upper_tick = 3;
            let lower_tick = 0;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                liquidity_delta,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();

            assert_eq!(x, TokenAmount(51));
            assert_eq!(y, TokenAmount(700));
            assert!(add)
        }
        {
            let current_tick_index = 2;
            let current_sqrt_price = SqrtPrice::new(1_000_140_000_000_000_000_000_000);

            let liquidity_delta = Liquidity::from_integer(5_000_000);
            let liquidity_sign = true;
            let upper_tick = 4;
            let lower_tick = 0;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                liquidity_delta,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();

            assert_eq!(x, TokenAmount(300));
            assert_eq!(y, TokenAmount(700));
            assert!(add)
        }
        // current tick smaller than lower tick
        {
            let current_tick_index = 0;
            let current_sqrt_price = Default::default();
            let liquidity_delta = Liquidity::from_integer(10);
            let liquidity_sign = true;
            let upper_tick = 4;
            let lower_tick = 2;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                liquidity_delta,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();

            assert_eq!(x, TokenAmount(1));
            assert_eq!(y, TokenAmount(0));
            assert!(!add)
        }
        // current tick greater than upper tick
        {
            let current_tick_index = 6;
            let current_sqrt_price = Default::default();

            let liquidity_delta = Liquidity::from_integer(10);
            let liquidity_sign = true;
            let upper_tick = 4;
            let lower_tick = 2;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                liquidity_delta,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();

            assert_eq!(x, TokenAmount(0));
            assert_eq!(y, TokenAmount(1));
            assert!(!add)
        }
    }

    #[test]
    fn test_domain_calculate_amount_delta() {
        // DOMAIN
        let max_liquidity = Liquidity::max_instance();

        // maximalize x
        {
            let current_tick_index = -MAX_TICK;
            let current_sqrt_price = Default::default();

            let liquidity_sign = true;
            let upper_tick = MAX_TICK;
            let lower_tick = -MAX_TICK + 1;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                max_liquidity,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();
            assert_eq!(x, TokenAmount(22299421242803663652778151079899766076));
            assert_eq!(y, TokenAmount(0)); // assert_eq!(y, TokenAmount(1));
            assert!(!add)
        }

        // maximalize y
        {
            let current_tick_index = MAX_TICK;
            let current_sqrt_price = Default::default();
            let liquidity_sign = true;
            let upper_tick = MAX_TICK - 1;
            let lower_tick = -MAX_TICK;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                max_liquidity,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();
            assert_eq!(x, TokenAmount(0));
            assert_eq!(y, TokenAmount(22299420613894225688327846143610371474));
            assert!(!add)
        }

        // delta liquidity = 0
        {
            let current_tick_index = 2;
            let current_sqrt_price = SqrtPrice::new(1_000_140_000_000_000_000_000_000);

            let liquidity_delta = Liquidity::from_integer(0);
            let liquidity_sign = true;
            let upper_tick = 4;
            let lower_tick = 0;

            let (x, y, add) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                liquidity_delta,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap();

            assert_eq!(x, TokenAmount(0));
            assert_eq!(y, TokenAmount(0));
            assert!(add)
        }

        // Error handling
        {
            let current_tick_index = 0;
            let current_sqrt_price = SqrtPrice::new(1_000_140_000_000_000_000_000_000);
            let liquidity_delta = Liquidity::from_integer(0);
            let liquidity_sign = true;
            let upper_tick = 4;
            let lower_tick = 10;

            let (_, cause, stack) = calculate_amount_delta(
                current_tick_index,
                current_sqrt_price,
                liquidity_delta,
                liquidity_sign,
                upper_tick,
                lower_tick,
            )
            .unwrap_err()
            .get();
            assert_eq!(cause, "upper_tick is not greater than lower_tick");
            assert_eq!(stack.len(), 1);
        }
        {
            let max_sqrt_price = SqrtPrice::max_instance(); // 2^128 - 1
            let max_liquidity = Liquidity::max_instance();
            {
                let current_tick_index = 0;
                let current_sqrt_price = max_sqrt_price;
                let liquidity_sign = true;
                let upper_tick = MAX_TICK;
                let lower_tick = -MAX_TICK;

                let (_, cause, stack) = calculate_amount_delta(
                    current_tick_index,
                    current_sqrt_price,
                    max_liquidity,
                    liquidity_sign,
                    upper_tick,
                    lower_tick,
                )
                .unwrap_err()
                .get();
                assert_eq!(
                    cause,
                    "conversion to math::types::token_amount::TokenAmount type failed"
                );
                assert_eq!(stack.len(), 2)
            }
        }
    }

    #[test]
    fn test_calculate_max_liquidity_per_tick() {
        // tick_spacing 1 [L_MAX / 443_637]
        {
            let max_l = calculate_max_liquidity_per_tick(1);
            assert_eq!(max_l, Liquidity::new(767028825190275976673213928125400));
        };
        // tick_spacing 2 [L_MAX / 221_819]
        {
            let max_l = calculate_max_liquidity_per_tick(2);
            assert_eq!(max_l, Liquidity::new(1534061108300221187926023169588438));
        }
        // tick_spacing 5 [L_MAX / 88_727]
        {
            let max_l = calculate_max_liquidity_per_tick(5);
            assert_eq!(max_l, Liquidity::new(3835161415588698631345301964810804));
        }
        // tick_spacing 100 [L_MAX / 4436]
        {
            let max_l = calculate_max_liquidity_per_tick(100);
            assert_eq!(max_l, Liquidity::new(76709280189571339824926647302021688));
        }
    }
}
