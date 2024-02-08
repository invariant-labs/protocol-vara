#![no_std]

extern crate alloc;
mod math;

use decimal::Decimal;
use gstd::{msg, prelude::*};
use math::sqrt_price::SqrtPrice;

static mut GLOBAL_SQRT_PRICE: SqrtPrice = SqrtPrice(0);

#[no_mangle]
extern "C" fn handle() {
    let command = msg::load_bytes().expect("Invalid message");
    
    let mut sqrt_price = unsafe { &mut GLOBAL_SQRT_PRICE };
    
    match command.as_slice() {
        b"inc" => *sqrt_price += SqrtPrice(1),
        b"dec" => *sqrt_price -= SqrtPrice(1),
        b"get" => {
            let sqrt_price_value = sqrt_price.get();
            msg::reply_bytes(format!("{sqrt_price_value}"), 0).expect("Unable to reply");
        }
        _ => (),
    }

    unsafe { GLOBAL_SQRT_PRICE = *sqrt_price };
}