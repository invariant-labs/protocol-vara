#![no_std]
include!(concat!(env!("OUT_DIR"), "/extended_vft_client.rs"));
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(target_arch = "wasm32")]
pub use extended_vft_app::wasm::*;
