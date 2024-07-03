use crate::alloc::string::{ToString, String};
use crate::errors::InvariantError;
use crate::is_token_x;
use crate::FeeTier;
use crate::{convert, resolve};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PoolKey {
    #[tsify(type = "`0x${string}`")]
    pub token_x: String,
    #[tsify(type = "`0x${string}`")]
    pub token_y: String,
    pub fee_tier: FeeTier,
}

impl PoolKey {
    pub fn new(
        token_0: String,
        token_1: String,
        fee_tier: FeeTier,
    ) -> Result<Self, InvariantError> {
        if token_0 == token_1 {
            return Err(InvariantError::TokensAreSame);
        }

        Ok(if is_token_x(token_0.clone(), token_1.clone()).unwrap() {
            PoolKey {
                token_x: token_0,
                token_y: token_1,
                fee_tier,
            }
        } else {
            PoolKey {
                token_x: token_1,
                token_y: token_0,
                fee_tier,
            }
        })
    }
}

#[wasm_bindgen(js_name = "_newPoolKey")]
pub fn new_pool_key(
    token_0: JsValue,
    token_1: JsValue,
    fee_tier: JsValue,
) -> Result<JsValue, JsValue> {
    let token_0: String = convert!(token_0)?;
    let token_1: String = convert!(token_1)?;
    let fee_tier: FeeTier = convert!(fee_tier)?;
    resolve!(PoolKey::new(token_0, token_1, fee_tier))
}
