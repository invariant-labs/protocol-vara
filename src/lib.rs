#![no_std]

use gstd::{msg, prelude::*};

#[derive(Copy, Clone)]
pub struct Liquidity {
    pub v: u128
}

static mut GLOBAL_LIQUIDITY: Liquidity = Liquidity { v: 0 };

#[no_mangle]
extern "C" fn handle() {
    let command = msg::load_bytes().expect("Invalid message");

    let liquidity = unsafe { &mut GLOBAL_LIQUIDITY };

    match command.as_slice() {
        b"inc" => liquidity.v += 1,
        b"dec" => liquidity.v -= 1,
        b"get" => {
            let liquidity_value = liquidity.v;
            msg::reply_bytes(format!("{liquidity_value}"), 0).expect("Unable to reply");
        }
        _ => (),
    }

    unsafe { GLOBAL_LIQUIDITY = *liquidity };
}