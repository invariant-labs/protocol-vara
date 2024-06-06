use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use gtest::*;
use math::percentage::Percentage;

use io::*;

pub fn init_invariant(sys: &System, protocol_fee: Percentage) -> Program<'_> {
    let bytes = include_bytes!("../../../../target/wasm32-unknown-unknown/release/invariant.opt.wasm");
    let program = Program::from_binary_with_id(sys, INVARIANT_ID, bytes);

    assert!(!program
        .send(
            PROGRAM_OWNER,
            InitInvariant {
                config: InvariantConfig {
                    admin: ADMIN.into(),
                    protocol_fee,
                },
            },
        )
        .main_failed());
    program
}
