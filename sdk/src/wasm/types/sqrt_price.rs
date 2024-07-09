use crate::consts::*;
use crate::types::{fixed_point::FixedPoint, token_amount::TokenAmount};
use crate::{convert, decimal_ops};
use core::convert::{TryFrom, TryInto};
use decimal::*;
use js_sys::BigInt;
use serde::{Deserialize, Serialize};
use traceable_result::*;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use wasm_wrapper::wasm_wrapper;

#[decimal(24, U384T)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SqrtPrice(#[tsify(type = "bigint")] pub u128);

decimal_ops!(SqrtPrice);

impl SqrtPrice {
    pub fn from_tick(i: i32) -> TrackableResult<Self> {
        calculate_sqrt_price(i)
    }

    pub fn big_div_values_to_token(
        nominator: U384T,
        denominator: U384T,
    ) -> TrackableResult<TokenAmount> {
        let nominator: U448T = SqrtPrice::from_value(nominator);
        let denominator: U448T = SqrtPrice::from_value(denominator);

        let intermediate = nominator
            .checked_mul(SqrtPrice::one().cast())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let casted_intermediate: U384T = (SqrtPrice::checked_from_value(intermediate))
            .map_err(|_| err!("Can't parse from U448T to U384T"))?;

        let result = casted_intermediate
            .checked_div(SqrtPrice::one().cast())
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let casted_result: U256 = TokenAmount::checked_from_value::<U256, U384T>(result)
            .map_err(|_| err!("Can't parse from U384T to U256T"))?;

        Ok(TokenAmount::new(casted_result))
    }

    pub fn big_div_values_to_token_up(
        nominator: U384T,
        denominator: U384T,
    ) -> TrackableResult<TokenAmount> {
        let nominator: U448T = SqrtPrice::from_value(nominator);
        let denominator: U448T = SqrtPrice::from_value(denominator);

        let intermediate = nominator
            .checked_mul(SqrtPrice::one().cast())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_add(denominator - 1)
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let casted_intermediate: U384T = (SqrtPrice::checked_from_value(intermediate))
            .map_err(|_| err!("Can't parse from U448T to U384T"))?;

        let result = casted_intermediate
            .checked_add(Self::almost_one().cast())
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(SqrtPrice::one().cast())
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let casted_result: U256 = TokenAmount::checked_from_value::<U256, U384T>(result)
            .map_err(|_| err!("Can't parse from U384T to U256T"))?;

        Ok(TokenAmount::new(casted_result))
    }

    // TODO - Configure nominator and denominator types
    pub fn big_div_values_up(nominator: U384T, denominator: U384T) -> SqrtPrice {
        let result = nominator
            .checked_mul(Self::one().cast())
            .unwrap()
            .checked_add(denominator.checked_sub(U384T::from(1u32)).unwrap())
            .unwrap()
            .checked_div(denominator)
            .unwrap();
        let casted_result = SqrtPrice::from_value::<u128, U384T>(result);
        SqrtPrice::new(casted_result)
    }

    pub fn checked_big_div_values(
        nominator: U448T,
        denominator: U448T,
    ) -> TrackableResult<SqrtPrice> {
        let result = nominator
            .checked_mul(Self::one().cast())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let casted_result = SqrtPrice::checked_from_value::<u128, U448T>(result)
            .map_err(|_| err!("Can't parse from U448T to u128"))?;
        Ok(SqrtPrice::new(casted_result))
    }

    pub fn checked_big_div_values_up(
        nominator: U448T,
        denominator: U448T,
    ) -> TrackableResult<SqrtPrice> {
        let result = nominator
            .checked_mul(Self::one().cast())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_add(
                denominator
                    .checked_sub(U448T::from(1u32))
                    .ok_or_else(|| err!(TrackableError::SUB))?,
            )
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        // TODO - add ok_or_mark_trace!
        let casted_result = SqrtPrice::checked_from_value::<u128, U448T>(result)
            .map_err(|_| err!("Can't parse from U448T to u128"))?;
        Ok(SqrtPrice::new(casted_result))
    }
}

