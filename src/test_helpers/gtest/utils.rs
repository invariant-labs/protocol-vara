#[macro_export]
macro_rules! send_query {
  (
    program: $program:expr,
    user: $user:expr,
    service_name: $name:literal,
    action: $action:literal,
    payload: ($($val:expr),*),
    response_type: $response_type:ty
  ) => {
        {
          use sails_rs::prelude::*;
          use crate::{send_request, test_helpers::gtest::*};

          let name = $name.to_string().encode();
          let action = $action.to_string().encode();
          let offset = name.len() + action.len();

          let request = [
              name,
              action,
              ( $( $val, )*).encode(),
          ]
          .concat();

          let result = $program.send_bytes($user, request);
          result.assert_success();
          let response = <$response_type>::decode(
            &mut &result
            .log()
            .last()
            .expect("Result did not emit events")
            .payload()[offset..]
          )
          .expect("Failed to decode response");

          response
        }
  };
}

#[macro_export]
macro_rules! send_request {
    (program: $program: expr, user: $user: expr, service_name: $name: literal, action: $action: literal, payload: ($($val: expr),*)) => {
        {
            use sails_rs::*;
            let request = [
                $name.encode(),
                $action.encode(),
                ( $( $val, )*).encode(),
            ]
            .concat();

            $program.send_bytes($user, request)
        }

    };
    (program: $program: expr, user: $user: expr, service_name: $name: literal, action: $action: literal, payload: ($($val: expr),*), value: $value: expr ) => {
      {
          use sails_rs::*;
          let request = [
              $name.encode(),
              $action.encode(),
              ( $( $val, )*).encode(),
          ]
          .concat();

          $program.send_bytes_with_value($user, request, $value)
      }

  };
}
