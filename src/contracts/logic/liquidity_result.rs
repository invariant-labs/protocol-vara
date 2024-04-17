extern crate alloc;

use decimal::*;
use traceable_result::*;

use math::liquidity::Liquidity;
use math::sqrt_price::{calculate_sqrt_price, SqrtPrice};
use math::token_amount::TokenAmount;
use math::MAX_TICK;

#[derive(Debug)]
pub struct LiquidityResult {
    pub x: TokenAmount,
    pub y: TokenAmount,
    pub l: Liquidity,
}

#[derive(Debug)]
pub struct SingleTokenLiquidity {
    pub l: Liquidity,
    pub amount: TokenAmount,
}

#[allow(dead_code)]
pub fn get_liquidity(
    x: TokenAmount,
    y: TokenAmount,
    lower_tick: i32,
    upper_tick: i32,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
) -> TrackableResult<LiquidityResult> {
    if lower_tick < -MAX_TICK || upper_tick > MAX_TICK {
        return Err(err!("Invalid Ticks"));
    }

    let lower_sqrt_price = ok_or_mark_trace!(calculate_sqrt_price(lower_tick))?;
    let upper_sqrt_price = ok_or_mark_trace!(calculate_sqrt_price(upper_tick))?;

    if upper_sqrt_price < current_sqrt_price {
        // single token y
        let result_by_y = ok_or_mark_trace!(get_liquidity_by_y_sqrt_price(
            y,
            lower_sqrt_price,
            upper_sqrt_price,
            current_sqrt_price,
            rounding_up,
        ))?;
        return Ok(LiquidityResult {
            x: result_by_y.amount,
            y,
            l: result_by_y.l,
        });
    } else if current_sqrt_price < lower_sqrt_price {
        // single token x
        let result_by_x = ok_or_mark_trace!(get_liquidity_by_x_sqrt_price(
            x,
            lower_sqrt_price,
            upper_sqrt_price,
            current_sqrt_price,
            rounding_up,
        ))?;
        return Ok(LiquidityResult {
            x,
            y: result_by_x.amount,
            l: result_by_x.l,
        });
    }
    let result_by_y = ok_or_mark_trace!(get_liquidity_by_y_sqrt_price(
        y,
        lower_sqrt_price,
        upper_sqrt_price,
        current_sqrt_price,
        rounding_up,
    ))?;
    let result_by_x = ok_or_mark_trace!(get_liquidity_by_x_sqrt_price(
        x,
        lower_sqrt_price,
        upper_sqrt_price,
        current_sqrt_price,
        rounding_up,
    ))?;
    Ok(if result_by_y.l < result_by_x.l {
        LiquidityResult {
            x: result_by_y.amount,
            y: result_by_x.amount,
            l: result_by_y.l,
        }
    } else {
        LiquidityResult {
            x: result_by_y.amount,
            y: result_by_x.amount,
            l: result_by_x.l,
        }
    })
}

