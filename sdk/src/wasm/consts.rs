use crate::types::sqrt_price::get_max_tick;
use js_sys::BigInt;
use traceable_result::*;
use wasm_bindgen::prelude::*;
use wasm_wrapper::wasm_wrapper;

pub const LIQUIDITY_TICK_LIMIT: u32 = 21544;
pub const POSITION_TICK_LIMIT: u32 = 17872;
pub const MAX_POOL_PAIRS_RETURNED: u32 = 4012;
pub const MAX_POOL_KEYS_RETURNED: u16 = 9590;
pub const POSITION_ENTRIES_LIMIT: u32 = 1800;

pub const MAX_TICK: i32 = 221_818;
pub const MIN_TICK: i32 = -MAX_TICK;

pub const MAX_SQRT_PRICE: u128 = 65535383934512647000000000000;
pub const MIN_SQRT_PRICE: u128 = 15258932000000000000;

pub const TICK_SEARCH_RANGE: i32 = 256;
pub const CHUNK_SIZE: i32 = 64;

pub const MAX_SWAP_STEPS: u32 = 1117 * 8 / 10 * 120 / 750; // 1117*8/10 - max gas limit, 120/750 gas limit/max gas coefficient;

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

#[wasm_wrapper("_getMaxChunk")]
pub fn get_max_chunk(tick_spacing: u16) -> TrackableResult<u16> {
    let max_tick = get_max_tick(tick_spacing)?;
    let max_bitmap_index = (max_tick + MAX_TICK) / tick_spacing as i32;
    let max_chunk_index = max_bitmap_index / CHUNK_SIZE;
    Ok(max_chunk_index as u16)
}

#[wasm_wrapper]
pub fn get_chunk_size() -> i32 {
    CHUNK_SIZE
}

#[wasm_wrapper]
pub fn get_max_swap_step() -> u32 {
    MAX_SWAP_STEPS
}

#[wasm_wrapper]
pub fn get_position_tick_limit() -> u32 {
    POSITION_TICK_LIMIT
}

#[wasm_wrapper]
pub fn get_max_pool_keys_returned() -> u16 {
    MAX_POOL_KEYS_RETURNED
}

#[wasm_wrapper]
pub fn get_position_entries_limit() -> u32 {
    POSITION_ENTRIES_LIMIT
}

#[wasm_wrapper]
pub fn get_max_pool_pairs_returned() -> u32 {
    MAX_POOL_PAIRS_RETURNED
}

#[wasm_wrapper]
pub fn get_liquidity_tick_limit() -> u32 {
    LIQUIDITY_TICK_LIMIT
}
