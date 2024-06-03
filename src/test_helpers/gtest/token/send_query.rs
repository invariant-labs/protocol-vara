#[macro_export]
macro_rules! send_query {
  (
    token: $token:expr,
    user: $user:expr,
    service_name: $name:literal,
    action: $action:literal,
    payload: ($($val:expr),*),
    response_type: $response_type:ident
  ) => {
        {
          use gstd::*;
          use crate::test_helpers::gtest::entrypoints::utils::InvariantResult;

          let name = $name.to_string().encode();
          let action = $action.to_string().encode();
          let offset = name.len() + action.len();

          let request = [
              name,
              action,
              ( $( $val, )*).encode(),
          ]
          .concat();

          let result = $token.send_bytes($user, request);
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
