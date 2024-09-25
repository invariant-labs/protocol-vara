use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{liquidity::Liquidity, sqrt_price::SqrtPrice};
use sails_rs::prelude::*;

#[test]
fn test_position_slippage_zero_slippage_and_inside_range() {
    let sys = System::new();

    let (invariant, token_x_program, token_y_program) = init_slippage_invariant_and_tokens(&sys);
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let pool_key =
        init_slippage_pool_with_liquidity(&invariant, &token_x_program, &token_y_program);

    let pool = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();
    let amount_y = balance_of(&token_y_program, REGULAR_USER_1);
    let amount_x = balance_of(&token_x_program, REGULAR_USER_1);

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, amount_x).assert_success();

    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, amount_y).assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        amount_x,
        token_y,
        amount_y,
        None::<&str>,
    )
    .unwrap();

    // zero slippage
    {
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let known_price = pool.sqrt_price;
        let tick = pool_key.fee_tier.tick_spacing as i32;

        create_position(
            &invariant,
            REGULAR_USER_1,
            pool_key,
            -tick,
            tick,
            liquidity_delta,
            known_price,
            known_price,
        )
        .assert_success();
    };

    // inside range
    {
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let limit_lower = SqrtPrice::new(994734637981406576896367u128);
        let limit_upper = SqrtPrice::new(1025038048074314166333500u128);

        let tick = pool_key.fee_tier.tick_spacing as i32;

        create_position(
            &invariant,
            REGULAR_USER_1,
            pool_key,
            -tick,
            tick,
            liquidity_delta,
            limit_lower,
            limit_upper,
        )
        .assert_success();
    }
}
#[test]
fn test_position_slippage_below_range() {
    let sys = System::new();
    let (invariant, token_x_program, token_y_program) = init_slippage_invariant_and_tokens(&sys);
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let pool_key =
        init_slippage_pool_with_liquidity(&invariant, &token_x_program, &token_y_program);

    let _pool = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();

    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let limit_lower = SqrtPrice::new(1014432353584998786339859u128);
    let limit_upper = SqrtPrice::new(1045335831204498605270797u128);
    let tick = pool_key.fee_tier.tick_spacing as i32;

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -tick,
        tick,
        liquidity_delta,
        limit_lower,
        limit_upper,
    )
    .assert_panicked_with(InvariantError::PriceLimitReached);

    let _lower_tick = get_tick(&invariant, pool_key, -tick).unwrap_err();
    let _upper_tick = get_tick(&invariant, pool_key, tick).unwrap_err();
}

#[test]
fn test_position_slippage_above_range() {
    let sys = System::new();
    let (invariant, token_x_program, token_y_program) = init_slippage_invariant_and_tokens(&sys);
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let pool_key =
        init_slippage_pool_with_liquidity(&invariant, &token_x_program, &token_y_program);

    let _pool = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();

    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let limit_lower = SqrtPrice::new(955339206774222158009382u128);
    let limit_upper = SqrtPrice::new(984442481813945288458906u128);
    let tick = pool_key.fee_tier.tick_spacing as i32;

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -tick,
        tick,
        liquidity_delta,
        limit_lower,
        limit_upper,
    )
    .assert_panicked_with(InvariantError::PriceLimitReached);

    let _lower_tick = get_tick(&invariant, pool_key, -tick).unwrap_err();
    let _upper_tick = get_tick(&invariant, pool_key, tick).unwrap_err();
}
