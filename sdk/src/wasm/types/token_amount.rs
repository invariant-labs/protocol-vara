use crate::alloc::string::ToString;
use crate::{convert, decimal_ops_uint};
use core::convert::{TryFrom, TryInto};
use decimal::*;
use js_sys::BigInt;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[decimal(0, U512)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TokenAmount(#[tsify(type = "bigint")] pub U256);

decimal_ops_uint!(TokenAmount);
