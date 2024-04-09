use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gstd::{prelude::*, ActorId};
use gtest::*;
use io::InvariantAction;
use math::{liquidity::Liquidity, sqrt_price::SqrtPrice};

#[test]
fn test_position_slippage_zero_slippage_and_inside_range() {
    let sys = System::new();

    let (invariant, token_x_program, token_y_program) = init_slippage_invariant_and_tokens(&sys);
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let pool_key =
        init_slippage_pool_with_liquidity(&sys, &invariant, &token_x_program, &token_y_program);

    let pool = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();

    // zero slippage
    {
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let known_price = pool.sqrt_price;
        let tick = pool_key.fee_tier.tick_spacing as i32;

        let res = invariant.send(
            REGULAR_USER_1,
            InvariantAction::CreatePosition {
                pool_key,
                lower_tick: -tick,
                upper_tick: tick,
                liquidity_delta,
                slippage_limit_lower: known_price,
                slippage_limit_upper: known_price,
            },
        );
        assert!(!res.main_failed())
    };

    // inside range
    {
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let limit_lower = SqrtPrice::new(994734637981406576896367);
        let limit_upper = SqrtPrice::new(1025038048074314166333500);

        let tick = pool_key.fee_tier.tick_spacing as i32;

        let res = invariant.send(
            REGULAR_USER_1,
            InvariantAction::CreatePosition {
                pool_key,
                lower_tick: -tick,
                upper_tick: tick,
                liquidity_delta,
                slippage_limit_lower: limit_lower,
                slippage_limit_upper: limit_upper,
            },
        );
        assert!(!res.main_failed())
    }
}
#[test]
fn test_position_slippage_below_range() {
    let sys = System::new();
    let (mut invariant, token_x_program, token_y_program) = init_slippage_invariant_and_tokens(&sys);
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let pool_key =
        init_slippage_pool_with_liquidity(&sys, &invariant, &token_x_program, &token_y_program);

    let _pool = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();

    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let limit_lower = SqrtPrice::new(1014432353584998786339859);
    let limit_upper = SqrtPrice::new(1045335831204498605270797);
    let tick = pool_key.fee_tier.tick_spacing as i32;

    let _res = invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: -tick,
            upper_tick: tick,
            liquidity_delta,
            slippage_limit_lower: limit_lower,
            slippage_limit_upper: limit_upper,
        },
        InvariantError::PriceLimitReached
    );

    let _lower_tick = get_tick(
        &invariant,
        pool_key,
        -tick,
    ).unwrap_err();
    let _upper_tick = get_tick(
        &invariant,
        pool_key,
        tick,
    ).unwrap_err();
}

#[test]
fn test_position_slippage_above_range() {
    let sys = System::new();
    let (mut invariant, token_x_program, token_y_program) = init_slippage_invariant_and_tokens(&sys);
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let pool_key =
        init_slippage_pool_with_liquidity(&sys, &invariant, &token_x_program, &token_y_program);

    let _pool = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();

    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let limit_lower = SqrtPrice::new(955339206774222158009382);
    let limit_upper = SqrtPrice::new(984442481813945288458906);
    let tick = pool_key.fee_tier.tick_spacing as i32;

    let _res = invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: -tick,
            upper_tick: tick,
            liquidity_delta,
            slippage_limit_lower: limit_lower,
            slippage_limit_upper: limit_upper,
        },
        InvariantError::PriceLimitReached
    );

    let _lower_tick = get_tick(
        &invariant,
        pool_key,
        -tick,
    ).unwrap_err();
    let _upper_tick = get_tick(
        &invariant,
        pool_key,
        tick,
    ).unwrap_err();
}
