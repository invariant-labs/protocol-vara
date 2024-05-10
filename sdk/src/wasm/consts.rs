use crate::types::sqrt_price::get_max_tick;
use js_sys::BigInt;
use wasm_bindgen::prelude::*;
use wasm_wrapper::wasm_wrapper;

pub use math::{MAX_TICK, MIN_TICK, MAX_SQRT_PRICE, MIN_SQRT_PRICE, TICK_SEARCH_RANGE};

pub const CHUNK_SIZE: i32 = 64;

#[wasm_wrapper]
pub fn get_global_max_sqrt_price() -> u128 {
    MAX_SQRT_PRICE
}

#[wasm_wrapper]
pub fn get_global_min_sqrt_price() -> u128 {
    MIN_SQRT_PRICE
}

#[wasm_wrapper]
pub fn get_tick_search_range() -> i32 {
    TICK_SEARCH_RANGE
}

#[wasm_wrapper]
pub fn get_max_chunk(tick_spacing: u16) -> u16 {
    let max_tick = get_max_tick(tick_spacing);
    let max_bitmap_index = (max_tick + MAX_TICK) / tick_spacing as i32;
    let max_chunk_index = max_bitmap_index / CHUNK_SIZE;
    max_chunk_index as u16
}

#[wasm_wrapper]
pub fn get_chunk_size() -> i32 {
    CHUNK_SIZE
}
