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

#[decimal(24)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SqrtPrice(#[tsify(type = "bigint")] pub u128);

decimal_ops!(SqrtPrice);

impl SqrtPrice {
    pub fn from_tick(i: i32) -> TrackableResult<Self> {
        calculate_sqrt_price(i)
    }

    pub fn big_div_values_to_token(
        nominator: U256,
        denominator: U256,
    ) -> TrackableResult<TokenAmount> {
        let nominator = u256_to_u320(nominator);
        let denominator = u256_to_u320(denominator);

        let intermediate_u320 = nominator
            .checked_mul(Self::one::<U320>())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let result = checked_u320_to_u256(intermediate_u320)
            .ok_or_else(|| err!("Can't parse from u320 to u256"))?
            .checked_div(Self::one::<U256>())
            .ok_or_else(|| err!(TrackableError::DIV))?
            .try_into()
            .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?;
        Ok(TokenAmount(result))
    }

    pub fn big_div_values_to_token_up(
        nominator: U256,
        denominator: U256,
    ) -> TrackableResult<TokenAmount> {
        let nominator = u256_to_u320(nominator);
        let denominator = u256_to_u320(denominator);

        let intermediate_u320 = nominator
            .checked_mul(Self::one::<U320>())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_add(denominator - 1)
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let result = checked_u320_to_u256(intermediate_u320)
            .ok_or_else(|| err!("Can't parse from u320 to u256"))?
            .checked_add(Self::almost_one::<U256>())
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(Self::one::<U256>())
            .ok_or_else(|| err!(TrackableError::DIV))?
            .try_into()
            .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?;
        Ok(TokenAmount::new(result))
    }

    pub fn big_div_values_up(nominator: U256, denominator: U256) -> SqrtPrice {
        SqrtPrice::new({
            nominator
                .checked_mul(Self::one::<U256>())
                .unwrap()
                .checked_add(denominator.checked_sub(U256::from(1u32)).unwrap())
                .unwrap()
                .checked_div(denominator)
                .unwrap()
                .try_into()
                .unwrap()
        })
    }

    pub fn checked_big_div_values(
        nominator: U256,
        denominator: U256,
    ) -> TrackableResult<SqrtPrice> {
        Ok(SqrtPrice::new(
            nominator
                .checked_mul(Self::one::<U256>())
                .ok_or_else(|| err!(TrackableError::MUL))?
                .checked_div(denominator)
                .ok_or_else(|| err!(TrackableError::DIV))?
                .try_into()
                .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?,
        ))
    }

    pub fn checked_big_div_values_up(
        nominator: U256,
        denominator: U256,
    ) -> TrackableResult<SqrtPrice> {
        let denominator = u256_to_u320(denominator);

        Ok(SqrtPrice::new(
            u256_to_u320(nominator)
                .checked_mul(Self::one::<U320>())
                .ok_or_else(|| err!(TrackableError::MUL))?
                .checked_add(
                    denominator
                        .checked_sub(U320::from(1u32))
                        .ok_or_else(|| err!(TrackableError::SUB))?,
                )
                .ok_or_else(|| err!(TrackableError::ADD))?
                .checked_div(denominator)
                .ok_or_else(|| err!(TrackableError::DIV))?
                .try_into()
                .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?,
        ))
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
        sqrt_price *= FixedPoint::new(1000049998750);
    }
    if tick & 0x2 != 0 {
        sqrt_price *= FixedPoint::new(1000100000000);
    }
    if tick & 0x4 != 0 {
        sqrt_price *= FixedPoint::new(1000200010000);
    }
    if tick & 0x8 != 0 {
        sqrt_price *= FixedPoint::new(1000400060004);
    }
    if tick & 0x10 != 0 {
        sqrt_price *= FixedPoint::new(1000800280056);
    }
    if tick & 0x20 != 0 {
        sqrt_price *= FixedPoint::new(1001601200560);
    }
    if tick & 0x40 != 0 {
        sqrt_price *= FixedPoint::new(1003204964963);
    }
    if tick & 0x80 != 0 {
        sqrt_price *= FixedPoint::new(1006420201726);
    }
    if tick & 0x100 != 0 {
        sqrt_price *= FixedPoint::new(1012881622442);
    }
    if tick & 0x200 != 0 {
        sqrt_price *= FixedPoint::new(1025929181080);
    }
    if tick & 0x400 != 0 {
        sqrt_price *= FixedPoint::new(1052530684591);
    }
    if tick & 0x800 != 0 {
        sqrt_price *= FixedPoint::new(1107820842005);
    }
    if tick & 0x1000 != 0 {
        sqrt_price *= FixedPoint::new(1227267017980);
    }
    if tick & 0x2000 != 0 {
        sqrt_price *= FixedPoint::new(1506184333421);
    }
    if tick & 0x4000 != 0 {
        sqrt_price *= FixedPoint::new(2268591246242);
    }
    if tick & 0x8000 != 0 {
        sqrt_price *= FixedPoint::new(5146506242525);
    }
    if tick & 0x0001_0000 != 0 {
        sqrt_price *= FixedPoint::new(26486526504348);
    }
    if tick & 0x0002_0000 != 0 {
        sqrt_price *= FixedPoint::new(701536086265529);
    }

    // Parsing to the Sqrt_price type by the end by convention (should always have 12 zeros at the end)
    Ok(if tick_index >= 0 {
        SqrtPrice::checked_from_decimal(sqrt_price)
            .map_err(|_| err!("calculate_sqrt_price: parsing from scale failed"))?
    } else {
        SqrtPrice::checked_from_decimal(
            FixedPoint::from_integer(1)
                .checked_div(sqrt_price)
                .map_err(|_| err!("calcaule_sqrt_price::checked_div division failed"))?,
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
