use crate::test_helpers::gtest::*;
use contracts::FeeTier;
use contracts::{InvariantError, PoolKey};
use decimal::*;
use gtest::System;
use math::types::percentage::Percentage;
use sails_rs::ActorId;

#[test]
fn test_get_position_with_associates() {
    let sys = System::new();
    sys.init_logger();
    let protocol_fee = Percentage::new(0);
    let invariant = init_invariant(&sys, protocol_fee);

    let init_mint = U256::from(100000000);
    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (init_mint, init_mint));

    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 10,
    };

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&invariant, &token_x_program, &token_y_program);

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let (lower_tick_index, upper_tick_index) = (-20, 10);

    let position_regular = get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap();

    let pool_regular = get_pool(&invariant, token_y, token_x, fee_tier).unwrap();
    let lower_tick_regular = get_tick(&invariant, pool_key, lower_tick_index).unwrap();
    let upper_tick_regular = get_tick(&invariant, pool_key, upper_tick_index).unwrap();

    let (position, pool, lower_tick, upper_tick) =
        get_position_with_associates(&invariant, REGULAR_USER_1, 0).unwrap();

    assert_eq!(position_regular, position);
    assert_eq!(pool_regular, pool);
    assert_eq!(lower_tick_regular, lower_tick);
    assert_eq!(upper_tick_regular, upper_tick);
}

#[test]
fn test_position_does_not_exist() {
    let sys = System::new();
    sys.init_logger();
    let protocol_fee = Percentage::new(0);
    let invariant = init_invariant(&sys, protocol_fee);

    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    init_basic_pool(&invariant, &token_x, &token_y);
    let result = get_position_with_associates(&invariant, REGULAR_USER_1, 0);

    assert_eq!(result, Err(InvariantError::PositionNotFound));
}
