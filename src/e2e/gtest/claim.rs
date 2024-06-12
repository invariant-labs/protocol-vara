use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gstd::*;
use gtest::*;
use io::*;
use math::{percentage::Percentage, token_amount::TokenAmount};

#[test]
fn test_claim() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let (token_x_program, token_y_program) = init_tokens(&sys);

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);
    init_basic_swap(&sys, &invariant, &token_x_program, &token_y_program);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    let user_amount_before_claim = balance_of(&token_x_program, REGULAR_USER_1);
    let invariant_amount_before_claim = balance_of(&token_x_program, INVARIANT_ID);

    assert_eq!(get_user_balances(&invariant, REGULAR_USER_1), vec![]);

    invariant
        .send(REGULAR_USER_1, InvariantAction::ClaimFee { position_id: 0 })
        .assert_success();

    let expected_tokens_claimed = 5;
    
    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_1),
        vec![(ActorId::from(TOKEN_X_ID), TokenAmount(expected_tokens_claimed))]
    );
    assert_eq!(
        withdraw_single_token(&invariant, REGULAR_USER_1, TOKEN_X_ID, None, None::<&str>),
        Some(TokenAmount(expected_tokens_claimed))
    );

    let user_amount_after_claim = balance_of(&token_x_program, REGULAR_USER_1);
    let invariant_amount_after_claim = balance_of(&token_x_program, INVARIANT_ID);

    let position = get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap();

    assert_eq!(
        user_amount_after_claim - expected_tokens_claimed,
        user_amount_before_claim
    );
    assert_eq!(
        invariant_amount_after_claim + expected_tokens_claimed,
        invariant_amount_before_claim
    );
    assert_eq!(position.fee_growth_inside_x, pool.fee_growth_global_x);
    assert_eq!(position.tokens_owed_x, TokenAmount(0));
}

#[test]
fn test_claim_not_owner() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let mut invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let (token_x_program, token_y_program) = init_tokens(&sys);

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);
    init_basic_swap(&sys, &invariant, &token_x_program, &token_y_program);

    let _res = invariant.send_and_assert_panic(
        REGULAR_USER_2,
        InvariantAction::ClaimFee { position_id: 0 },
        InvariantError::PositionNotFound,
    );
}
