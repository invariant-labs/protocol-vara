use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{percentage::Percentage, token_amount::TokenAmount};
use sails_rs::ActorId;

#[test]
fn test_protocol_fee() {
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
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    withdraw_protocol_fee(&invariant, ADMIN, pool_key).assert_success();

    withdraw_token_pair(
        &invariant,
        ADMIN,
        token_x,
        None,
        token_y,
        None,
        None::<&str>,
    )
    .unwrap();

    let amount_x = balance_of(&token_x_program, ADMIN);
    let amount_y = balance_of(&token_y_program, ADMIN);
    assert_eq!(amount_x, U256::from(1));
    assert_eq!(amount_y, U256::from(0));

    let amount_x = balance_of(&token_x_program, INVARIANT_ID);
    let amount_y = balance_of(&token_y_program, INVARIANT_ID);
    assert_eq!(amount_x, U256::from(1499));
    assert_eq!(amount_y, U256::from(7));

    let pool_after_withdraw = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    assert_eq!(
        pool_after_withdraw.fee_protocol_token_x,
        TokenAmount::new(U256::from(0))
    );
    assert_eq!(
        pool_after_withdraw.fee_protocol_token_y,
        TokenAmount::new(U256::from(0))
    );
}

#[test]
fn test_protocol_fee_not_admin() {
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
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    withdraw_protocol_fee(&invariant, REGULAR_USER_1, pool_key)
        .assert_panicked_with(InvariantError::NotFeeReceiver);
}

#[test]
fn test_withdraw_fee_not_deployer() {
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
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    change_fee_receiver(&invariant, ADMIN, pool_key, REGULAR_USER_2.into()).assert_success();

    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    assert_eq!(pool.fee_receiver, REGULAR_USER_2.into());

    withdraw_protocol_fee(&invariant, REGULAR_USER_2, pool_key).assert_success();

    withdraw_single_token(&invariant, REGULAR_USER_2, token_x, None, None::<&str>).unwrap();

    assert_eq!(
        withdraw_single_token(
            &invariant,
            REGULAR_USER_2,
            token_y,
            None,
            Some(InvariantError::NoBalanceForTheToken)
        ),
        None
    );

    let amount_x = balance_of(&token_x_program, REGULAR_USER_2);
    let amount_y = balance_of(&token_y_program, REGULAR_USER_2);
    assert_eq!(amount_x, U256::from(1));
    assert_eq!(amount_y, U256::from(993));

    let amount_x = balance_of(&token_x_program, INVARIANT_ID);
    let amount_y = balance_of(&token_y_program, INVARIANT_ID);
    assert_eq!(amount_x, U256::from(1499));
    assert_eq!(amount_y, U256::from(7));

    let pool_after_withdraw = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    assert_eq!(
        pool_after_withdraw.fee_protocol_token_x,
        TokenAmount::new(U256::from(0))
    );
    assert_eq!(
        pool_after_withdraw.fee_protocol_token_y,
        TokenAmount::new(U256::from(0))
    );
}
