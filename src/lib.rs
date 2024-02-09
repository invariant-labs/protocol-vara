#![no_std]

extern crate alloc;
#[cfg(test)]
mod e2e;
mod math;

use gstd::{
    msg::{self, reply},
    prelude::*,
};
use io::*;

#[derive(Default, Clone)]
pub struct Invariant {
    pub config: InvariantConfig,
}

impl Invariant {
    pub fn change_protocol_fee(&mut self, protocol_fee: u128) {
        self.config.protocol_fee = protocol_fee;

        reply(InvariantEvent::ProtocolFeeChanged(protocol_fee), 0).expect("Unable to reply");
    }
}

static mut INVARIANT: Option<Invariant> = None;

#[no_mangle]
extern "C" fn init() {
    let init: InitInvariant = msg::load().expect("Unable to decode InitInvariant");

    let invariant = Invariant {
        config: init.config,
    };

    unsafe {
        INVARIANT = Some(invariant);
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: InvariantAction = msg::load().expect("Unable to decode InvariantAction");
    let invariant = unsafe { INVARIANT.get_or_insert(Default::default()) };

    match action {
        InvariantAction::ChangeProtocolFee(protocol_fee) => {
            invariant.change_protocol_fee(protocol_fee)
        }
    }
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
