use gtest::*;
use gstd::Encode;

pub const USERS: &[u64] = &[3, 4, 5];

#[macro_export]
macro_rules! send_request {
    (ft: $ft: expr, user: $user: expr, service_name: $name: literal, action: $action: literal, payload: ($($val: expr),*)) => {
        {
            let request = [
                $name.encode(),
                $action.to_string().encode(),
                ( $( $val, )*).encode(),
            ]
            .concat();

            $ft.send_bytes($user, request)

        }

    };
}

pub fn init(sys: &System) -> Program {
    let bytes = include_bytes!("../../../../target/wasm32-unknown-unknown/release/gear_erc20_wasm.opt.wasm"); 
    let ft = Program::from_binary_with_id(
        sys,
        10,
        bytes
    );

    let init = ("TokenName".to_owned(), "TokenSymbol".to_owned(), 10_u8);
    let request = ["New".encode(), init.encode()].concat();

    let res = ft.send_bytes(USERS[0], request.clone());
    assert!(!res.main_failed());

    ft
}
