#[macro_export]
macro_rules! send_request {
    (token: $token: expr, user: $user: expr, service_name: $name: literal, action: $action: literal, payload: ($($val: expr),*)) => {
        {
            use gstd::*;
            let request = [
                $name.encode(),
                $action.encode(),
                ( $( $val, )*).encode(),
            ]
            .concat();

            $token.send_bytes($user, request)
        }

    };
}
