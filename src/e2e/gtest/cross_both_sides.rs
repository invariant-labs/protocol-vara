use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gstd::{prelude::*, ActorId};
use gtest::*;
use io::*;
use math::{
    fee_growth::FeeGrowth,
    liquidity::Liquidity,
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
    token_amount::TokenAmount,
    MAX_SQRT_PRICE, MIN_SQRT_PRICE,
};
#[test]
fn test_cross_both_sides() {
    let sys = System::new();
    sys.init_logger();

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    let mint_amount = 10u128.pow(5);

    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (mint_amount, mint_amount));

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    invariant
        .send(ADMIN, InvariantAction::AddFeeTier(fee_tier))
        .assert_success();

    assert!(invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePool {
                token_0: token_x,
                token_1: token_y,
                fee_tier,
                init_sqrt_price,
                init_tick,
            }
        )
        .events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    let lower_tick_index = -10;
    let upper_tick_index = 10;

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID.into(),
        mint_amount,
    )
    .assert_success();
    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID.into(),
        mint_amount,
    )
    .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        mint_amount,
        token_y,
        mint_amount,
        None::<&str>,
    )
    .unwrap();

    let liquidity_delta = Liquidity::from_integer(20006000);

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePosition {
                pool_key,
                lower_tick: lower_tick_index,
                upper_tick: upper_tick_index,
                liquidity_delta,
                slippage_limit_lower: pool_state.sqrt_price,
                slippage_limit_upper: pool_state.sqrt_price,
            },
        )
        .assert_success();

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePosition {
                pool_key,
                lower_tick: -20,
                upper_tick: lower_tick_index,
                liquidity_delta,
                slippage_limit_lower: pool_state.sqrt_price,
                slippage_limit_upper: pool_state.sqrt_price,
            },
        )
        .assert_success();

    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool.liquidity, liquidity_delta);

    let limit_without_cross_tick_amount = TokenAmount(10_068);
    let not_cross_amount = TokenAmount(1);
    let min_amount_to_cross_from_tick_price = TokenAmount(3);
    let crossing_amount_by_amount_out = TokenAmount(20136101434);

    let mint_amount = limit_without_cross_tick_amount.get()
        + not_cross_amount.get()
        + min_amount_to_cross_from_tick_price.get()
        + crossing_amount_by_amount_out.get();

    mint(&token_x_program, REGULAR_USER_1, mint_amount).assert_success();
    mint(&token_y_program, REGULAR_USER_1, mint_amount).assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID.into(),
        mint_amount,
    )
    .assert_success();
    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID.into(),
        mint_amount,
    )
    .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        mint_amount,
        token_y,
        mint_amount,
        None::<&str>,
    )
    .unwrap();

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    let limit_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::Swap {
                pool_key,
                x_to_y: true,
                amount: limit_without_cross_tick_amount,
                by_amount_in: true,
                sqrt_price_limit: limit_sqrt_price,
            },
        )
        .assert_success();

    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    let expected_tick = -10;
    let expected_price = calculate_sqrt_price(expected_tick).unwrap();

    assert_eq!(pool.current_tick_index, expected_tick);
    assert_eq!(pool.liquidity, pool_before.liquidity);
    assert_eq!(pool.sqrt_price, expected_price);

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::Swap {
                pool_key,
                x_to_y: true,
                amount: min_amount_to_cross_from_tick_price,
                by_amount_in: true,
                sqrt_price_limit: limit_sqrt_price,
            },
        )
        .assert_success();

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::Swap {
                pool_key,
                x_to_y: false,
                amount: min_amount_to_cross_from_tick_price,
                by_amount_in: true,
                sqrt_price_limit: SqrtPrice::new(MAX_SQRT_PRICE),
            },
        )
        .assert_success();

    let massive_x = 10u128.pow(19);
    let massive_y = 10u128.pow(19);

    mint(&token_x_program, REGULAR_USER_1, massive_x).assert_success();

    mint(&token_y_program, REGULAR_USER_1, massive_y).assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID.into(),
        massive_y,
    )
    .assert_success();
    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID.into(),
        massive_x,
    )
    .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        massive_y,
        token_y,
        massive_x,
        None::<&str>,
    )
    .unwrap();

    let massive_liquidity_delta = Liquidity::from_integer(19996000399699881985603u128);

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePosition {
                pool_key,
                lower_tick: -20,
                upper_tick: 0,
                liquidity_delta: massive_liquidity_delta,
                slippage_limit_lower: SqrtPrice::new(MIN_SQRT_PRICE),
                slippage_limit_upper: SqrtPrice::new(MAX_SQRT_PRICE),
            },
        )
        .assert_success();

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::Swap {
                pool_key,
                x_to_y: true,
                amount: TokenAmount(1),
                by_amount_in: false,
                sqrt_price_limit: limit_sqrt_price,
            },
        )
        .assert_success();

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::Swap {
                pool_key,
                x_to_y: false,
                amount: TokenAmount(2),
                by_amount_in: true,
                sqrt_price_limit: SqrtPrice::new(MAX_SQRT_PRICE),
            },
        )
        .assert_success();

    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let expected_liquidity = Liquidity::from_integer(19996000399699901991603u128);
    let expected_liquidity_change_on_last_tick =
        Liquidity::from_integer(19996000399699901991603u128);
    let expected_liquidity_change_on_upper_tick = Liquidity::from_integer(20006000);

    assert_eq!(pool.current_tick_index, -20);
    assert_eq!(
        pool.fee_growth_global_x,
        FeeGrowth::new(29991002699190242927121)
    );
    assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(0));
    assert_eq!(pool.fee_protocol_token_x, TokenAmount(4));
    assert_eq!(pool.fee_protocol_token_y, TokenAmount(2));
    assert_eq!(pool.liquidity, expected_liquidity);
    assert_eq!(pool.sqrt_price, SqrtPrice::new(999500149964999999999999));

    let final_last_tick = get_tick(&invariant, pool_key, -20).unwrap();
    assert_eq!(final_last_tick.fee_growth_outside_x, FeeGrowth::new(0));
    assert_eq!(final_last_tick.fee_growth_outside_y, FeeGrowth::new(0));
    assert_eq!(
        final_last_tick.liquidity_change,
        expected_liquidity_change_on_last_tick
    );

    let final_lower_tick = get_tick(&invariant, pool_key, -10).unwrap();
    assert_eq!(
        final_lower_tick.fee_growth_outside_x,
        FeeGrowth::new(29991002699190242927121)
    );
    assert_eq!(final_lower_tick.fee_growth_outside_y, FeeGrowth::new(0));
    assert_eq!(final_lower_tick.liquidity_change, Liquidity::new(0));

    let final_upper_tick = get_tick(&invariant, pool_key, 10).unwrap();
    assert_eq!(final_upper_tick.fee_growth_outside_x, FeeGrowth::new(0));
    assert_eq!(final_upper_tick.fee_growth_outside_y, FeeGrowth::new(0));
    assert_eq!(
        final_upper_tick.liquidity_change,
        expected_liquidity_change_on_upper_tick
    );
}
#[test]
fn test_cross_both_sides_not_cross_case() {
    let sys = System::new();
    sys.init_logger();

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    let mint_amount = 10u128.pow(5);

    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let mut invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (mint_amount, mint_amount));

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    invariant
        .send(ADMIN, InvariantAction::AddFeeTier(fee_tier))
        .assert_success();

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePool {
                token_0: token_x,
                token_1: token_y,
                fee_tier,
                init_sqrt_price,
                init_tick,
            },
        )
        .assert_success();

    let lower_tick_index = -10;
    let upper_tick_index = 10;

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();

    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        mint_amount,
        token_y,
        mint_amount,
        None::<&str>,
    )
    .unwrap();

    let liquidity_delta = Liquidity::new(20006000000000);

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePosition {
                pool_key,
                lower_tick: lower_tick_index,
                upper_tick: upper_tick_index,
                liquidity_delta,
                slippage_limit_lower: pool_state.sqrt_price,
                slippage_limit_upper: pool_state.sqrt_price,
            },
        )
        .assert_success();

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePosition {
                pool_key,
                lower_tick: -20,
                upper_tick: lower_tick_index,
                liquidity_delta,
                slippage_limit_lower: pool_state.sqrt_price,
                slippage_limit_upper: pool_state.sqrt_price,
            },
        )
        .assert_success();

    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool.liquidity, liquidity_delta);

    let limit_without_cross_tick_amount = TokenAmount(10_068);
    let not_cross_amount = TokenAmount(1);
    let min_amount_to_cross_from_tick_price = TokenAmount(3);
    let crossing_amount_by_amount_out = TokenAmount(20136101434);

    let mint_amount = limit_without_cross_tick_amount.get()
        + not_cross_amount.get()
        + min_amount_to_cross_from_tick_price.get()
        + crossing_amount_by_amount_out.get();

    mint(&token_x_program, REGULAR_USER_1, mint_amount).assert_success();
    mint(&token_y_program, REGULAR_USER_1, mint_amount).assert_success();

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();

    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        mint_amount,
        token_y,
        mint_amount,
        None::<&str>,
    )
    .unwrap();

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let limit_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE);

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::Swap {
                pool_key,
                x_to_y: true,
                amount: limit_without_cross_tick_amount,
                by_amount_in: true,
                sqrt_price_limit: limit_sqrt_price,
            },
        )
        .assert_success();
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let expected_tick = -10;
    let expected_price = calculate_sqrt_price(expected_tick).unwrap();

    assert_eq!(pool.current_tick_index, expected_tick);
    assert_eq!(pool.liquidity, pool_before.liquidity);
    assert_eq!(pool.sqrt_price, expected_price);

    let slippage = SqrtPrice::new(MIN_SQRT_PRICE);

    invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::Swap {
            pool_key,
            x_to_y: true,
            amount: not_cross_amount,
            by_amount_in: true,
            sqrt_price_limit: slippage,
        },
        InvariantError::NoGainSwap,
    );
}
