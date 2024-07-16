use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::System;
use math::{
    liquidity::Liquidity,
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
};
use sails_rtl::prelude::*;
use sails_rtl::From;

#[test]
fn test_get_liquidity_ticks() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let initial_amount = 10u128.pow(10);
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount.into(), initial_amount.into()));

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

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, initial_amount.into())
        .assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, initial_amount.into())
        .assert_success();

    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_amount.into(),
        None::<&str>,
    );
    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        token_y,
        initial_amount.into(),
        None::<&str>,
    );

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
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

    let tickmap = get_tickmap(&invariant, pool_key);
    assert_eq!(tickmap.len(), 2);
    let mut ticks = vec![];
    tickmap.iter().for_each(|(chunk_index, chunk)| {
        for i in 0..64 {
            if chunk & (1 << i) != 0 {
                ticks.push(position_to_tick(
                    *chunk_index,
                    i,
                    pool_key.fee_tier.tick_spacing,
                ));
            }
        }
    });
    assert_eq!(ticks, vec![-10i32, 10]);

    let result = get_liquidity_ticks(&invariant, pool_key, ticks.clone()).unwrap();
    assert_eq!(result.len(), 2);

    let lower_tick = get_tick(&invariant, pool_key, -10).unwrap();
    let upper_tick = get_tick(&invariant, pool_key, 10).unwrap();

    assert_eq!(LiquidityTick::from(&lower_tick), result[0]);
    assert_eq!(LiquidityTick::from(&upper_tick), result[1]);
}

#[test]
fn test_get_liquidity_ticks_different_tick_spacings() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let initial_amount = 10u128.pow(10);
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount.into(), initial_amount.into()));

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);
    let fee_tier = FeeTier::new(Percentage::from_scale(1, 2), 1).unwrap();

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

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, initial_amount.into())
        .assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, initial_amount.into())
        .assert_success();

    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_amount.into(),
        None::<&str>,
    );
    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        token_y,
        initial_amount.into(),
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

    let result = get_liquidity_ticks(&invariant, pool_key_1, vec![-10, 30]).unwrap();
    assert_eq!(result.len(), 2);

    let result = get_liquidity_ticks(&invariant, pool_key_2, vec![-20, 40]).unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn test_get_liquidity_ticks_limit() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let initial_amount = 10u128.pow(10);
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount.into(), initial_amount.into()));

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

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, initial_amount.into())
        .assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, initial_amount.into())
        .assert_success();

    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_amount.into(),
        None::<&str>,
    );
    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        token_y,
        initial_amount.into(),
        None::<&str>,
    );

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    let mut ticks = vec![];
    for i in 1..=LIQUIDITY_TICK_LIMIT / 2 {
        ticks.push(i as i32);
        ticks.push(-(i as i32));

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

    let result = get_liquidity_ticks(&invariant, pool_key, ticks.clone()).unwrap();
    assert_eq!(result.len(), LIQUIDITY_TICK_LIMIT);
}

#[test]
fn test_get_liquidity_ticks_limit_with_spread() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let initial_amount = 10u128.pow(10);
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount.into(), initial_amount.into()));

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

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, initial_amount.into())
        .assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, initial_amount.into())
        .assert_success();

    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_amount.into(),
        None::<&str>,
    );
    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        token_y,
        initial_amount.into(),
        None::<&str>,
    );

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let spread = 10;
    let mut ticks = vec![];
    for i in 1..=LIQUIDITY_TICK_LIMIT / 2 {
        let index = (i * spread) as i32;
        ticks.push(index);
        ticks.push(-index);

        create_position(
            &invariant,
            REGULAR_USER_1,
            pool_key,
            -index,
            index,
            Liquidity::new(10.into()),
            SqrtPrice::new(0),
            SqrtPrice::max_instance(),
        )
        .assert_success();
    }

    let result = get_liquidity_ticks(&invariant, pool_key, ticks.clone()).unwrap();
    assert_eq!(result.len(), LIQUIDITY_TICK_LIMIT);
}

#[test]
fn test_get_liquidity_ticks_partial_query() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let initial_amount = 10u128.pow(10);
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount.into(), initial_amount.into()));

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

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, initial_amount.into())
        .assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, initial_amount.into())
        .assert_success();

    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_amount.into(),
        None::<&str>,
    );
    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        token_y,
        initial_amount.into(),
        None::<&str>,
    );
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
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

    let result_1 = get_liquidity_ticks(&invariant, pool_key, vec![-10i32, 10]).unwrap();
    assert_eq!(result_1.len(), 2);

    let result_2 = get_liquidity_ticks(&invariant, pool_key, vec![10]).unwrap();
    assert_eq!(result_2.len(), 1);

    assert_eq!(result_1[1], result_2[0]);
}
