use gtest::*;
use crate::test_helpers::gtest::*;
use crate::test_helpers::consts::*;
use fungible_token_io::*;

pub fn init_tokens(sys: &System) -> (Program<'_>, Program<'_>) {
  let token_x = Program::from_file_with_id(sys, TOKEN_X_ID, TOKEN_PATH);
  let token_y = Program::from_file_with_id(sys, TOKEN_Y_ID, TOKEN_PATH);
  assert_ne!(token_x.id(), token_y.id());

  assert!(!token_x
      .send(
          PROGRAM_OWNER,
          InitConfig::default()
      )
      .main_failed());

  assert!(!token_y
      .send(
          PROGRAM_OWNER,
          InitConfig::default()
      )
      .main_failed());
  (token_x, token_y)
}