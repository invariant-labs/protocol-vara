use crate::{convert, decimal_ops};
use core::convert::{TryFrom, TryInto};
use decimal::*;
use js_sys::BigInt;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[decimal(24, U384T)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Price(#[tsify(type = "bigint")] pub U128);

decimal_ops!(Price);