#[allow(dead_code)]
pub fn get_liquidity_by_x(
    x: TokenAmount,
    lower_tick: i32,
    upper_tick: i32,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
) -> TrackableResult<SingleTokenLiquidity> {
    if lower_tick < -MAX_TICK || upper_tick > MAX_TICK {
        return Err(err!("Invalid Ticks"));
    }

    let lower_sqrt_price = ok_or_mark_trace!(calculate_sqrt_price(lower_tick))?;
    let upper_sqrt_price = ok_or_mark_trace!(calculate_sqrt_price(upper_tick))?;

    ok_or_mark_trace!(get_liquidity_by_x_sqrt_price(
        x,
        lower_sqrt_price,
        upper_sqrt_price,
        current_sqrt_price,
        rounding_up,
    ))
}
pub fn get_liquidity_by_x_sqrt_price(
    x: TokenAmount,
    lower_sqrt_price: SqrtPrice,
    upper_sqrt_price: SqrtPrice,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
) -> TrackableResult<SingleTokenLiquidity> {
    if upper_sqrt_price < current_sqrt_price {
        return Err(err!("Upper Sqrt Price < Current Sqrt Price"));
    }

    if current_sqrt_price < lower_sqrt_price {
        let nominator =
            (lower_sqrt_price.big_mul(upper_sqrt_price)).big_div(SqrtPrice::from_integer(1));
        let denominator = upper_sqrt_price - lower_sqrt_price;
        let liquidity = Liquidity::new(
            (U256::from(x.get())
                * U256::from(nominator.get())
                * U256::from(Liquidity::from_integer(1).get())
                / U256::from(denominator.get()))
            .try_into()
            .map_err(|_| err!("Overflow in calculating liquidity"))?,
        );
        return Ok(SingleTokenLiquidity {
            l: liquidity,
            amount: TokenAmount(0),
        });
    }

    let nominator = current_sqrt_price
        .big_mul(upper_sqrt_price)
        .big_div(SqrtPrice::from_integer(1));
    let denominator = upper_sqrt_price - current_sqrt_price;
    let liquidity = Liquidity::new(
        (U256::from(x.get())
            * U256::from(nominator.get())
            * U256::from(Liquidity::from_integer(1).get())
            / U256::from(denominator.get()))
        .try_into()
        .map_err(|_| err!("Overflow in calculating liquidity"))?,
    );

    let sqrt_price_diff = current_sqrt_price - lower_sqrt_price;
    let y = calculate_y(sqrt_price_diff, liquidity, rounding_up)?;
    Ok(SingleTokenLiquidity {
        l: liquidity,
        amount: y,
    })
}

#[allow(dead_code)]
pub fn get_liquidity_by_y(
    y: TokenAmount,
    lower_tick: i32,
    upper_tick: i32,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
) -> TrackableResult<SingleTokenLiquidity> {
    if lower_tick < -MAX_TICK || upper_tick > MAX_TICK {
        return Err(err!("Invalid Ticks"));
    }

    let lower_sqrt_price = ok_or_mark_trace!(calculate_sqrt_price(lower_tick))?;
    let upper_sqrt_price = ok_or_mark_trace!(calculate_sqrt_price(upper_tick))?;

    ok_or_mark_trace!(get_liquidity_by_y_sqrt_price(
        y,
        lower_sqrt_price,
        upper_sqrt_price,
        current_sqrt_price,
        rounding_up,
    ))
}

#[allow(dead_code)]
pub fn get_liquidity_by_y_sqrt_price(
    y: TokenAmount,
    lower_sqrt_price: SqrtPrice,
    upper_sqrt_price: SqrtPrice,
    current_sqrt_price: SqrtPrice,
    rounding_up: bool,
) -> TrackableResult<SingleTokenLiquidity> {
    if current_sqrt_price < lower_sqrt_price {
        return Err(err!("Current Sqrt Price < Lower Sqrt Price"));
    }

    if upper_sqrt_price <= current_sqrt_price {
        let sqrt_price_diff = upper_sqrt_price - lower_sqrt_price;
        let liquidity = Liquidity::new(
            (U256::from(y.get())
                * U256::from(SqrtPrice::from_integer(1).get())
                * U256::from(Liquidity::from_integer(1).get())
                / U256::from(sqrt_price_diff.get()))
            .try_into()
            .map_err(|_| err!("Overflow while calculating liquidity"))?,
        );
        return Ok(SingleTokenLiquidity {
            l: liquidity,
            amount: TokenAmount::new(0),
        });
    }

    let sqrt_price_diff = current_sqrt_price - lower_sqrt_price;
    let liquidity = Liquidity::new(
        (U256::from(y.get())
            * U256::from(SqrtPrice::from_integer(1).get())
            * U256::from(Liquidity::from_integer(1).get())
            / U256::from(sqrt_price_diff.get()))
        .try_into()
        .map_err(|_| err!("Overflow while calculating liquidity"))?,
    );
    let denominator =
        (current_sqrt_price.big_mul(upper_sqrt_price)).big_div(SqrtPrice::from_integer(1));
    let nominator = upper_sqrt_price - current_sqrt_price;

    let x = calculate_x(nominator, denominator, liquidity, rounding_up)?;

    Ok(SingleTokenLiquidity {
        l: liquidity,
        amount: x,
    })
}

