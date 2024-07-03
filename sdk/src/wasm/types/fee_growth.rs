use crate::liquidity::*;

use crate::token_amount::TokenAmount;
use crate::{convert, decimal_ops};
use core::convert::{TryFrom, TryInto};
use decimal::*;
use js_sys::BigInt;
use serde::{Deserialize, Serialize};
use traceable_result::*;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[decimal(28, U256)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct FeeGrowth(#[tsify(type = "bigint")] pub U128);

decimal_ops!(FeeGrowth);

impl FeeGrowth {
    pub fn unchecked_add(self, other: FeeGrowth) -> FeeGrowth {
        if other.get() > FeeGrowth::max_instance().get() - self.get() {
            FeeGrowth::new((other.get() - (FeeGrowth::max_instance().get() - self.get())) - 1)
        } else {
            FeeGrowth::new(self.get() + other.get())
        }
    }

    pub fn unchecked_sub(self, other: FeeGrowth) -> FeeGrowth {
        if other.get() > self.get() {
            FeeGrowth::new(FeeGrowth::max_instance().get() - (other.get() - self.get()) + 1)
        } else {
            FeeGrowth::new(self.get() - other.get())
        }
    }

    pub fn from_fee(liquidity: Liquidity, fee: TokenAmount) -> TrackableResult<Self> {
        Ok(Self::new(
            Self::checked_from_value(
                fee.cast::<U384T>()
                    .checked_mul(FeeGrowth::one().cast())
                    .ok_or_else(|| err!(TrackableError::MUL))?
                    .checked_mul(Liquidity::one().cast())
                    .ok_or_else(|| err!(TrackableError::MUL))?
                    .checked_div(liquidity.cast())
                    .ok_or_else(|| err!(TrackableError::DIV))?,
            )
            .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?,
        ))
    }

    pub fn to_fee(self, liquidity: Liquidity) -> TrackableResult<TokenAmount> {
        Ok(TokenAmount::new(
            TokenAmount::checked_from_value(
                self.cast::<U384T>()
                    .checked_mul(liquidity.cast())
                    .ok_or_else(|| err!(TrackableError::MUL))?
                    .checked_div(Liquidity::one().cast())
                    .ok_or_else(|| err!(TrackableError::MUL))?
                    .checked_div(FeeGrowth::one().cast())
                    .ok_or_else(|| err!(TrackableError::MUL))?,
            )
            .map_err(|_| err!(TrackableError::cast::<TokenAmount>().as_str()))?,
        ))
    }
}

pub fn calculate_fee_growth_inside(
    tick_lower: i32,
    tick_lower_fee_growth_outside_x: FeeGrowth,
    tick_lower_fee_growth_outside_y: FeeGrowth,
    tick_upper: i32,
    tick_upper_fee_growth_outside_x: FeeGrowth,
    tick_upper_fee_growth_outside_y: FeeGrowth,
    tick_current: i32,
    fee_growth_global_x: FeeGrowth,
    fee_growth_global_y: FeeGrowth,
) -> (FeeGrowth, FeeGrowth) {
    // determine position relative to current tick
    let current_above_lower = tick_current >= tick_lower;
    let current_below_upper = tick_current < tick_upper;

    // calculate fee growth below
    let fee_growth_below_x = if current_above_lower {
        tick_lower_fee_growth_outside_x
    } else {
        fee_growth_global_x.unchecked_sub(tick_lower_fee_growth_outside_x)
    };
    let fee_growth_below_y = if current_above_lower {
        tick_lower_fee_growth_outside_y
    } else {
        fee_growth_global_y.unchecked_sub(tick_lower_fee_growth_outside_y)
    };

    // calculate fee growth above
    let fee_growth_above_x = if current_below_upper {
        tick_upper_fee_growth_outside_x
    } else {
        fee_growth_global_x.unchecked_sub(tick_upper_fee_growth_outside_x)
    };
    let fee_growth_above_y = if current_below_upper {
        tick_upper_fee_growth_outside_y
    } else {
        fee_growth_global_y.unchecked_sub(tick_upper_fee_growth_outside_y)
    };

    // calculate fee growth inside
    let fee_growth_inside_x = fee_growth_global_x
        .unchecked_sub(fee_growth_below_x)
        .unchecked_sub(fee_growth_above_x);
    let fee_growth_inside_y = fee_growth_global_y
        .unchecked_sub(fee_growth_below_y)
        .unchecked_sub(fee_growth_above_y);

    (fee_growth_inside_x, fee_growth_inside_y)
}
