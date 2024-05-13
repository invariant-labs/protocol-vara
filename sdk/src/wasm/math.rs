use crate::{
    liquidity::Liquidity,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
    token_amount::TokenAmount,
    MAX_TICK,
};
use core::convert::TryInto;
use decimal::*;
use serde::{Deserialize, Serialize};
use traceable_result::*;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_wrapper::wasm_wrapper;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SingleTokenLiquidity {
    pub l: Liquidity,
    pub amount: TokenAmount,
}

#[wasm_wrapper]
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

#[wasm_wrapper]
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
