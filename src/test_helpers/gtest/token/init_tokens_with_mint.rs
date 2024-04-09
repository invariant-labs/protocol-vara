use gtest::*;
use crate::test_helpers::gtest::consts::*;
use super::init_tokens;
use fungible_token_io::*;

pub fn init_tokens_with_mint(sys: &System, initial_balances: (u128, u128))-> (Program<'_>, Program<'_>){
  
  let (token_x, token_y) = init_tokens(&sys);
  assert!(!token_x.send(REGULAR_USER_1, FTAction::Mint(initial_balances.0)).main_failed());
  assert!(!token_x.send(REGULAR_USER_2, FTAction::Mint(initial_balances.0)).main_failed());
  assert!(!token_y.send(REGULAR_USER_1, FTAction::Mint(initial_balances.1)).main_failed());
  assert!(!token_y.send(REGULAR_USER_2, FTAction::Mint(initial_balances.1)).main_failed());

  (token_x, token_y)
}
