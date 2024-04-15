use crate::test_helpers::gtest::*;
use decimal::*;
use gstd::{prelude::*, ActorId};
use gtest::*;
use math::percentage::Percentage;

#[test]
fn test_swap() {
    let sys = System::new();
    sys.init_logger();
    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (token_x_program, token_y_program) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);
    init_basic_swap(&sys, &invariant, &token_x_program, &token_y_program);
}
