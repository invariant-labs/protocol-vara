use crate::alloc::string::{String, ToString};
use crate::types::liquidity::Liquidity;
use crate::types::sqrt_price::SqrtPrice;
use crate::types::token_amount::TokenAmount;
use crate::PoolKey;

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Default, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CreatePositionEvent {
    #[tsify(type = "bigint")]
    timestamp: u64,
    address: String,
    pool: PoolKey,
    liquidity: Liquidity,
    #[tsify(type = "bigint")]
    lower_tick: i32,
    #[tsify(type = "bigint")]
    upper_tick: i32,
    current_sqrt_price: SqrtPrice,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CrossTickEvent {
    #[tsify(type = "bigint")]
    timestamp: u64,
    address: String,
    pool: PoolKey,
    #[tsify(type = "bigint[]")]
    indexes: Vec<i32>,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct RemovePositionEvent {
    #[tsify(type = "bigint")]
    timestamp: u64,
    address: String,
    pool: PoolKey,
    liquidity: Liquidity,
    #[tsify(type = "bigint")]
    lower_tick: i32,
    #[tsify(type = "bigint")]
    upper_tick: i32,
    current_sqrt_price: SqrtPrice,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SwapEvent {
    #[tsify(type = "bigint")]
    timestamp: u64,
    address: String,
    pool: PoolKey,
    amount_in: TokenAmount,
    amount_out: TokenAmount,
    fee: TokenAmount,
    start_sqrt_price: SqrtPrice,
    target_sqrt_price: SqrtPrice,
    x_to_y: bool,
}
