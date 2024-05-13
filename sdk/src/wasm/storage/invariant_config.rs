use crate::alloc::string::ToString;
use crate::types::percentage::Percentage;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;
#[derive(Debug, Clone, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct InvariantConfig {
    #[tsify(type = "string")]
    pub admin: String,
    pub protocol_fee: Percentage,
}
