#![no_std]

extern crate alloc;
mod math;

use gstd::{msg, prelude::*};
use math::sqrt_price::SqrtPrice;

#[derive(Copy, Clone)]
pub struct Liquidity {
    pub v: u128
}

static mut GLOBAL_LIQUIDITY: Liquidity = Liquidity { v: 0 };
static mut GLOBAL_SQRT_PRICE: SqrtPrice = SqrtPrice { v: 0 };

#[no_mangle]
extern "C" fn handle() {
    let command = msg::load_bytes().expect("Invalid message");

    let liquidity = unsafe { &mut GLOBAL_LIQUIDITY };
    let sqrt_price = unsafe { &mut GLOBAL_SQRT_PRICE };
    
    match command.as_slice() {
        b"inc" => liquidity.v += 1,
        b"dec" => liquidity.v -= 1,
        b"get" => {
            let liquidity_value = liquidity.v;
            msg::reply_bytes(format!("{liquidity_value}"), 0).expect("Unable to reply");
        }
        _ => (),
    }
    sqrt_price.v = 0;

    unsafe { GLOBAL_LIQUIDITY = *liquidity };
    unsafe { GLOBAL_SQRT_PRICE = *sqrt_price };
}