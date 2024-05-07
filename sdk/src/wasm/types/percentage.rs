#[allow(unused_imports)]
use crate::{decimal_ops, convert};
use core::convert::{TryFrom, TryInto};
use decimal::*;
use js_sys::BigInt;
use serde::{Deserialize, Serialize};

use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[decimal(12)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Percentage(#[tsify(type = "bigint")] pub u64);

decimal_ops!(Percentage);