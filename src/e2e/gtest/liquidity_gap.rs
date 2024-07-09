use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gtest::*;
use math::{
    fee_growth::FeeGrowth,
    liquidity::Liquidity,
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
    token_amount::TokenAmount,
    MIN_SQRT_PRICE,
};
use sails_rtl::ActorId;

#[test]
fn test_liquidity_gap() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    let mint_amount = U256::from(u128::MAX);

    let (token_x_program, token_y_program) = init_tokens(&sys);

    mint(&token_x_program, REGULAR_USER_1, mint_amount).assert_success();
    mint(&token_y_program, REGULAR_USER_1, mint_amount).assert_success();

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let _res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_1);

    let lower_tick_index = -10;
    let upper_tick_index = 10;

    let liquidity_delta = Liquidity::from_integer(20_006_000);

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

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

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    withdraw_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        None,
        token_y,
        None,
        None::<&str>,
    )
    .unwrap();

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool_state.liquidity, liquidity_delta);

    let swap_amount = TokenAmount::new(U256::from(10067));

    mint(&token_x_program, REGULAR_USER_2, swap_amount.get()).assert_success();

    increase_allowance(
        &token_x_program,
        REGULAR_USER_2,
        INVARIANT_ID,
        swap_amount.get(),
    )
    .assert_success();

    let invariant_x_before = balance_of(&token_x_program, INVARIANT_ID);
    let invariant_y_before = balance_of(&token_y_program, INVARIANT_ID);

    swap_exact_limit(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        true,
        swap_amount,
        true,
    );
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    let expected_price = calculate_sqrt_price(-10).unwrap();
    let expected_y_amount_out = U256::from(9999);

    assert_eq!(pool.liquidity, liquidity_delta);
    assert_eq!(pool.current_tick_index, lower_tick_index);
    assert_eq!(pool.sqrt_price, expected_price);

    let user_x = balance_of(&token_x_program, REGULAR_USER_2);
    let user_y = balance_of(&token_y_program, REGULAR_USER_2);
    let invariant_x_after = balance_of(&token_x_program, INVARIANT_ID);
    let invariant_y_after = balance_of(&token_y_program, INVARIANT_ID);
    let delta_invariant_x = invariant_x_after - invariant_x_before;
    let delta_invariant_y = invariant_y_before - invariant_y_after;

    assert_eq!(user_x, U256::from(0));
    assert_eq!(user_y, expected_y_amount_out);
    assert_eq!(delta_invariant_x, swap_amount.get());
    assert_eq!(delta_invariant_y, expected_y_amount_out);
    assert_eq!(
        pool.fee_growth_global_x,
        FeeGrowth::new(29991002699190242927121u128)
    );
    assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(0));
    assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(U256::from(1)));
    assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(U256::from(0)));

    // No gain swap
    let swap_amount = TokenAmount::new(U256::from(1));
    let target_sqrt_price = SqrtPrice::new(MIN_SQRT_PRICE.into());

    swap(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        true,
        swap_amount,
        true,
        target_sqrt_price,
    )
    .assert_panicked_with(InvariantError::NoGainSwap);

    // Should skip gap and then swap
    let lower_tick_after_swap = -90;
    let upper_tick_after_swap = -50;
    let liquidity_delta = Liquidity::from_integer(20008000);

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        mint_amount / 10,
    )
    .assert_success();
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        mint_amount / 10,
    )
    .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        mint_amount / 10,
        token_y,
        mint_amount / 10,
        None::<&str>,
    )
    .unwrap();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick_after_swap,
        upper_tick_after_swap,
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    let swap_amount = TokenAmount::new(U256::from(5000));

    mint(&token_x_program, REGULAR_USER_2, swap_amount.get()).assert_success();

    increase_allowance(
        &token_x_program,
        REGULAR_USER_2,
        INVARIANT_ID,
        swap_amount.get(),
    )
    .assert_success();

    swap_exact_limit(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        true,
        swap_amount,
        true,
    );

    get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
}
