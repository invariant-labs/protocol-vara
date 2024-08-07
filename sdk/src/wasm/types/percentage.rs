use crate::{convert, decimal_ops};
use core::convert::{TryFrom, TryInto};
use decimal::*;
use js_sys::BigInt;
use serde::{Deserialize, Serialize};

use tsify::*;
use wasm_bindgen::prelude::*;

#[decimal(12, U256)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Percentage(#[tsify(type = "bigint")] pub u128);

decimal_ops!(Percentage);