#[allow(dead_code)]
pub fn calculate_x(
    nominator: SqrtPrice,
    denominator: SqrtPrice,
    liquidity: Liquidity,
    rounding_up: bool,
) -> TrackableResult<TokenAmount> {
    let common = liquidity.big_mul(nominator).big_div(denominator).get();

    Ok(if rounding_up {
        TokenAmount::new(
            ((U256::from(common) + U256::from(Liquidity::from_integer(1).get()) - U256::from(1))
                / U256::from(Liquidity::from_integer(1).get()))
            .try_into()
            .map_err(|_| err!("Overflow while casting to TokenAmount"))?,
        )
    } else {
        TokenAmount::new(
            (U256::from(common) / U256::from(Liquidity::from_integer(1).get()))
                .try_into()
                .map_err(|_| err!("Overflow while casting to TokenAmount"))?,
        )
    })
}

pub fn calculate_y(
    sqrt_price_diff: SqrtPrice,
    liquidity: Liquidity,
    rounding_up: bool,
) -> TrackableResult<TokenAmount> {
    let shifted_liquidity = liquidity.get() / Liquidity::from_integer(1).get();
    Ok(if rounding_up {
        TokenAmount::new(
            (((U256::from(sqrt_price_diff.get()) * U256::from(shifted_liquidity))
                + U256::from(SqrtPrice::from_integer(1).get() - 1))
                / U256::from(SqrtPrice::from_integer(1).get()))
            .try_into()
            .map_err(|_| err!("Overflow in calculating TokenAmount"))?,
        )
    } else {
        TokenAmount::new(
            (U256::from(sqrt_price_diff.get()) * U256::from(shifted_liquidity)
                / U256::from(SqrtPrice::from_integer(1).get()))
            .try_into()
            .map_err(|_| err!("Overflow in calculating TokenAmount"))?,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_liquidity_by_x_test() {
        let x = TokenAmount::new(43_0000);
        let current_sqrt_price = calculate_sqrt_price(100).unwrap();
        // below current tick
        {
            let lower_tick = -50;
            let upper_tick = 10;
            let (_, cause, stack) =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, true)
                    .unwrap_err()
                    .get();
            assert_eq!(cause, "Upper Sqrt Price < Current Sqrt Price");
            assert_eq!(stack.len(), 2);
        }
        // in current tick
        {
            let expected_l = Liquidity::new(432392997319297);
            let expected_y_up = TokenAmount(434322);
            let expected_y_down = TokenAmount(434321);

            let lower_tick = 80;
            let upper_tick = 120;

            let result_up =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, true).unwrap();
            assert_eq!(expected_l, result_up.l);
            assert_eq!(expected_y_up, result_up.amount);

            let result_down =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, false).unwrap();

            assert_eq!(expected_l, result_down.l);
            assert_eq!(expected_y_down, result_down.amount);
        }
        // above current tick
        {
            let expected_l = Liquidity::new(13548826311623);
            let expected_y = TokenAmount(0);
            let lower_tick = 150;
            let upper_tick = 800;

            let result_up =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, true).unwrap();
            assert_eq!(expected_l, result_up.l);
            assert_eq!(expected_y, result_up.amount);

            let result_down =
                get_liquidity_by_x(x, lower_tick, upper_tick, current_sqrt_price, false).unwrap();
            assert_eq!(expected_l, result_down.l);
            assert_eq!(expected_y, result_up.amount);
        }
    }

    #[test]
    fn get_liquidity_by_y_test() {
        let y = TokenAmount(47_600_000_000);
        let current_sqrt_price = calculate_sqrt_price(-20000).unwrap();
        // below current tick
        {
            let expected_l = Liquidity::new(2789052279103923275);
            let expected_x = TokenAmount(0);
            let lower_tick = -22000;
            let upper_tick = -21000;

            let result_up =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, true).unwrap();
            assert_eq!(expected_l, result_up.l);
            assert_eq!(expected_x, result_up.amount);
            let result_down =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, false).unwrap();
            assert_eq!(expected_l, result_down.l);
            assert_eq!(expected_x, result_down.amount);
        }
        // in current tick
        {
            let expected_l = Liquidity::new(584945290554346935);
            let expected_x_up = TokenAmount(77539808126);
            let expected_x_down = TokenAmount(77539808125);
            let lower_tick = -25000;
            let upper_tick = -19000;

            let result_up =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, true).unwrap();
            assert_eq!(expected_l, result_up.l);
            assert_eq!(expected_x_up, result_up.amount);
            let result_down =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, false).unwrap();
            assert_eq!(expected_l, result_down.l);
            assert_eq!(expected_x_down, result_down.amount);
        }
        // above current tick
        {
            let lower_tick = -10000;
            let upper_tick = 0;

            let (_, cause, stack) =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, true)
                    .unwrap_err()
                    .get();
            assert_eq!(cause, "Current Sqrt Price < Lower Sqrt Price");
            assert_eq!(stack.len(), 2);
            let (_, cause, stack) =
                get_liquidity_by_y(y, lower_tick, upper_tick, current_sqrt_price, false)
                    .unwrap_err()
                    .get();
            assert_eq!(cause, "Current Sqrt Price < Lower Sqrt Price");
            assert_eq!(stack.len(), 2);
        }
    }

    #[test]
    fn get_liquidity_test() {
        let y = TokenAmount(47_600_000_000);
        let current_sqrt_price = calculate_sqrt_price(-20000).unwrap();

        // below current tick
        {
            let lower_tick = -22000;
            let upper_tick = -21000;
            let expected_x = TokenAmount(0);
            let expected_l = Liquidity::new(2789052279103923275);
            let result_up = get_liquidity(
                expected_x,
                y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
            )
            .unwrap();
            assert_eq!(expected_l, result_up.l);
            assert_eq!(result_up.x, expected_x);

            let result_down = get_liquidity(
                expected_x,
                y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
            )
            .unwrap();
            assert_eq!(expected_l, result_down.l);
            assert_eq!(result_down.x, expected_x);
        }
        // in current tick
        {
            let lower_tick = -25000;
            let upper_tick = -19000;
            let expected_x_up = TokenAmount(77539808126);
            let expected_x_down = TokenAmount(77539808125);
            let expected_l_up = Liquidity::new(584945290554346935);
            let expected_l_down = Liquidity::new(584945290552911923);
            let result_up = get_liquidity(
                expected_x_up,
                y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
            )
            .unwrap();
            assert_eq!(expected_l_up, result_up.l);
            assert_eq!(result_up.x, expected_x_up);

            let result_down = get_liquidity(
                expected_x_down,
                y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                false,
            )
            .unwrap();
            assert_eq!(expected_l_down, result_down.l);
            assert_eq!(result_down.x, expected_x_down);
        }
        // above current tick
        {
            let lower_tick = 150;
            let upper_tick = 800;
            let x = TokenAmount(430_000_000);
            let expected_y = TokenAmount(0);
            let expected_l = Liquidity::new(13548826311623850);
            let result_up = get_liquidity(
                x,
                expected_y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
            )
            .unwrap();
            assert_eq!(expected_l, result_up.l);
            assert_eq!(result_up.y, expected_y);

            let result_down = get_liquidity(
                x,
                expected_y,
                lower_tick,
                upper_tick,
                current_sqrt_price,
                true,
            )
            .unwrap();
            assert_eq!(expected_l, result_down.l);
            assert_eq!(result_down.y, expected_y);
        }
    }
}
