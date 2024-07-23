use crate::test_helpers::gtest::*;
use contracts::{FeeTier, PoolKey};
use decimal::*;
use gtest::System;
use math::types::{
    liquidity::Liquidity,
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
};
use sails_rtl::ActorId;

#[test]
fn test_get_positions() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::new(0));
    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (500.into(), 500.into()));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = 10;
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

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        500.into(),
        token_y,
        500.into(),
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

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -20,
        20,
        Liquidity::new(10.into()),
        SqrtPrice::new(0),
        SqrtPrice::max_instance(),
    )
    .assert_success();

    let result = get_positions(&invariant, REGULAR_USER_1, 2, 0).unwrap();
    assert_eq!(result.0.len(), 1);
    assert_eq!(result.0[0].1.len(), 2);

    assert_eq!(result.1, 2);
}

#[test]
fn test_get_positions_less_than_exist() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::new(0));
    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (500.into(), 500.into()));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = 10;
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

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        500.into(),
        token_y,
        500.into(),
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

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -20,
        20,
        Liquidity::new(10.into()),
        SqrtPrice::new(0),
        SqrtPrice::max_instance(),
    )
    .assert_success();

    let result = get_positions(&invariant, REGULAR_USER_1, 1, 0).unwrap();
    assert_eq!(result.0.len(), 1);
    assert_eq!(result.0[0].1.len(), 1);

    assert_eq!(result.1, 2);
}

#[test]
fn test_get_positions_more_than_exist() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::new(0));
    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (500.into(), 500.into()));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = 10;
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

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        500.into(),
        token_y,
        500.into(),
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

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -20,
        20,
        Liquidity::new(10.into()),
        SqrtPrice::new(0),
        SqrtPrice::max_instance(),
    )
    .assert_success();

    let result = get_positions(&invariant, REGULAR_USER_1, 3, 0).unwrap();
    assert_eq!(result.0.len(), 1);
    assert_eq!(result.0[0].1.len(), 2);

    assert_eq!(result.1, 2);
}

#[test]
fn test_get_positions_with_offset() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::new(0));
    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (500.into(), 500.into()));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = 10;
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

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        500.into(),
        token_y,
        500.into(),
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

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -20,
        20,
        Liquidity::new(10.into()),
        SqrtPrice::new(0),
        SqrtPrice::max_instance(),
    )
    .assert_success();

    let result = get_positions(&invariant, REGULAR_USER_1, 1, 1).unwrap();
    assert_eq!(result.0.len(), 1);
    assert_eq!(result.0[0].1.len(), 1);

    assert_eq!(result.1, 2);
}

#[test]
fn test_get_positions_with_offset_less_than_exist() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::new(0));
    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (500.into(), 500.into()));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = 10;
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

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        500.into(),
        token_y,
        500.into(),
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

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -20,
        20,
        Liquidity::new(10.into()),
        SqrtPrice::new(0),
        SqrtPrice::max_instance(),
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -30,
        30,
        Liquidity::new(10.into()),
        SqrtPrice::new(0),
        SqrtPrice::max_instance(),
    )
    .assert_success();

    let result = get_positions(&invariant, REGULAR_USER_1, 1, 1).unwrap();
    assert_eq!(result.0.len(), 1);
    assert_eq!(result.0[0].1.len(), 1);
    assert_eq!(result.1, 3);
}

#[test]
fn test_get_positions_with_offset_more_than_exist() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::new(0));
    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (500.into(), 500.into()));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::new(0), 1).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = 10;
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

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        500.into(),
        token_y,
        500.into(),
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

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -20,
        20,
        Liquidity::new(10.into()),
        SqrtPrice::new(0),
        SqrtPrice::max_instance(),
    )
    .assert_success();

    let result = get_positions(&invariant, REGULAR_USER_1, 2, 1).unwrap();
    assert_eq!(result.0.len(), 1);
    assert_eq!(result.0[0].1.len(), 1);
    assert_eq!(result.1, 2);
}
