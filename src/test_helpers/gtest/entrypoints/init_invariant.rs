use crate::send_request;
use crate::test_helpers::gtest::*;
use gtest::*;
use io::*;
use math::percentage::Percentage;
use sails_rs::prelude::*;

use super::InvariantResult;

pub fn init_invariant(sys: &System, protocol_fee: Percentage) -> Program<'_> {
    let bytes =
        include_bytes!("../../../../target/wasm32-unknown-unknown/release/invariant_wasm.opt.wasm");
    let program = Program::from_binary_with_id(sys, INVARIANT_ID, bytes);

    let init = InvariantConfig {
        admin: ADMIN.into(),
        protocol_fee,
    };

    let request: Vec<u8> = ["New".encode(), init.encode()].concat();
    program
        .send_bytes(PROGRAM_OWNER, request.clone())
        .assert_success();

    program
}