pub fn calculate_sqrt_price(tick_index: i32) -> TrackableResult<SqrtPrice> {
    // checking if tick be converted to sqrt_price (overflows if more)
    let tick = tick_index.abs();

    if tick > MAX_TICK {
        return Err(err!("tick over bounds"));
    }

    let mut sqrt_price = FixedPoint::from_integer(1);

    if tick & 0x1 != 0 {
        sqrt_price *= FixedPoint::new(1000049998750u128);
    }
    if tick & 0x2 != 0 {
        sqrt_price *= FixedPoint::new(1000100000000u128);
    }
    if tick & 0x4 != 0 {
        sqrt_price *= FixedPoint::new(1000200010000u128);
    }
    if tick & 0x8 != 0 {
        sqrt_price *= FixedPoint::new(1000400060004u128);
    }
    if tick & 0x10 != 0 {
        sqrt_price *= FixedPoint::new(1000800280056u128);
    }
    if tick & 0x20 != 0 {
        sqrt_price *= FixedPoint::new(1001601200560u128);
    }
    if tick & 0x40 != 0 {
        sqrt_price *= FixedPoint::new(1003204964963u128);
    }
    if tick & 0x80 != 0 {
        sqrt_price *= FixedPoint::new(1006420201726u128);
    }
    if tick & 0x100 != 0 {
        sqrt_price *= FixedPoint::new(1012881622442u128);
    }
    if tick & 0x200 != 0 {
        sqrt_price *= FixedPoint::new(1025929181080u128);
    }
    if tick & 0x400 != 0 {
        sqrt_price *= FixedPoint::new(1052530684591u128);
    }
    if tick & 0x800 != 0 {
        sqrt_price *= FixedPoint::new(1107820842005u128);
    }
    if tick & 0x1000 != 0 {
        sqrt_price *= FixedPoint::new(1227267017980u128);
    }
    if tick & 0x2000 != 0 {
        sqrt_price *= FixedPoint::new(1506184333421u128);
    }
    if tick & 0x4000 != 0 {
        sqrt_price *= FixedPoint::new(2268591246242u128);
    }
    if tick & 0x8000 != 0 {
        sqrt_price *= FixedPoint::new(5146506242525u128);
    }
    if tick & 0x0001_0000 != 0 {
        sqrt_price *= FixedPoint::new(26486526504348u128);
    }
    if tick & 0x0002_0000 != 0 {
        sqrt_price *= FixedPoint::new(701536086265529u128);
    }

    Ok(if tick_index >= 0 {
        SqrtPrice::checked_from_decimal(sqrt_price)
            .map_err(|_| err!("calculate_sqrt_price: parsing from scale failed"))?
    } else {
        SqrtPrice::checked_from_decimal(
            FixedPoint::from_integer(1)
                .checked_div(sqrt_price)
                .map_err(|_| err!("calculate_sqrt_price::checked_div division failed"))?,
        )
        .map_err(|_| err!("calculate_sqrt_price: parsing scale failed"))?
    })
}

#[wasm_wrapper]
pub fn get_max_tick(tick_spacing: u16) -> i32 {
    let tick_spacing = tick_spacing as i32;
    MAX_TICK / tick_spacing * tick_spacing
}

#[wasm_wrapper]
pub fn get_min_tick(tick_spacing: u16) -> i32 {
    let tick_spacing = tick_spacing as i32;
    MIN_TICK / tick_spacing * tick_spacing
}

#[wasm_wrapper]
pub fn get_max_sqrt_price(tick_spacing: u16) -> SqrtPrice {
    let max_tick = get_max_tick(tick_spacing);
    SqrtPrice::from_tick(max_tick).unwrap()
}

#[wasm_wrapper]
pub fn get_min_sqrt_price(tick_spacing: u16) -> SqrtPrice {
    let min_tick = get_min_tick(tick_spacing);
    SqrtPrice::from_tick(min_tick).unwrap()
}
