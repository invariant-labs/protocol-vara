use crate::test_helpers::gtest::*;
use contracts::{FeeTier, PoolKey, PositionTick, POSITION_TICK_LIMIT};
use decimal::*;
use gtest::*;
use math::types::{
    liquidity::Liquidity,
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
};
use sails_rs::prelude::*;

#[test]
fn test_get_position_ticks() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let initial_amount = 10u128.pow(10).into();
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount, initial_amount));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(1, 2), 1).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_success();

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, 500.into()).assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, 500.into()).assert_success();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        500.into(),
        token_y,
        500.into(),
        None::<&str>,
    );

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -10,
        10,
        Liquidity::new(10.into()),
        SqrtPrice::new(0),
        SqrtPrice::max_instance(),
    )
    .assert_success();

    let result = get_position_ticks(&invariant, REGULAR_USER_1, 0);
    assert_eq!(result.len(), 2);

    let lower_tick = get_tick(&invariant, pool_key, -10).unwrap();
    let upper_tick = get_tick(&invariant, pool_key, 10).unwrap();

    assert_eq!(result[0], PositionTick::from(&lower_tick));
    assert_eq!(result[1], PositionTick::from(&upper_tick));
}

#[test]
fn test_get_position_ticks_limit() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let initial_amount = 10u128.pow(10).into();
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount, initial_amount));

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);
    let fee_tier = FeeTier::new(Percentage::from_scale(1, 2), 1).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_success();

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_amount,
        token_y,
        initial_amount,
        None::<&str>,
    );

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    for i in 1..=POSITION_TICK_LIMIT / 2 {
        create_position(
            &invariant,
            REGULAR_USER_1,
            pool_key,
            -(i as i32),
            i as i32,
            Liquidity::new(10.into()),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
        )
        .assert_success();
    }

    let result = get_position_ticks(&invariant, REGULAR_USER_1, 0);
    assert_eq!(result.len(), POSITION_TICK_LIMIT);

    for i in 1..=POSITION_TICK_LIMIT / 2 {
        let lower_tick = get_tick(&invariant, pool_key, -(i as i32)).unwrap();
        let upper_tick = get_tick(&invariant, pool_key, i as i32).unwrap();

        assert_eq!(result[i * 2 - 2], PositionTick::from(&lower_tick));
        assert_eq!(result[i * 2 - 1], PositionTick::from(&upper_tick));
    }
}

#[test]
fn test_get_position_ticks_with_offset() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let initial_amount = 10u128.pow(10).into();
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount, initial_amount));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier_1 = FeeTier::new(Percentage::from_scale(1, 2), 2).unwrap();
    let fee_tier_2 = FeeTier::new(Percentage::from_scale(1, 2), 10).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier_1).assert_success();
    add_fee_tier(&invariant, ADMIN, fee_tier_2).assert_success();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier_1,
        init_sqrt_price,
        init_tick,
    )
    .assert_success();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier_2,
        init_sqrt_price,
        init_tick,
    )
    .assert_success();

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_amount,
        token_y,
        initial_amount,
        None::<&str>,
    );

    let pool_key_1 = PoolKey::new(token_x, token_y, fee_tier_1).unwrap();
    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key_1,
        -10,
        30,
        Liquidity::new(10.into()),
        SqrtPrice::new(0),
        SqrtPrice::max_instance(),
    )
    .assert_success();

    let pool_key_2 = PoolKey::new(token_x, token_y, fee_tier_2).unwrap();
    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key_2,
        -20,
        40,
        Liquidity::new(10.into()),
        SqrtPrice::new(0),
        SqrtPrice::max_instance(),
    )
    .assert_success();

    let result_1 = get_position_ticks(&invariant, REGULAR_USER_1, 0);
    assert_eq!(result_1.len(), 4);

    let result_2 = get_position_ticks(&invariant, REGULAR_USER_1, 1);
    assert_eq!(result_2.len(), 2);

    assert_eq!(result_1[2], result_2[0]);
    assert_eq!(result_1[3], result_2[1]);
}
