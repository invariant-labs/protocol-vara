use crate::alloc::string::ToString;
use crate::errors::InvariantError;
use crate::types::percentage::Percentage;
use crate::{convert, resolve};
use decimal::*;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FeeTier {
    pub fee: Percentage,
    #[tsify(type = "bigint")]
    pub tick_spacing: u16,
}

impl FeeTier {
    pub fn new(fee: Percentage, tick_spacing: u16) -> Result<Self, InvariantError> {
        if tick_spacing == 0 || tick_spacing > 100 {
            return Err(InvariantError::InvalidTickSpacing);
        }

        if fee > Percentage::from_integer(1) {
            return Err(InvariantError::InvalidFee);
        }

        Ok(Self { fee, tick_spacing })
    }
}

#[wasm_bindgen(js_name = "_newFeeTier")]
pub fn new_fee_tier(js_fee: JsValue, js_tick_spacing: JsValue) -> Result<JsValue, JsValue> {
    let fee: Percentage = convert!(js_fee)?;
    let tick_spacing: u16 = convert!(js_tick_spacing)?;
    resolve!(FeeTier::new(fee, tick_spacing))
}
