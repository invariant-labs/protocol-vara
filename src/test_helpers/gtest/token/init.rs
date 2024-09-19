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

pub fn init_token(sys: &System, id: u64) -> Program<'_> {
    let bytes = include_bytes!(
        "../../../../target/wasm32-unknown-unknown/release/gear_erc20_wasm.opt.wasm"
    );
    let token = Program::from_binary_with_id(sys, id, bytes);

    let init = ("TokenName".to_owned(), "TokenSymbol".to_owned(), 10_u8);
    let request: Vec<u8> = ["New".encode(), init.encode()].concat();
    token
        .send_bytes(PROGRAM_OWNER, request.clone())
        .assert_success();

    send_request!(program: token, user: PROGRAM_OWNER, service_name: "Admin", action: "GrantRole", payload: (ActorId::from(PROGRAM_OWNER), Role::Minter)).assert_success();
    send_request!(program: token, user: PROGRAM_OWNER, service_name: "Admin", action: "GrantRole", payload: (ActorId::from(PROGRAM_OWNER), Role::Burner)).assert_success();

    token
}
