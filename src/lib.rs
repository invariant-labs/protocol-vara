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
    use gstd::ActorId;
    use gtest::{Log, Program, System};

    pub const USER: [u8; 32] = [0; 32];

    #[test]
    fn test_init() {
        let sys = System::new();
        sys.init_logger();

        let program_id = 105;
        let program = Program::from_file_with_id(
            &sys,
            program_id,
            "./target/wasm32-unknown-unknown/release/invariant.wasm",
        );

        assert!(!program
            .send(
                100001,
                InitInvariant {
                    config: InvariantConfig {
                        admin: ActorId::new(USER),
                        protocol_fee: 100,
                    },
                },
            )
            .main_failed());
    }
}
