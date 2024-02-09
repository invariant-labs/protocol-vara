#![no_std]

extern crate alloc;
#[cfg(test)]
mod e2e;
mod math;

use decimal::Decimal;
use gstd::{msg, prelude::*};
use math::percentage::Percentage;
use math::sqrt_price::SqrtPrice;

static mut GLOBAL_SQRT_PRICE: SqrtPrice = SqrtPrice(0);
static mut PROTOCOL_FEE: Percentage = Percentage(0);

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

#[no_mangle]
extern "C" fn init() {
    let payload = msg::load().expect("Invalid message");
    unsafe { PROTOCOL_FEE = payload };
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtest::{Log, Program, System};

    #[test]
    fn test_init_and_call() {
        let sys = System::new();
        let program_id = 105;
        let program = Program::from_file_with_id(
            &sys,
            program_id,
            "./target/wasm32-unknown-unknown/release/invariant.wasm",
        );

        sys.init_logger();
        let _ = Log::builder();

        let _ = program.send_bytes(100001, "INIT MESSAGE");
        let _ = program.send_bytes(100001, b"inc");
        let res = program.send_bytes(100001, b"get");

        let expected_response = Log::builder()
            .source(program_id)
            .dest(100001)
            .payload_bytes(b"1");

        assert!(res.contains(&expected_response));
    }
}
