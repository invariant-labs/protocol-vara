use crate::send_request;
use crate::test_helpers::gtest::*;
use gtest::*;
use sails_rs::prelude::*;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Role {
    Admin,
    Burner,
    Minter,
}

pub fn init_tokens(sys: &System) -> (Program<'_>, Program<'_>) {
    let bytes = include_bytes!(
        "../../../../target/wasm32-unknown-unknown/release/extended_vft_wasm.opt.wasm"
    );
    let token_x = Program::from_binary_with_id(sys, TOKEN_X_ID, bytes);
    let token_y = Program::from_binary_with_id(sys, TOKEN_Y_ID, bytes);
    assert_ne!(token_x.id(), token_y.id());

    let init = ("TokenName".to_owned(), "TokenSymbol".to_owned(), 10_u8);
    let request: Vec<u8> = ["New".encode(), init.encode()].concat();
    token_x
        .send_bytes(PROGRAM_OWNER, request.clone())
        .assert_success();

    token_y
        .send_bytes(PROGRAM_OWNER, request.clone())
        .assert_success();

    (token_x, token_y)
}
