use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gtest::*;
use math::{
    get_delta_x, get_delta_y,
    liquidity::Liquidity,
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, get_max_tick, SqrtPrice},
    token_amount::TokenAmount,
    MAX_SQRT_PRICE, MAX_TICK, MIN_SQRT_PRICE,
};
use sails_rtl::ActorId;

#[test]
fn test_limits_big_deposit_x_and_swap_y() {
    let sys = System::new();
    sys.init_logger();

    big_deposit_and_swap(&sys, true);
}

#[test]
fn test_limits_big_deposit_y_and_swap_x() {
    let sys = System::new();
    sys.init_logger();

    big_deposit_and_swap(&sys, false);
}

#[test]
fn test_limits_big_deposit_both_tokens() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);
    let mint_amount = u128::MAX;
    let approved_amount = 2u128.pow(75) - 1;

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (mint_amount, mint_amount));

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, u128::MAX).assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, u128::MAX).assert_success();

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

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

    let lower_tick = -(fee_tier.tick_spacing as i32);
    let upper_tick = fee_tier.tick_spacing as i32;
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    let liquidity_delta = get_liquidity_by_x(
        TokenAmount(approved_amount),
        lower_tick,
        upper_tick,
        pool.sqrt_price,
        false,
    )
    .unwrap()
    .l;
    let y: TokenAmount = get_delta_y(
        calculate_sqrt_price(lower_tick).unwrap(),
        pool.sqrt_price,
        liquidity_delta,
        true,
    )
    .unwrap();
    let x: TokenAmount = get_delta_x(
        calculate_sqrt_price(lower_tick).unwrap(),
        pool.sqrt_price,
        liquidity_delta,
        true,
    )
    .unwrap();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let slippage_limit_lower = pool.sqrt_price;
    let slippage_limit_upper = pool.sqrt_price;

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        x.get(),
        token_y,
        y.get(),
        None::<&str>,
    )
    .unwrap();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick,
        upper_tick,
        liquidity_delta,
        slippage_limit_lower,
        slippage_limit_upper,
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

    let user_amount_x = balance_of(&token_x_program, REGULAR_USER_1);
    let user_amount_y = balance_of(&token_y_program, REGULAR_USER_1);
    assert_eq!(user_amount_x, u128::MAX - approved_amount);
    assert_eq!(user_amount_y, u128::MAX - y.get());

    let contract_amount_x = balance_of(&token_x_program, INVARIANT_ID);
    let contract_amount_y = balance_of(&token_y_program, INVARIANT_ID);
    assert_eq!(contract_amount_x, approved_amount);
    assert_eq!(contract_amount_y, y.get());
}

#[test]
fn test_deposit_limits_at_upper_limit() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);
    let mint_amount = 2u128.pow(75) - 1;

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (mint_amount, mint_amount));

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, u128::MAX).assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, u128::MAX).assert_success();

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = get_max_tick(1);
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

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

    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    assert_eq!(pool.current_tick_index, init_tick);
    assert_eq!(pool.sqrt_price, calculate_sqrt_price(init_tick).unwrap());

    let position_amount = mint_amount - 1;

    let liquidity_delta = get_liquidity_by_y(
        TokenAmount(position_amount),
        0,
        MAX_TICK,
        pool.sqrt_price,
        false,
    )
    .unwrap()
    .l;

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let slippage_limit_lower = pool.sqrt_price;
    let slippage_limit_upper = pool.sqrt_price;

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
        0,
        MAX_TICK,
        liquidity_delta,
        slippage_limit_lower,
        slippage_limit_upper,
    )
    .assert_success();
}

#[test]
fn test_limits_big_deposit_and_swaps() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);
    let mint_amount = 2u128.pow(76) - 1;

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (u128::MAX, u128::MAX));

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, u128::MAX).assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, u128::MAX).assert_success();

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
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

    let pos_amount = mint_amount / 2;
    let lower_tick = -(fee_tier.tick_spacing as i32);
    let upper_tick = fee_tier.tick_spacing as i32;
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let liquidity_delta = get_liquidity_by_x(
        TokenAmount(pos_amount),
        lower_tick,
        upper_tick,
        pool.sqrt_price,
        false,
    )
    .unwrap()
    .l;

    let y = get_delta_y(
        calculate_sqrt_price(lower_tick).unwrap(),
        pool.sqrt_price,
        liquidity_delta,
        true,
    )
    .unwrap();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let slippage_limit_lower = pool.sqrt_price;
    let slippage_limit_upper = pool.sqrt_price;

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        u128::MAX,
        token_y,
        u128::MAX,
        None::<&str>,
    )
    .unwrap();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick,
        upper_tick,
        liquidity_delta,
        slippage_limit_lower,
        slippage_limit_upper,
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

    let user_amount_x = balance_of(&token_x_program, REGULAR_USER_1);
    let user_amount_y = balance_of(&token_y_program, REGULAR_USER_1);
    assert_eq!(user_amount_x, u128::MAX - pos_amount);
    assert_eq!(user_amount_y, u128::MAX - y.get());

    let contract_amount_x = balance_of(&token_x_program, INVARIANT_ID);
    let contract_amount_y = balance_of(&token_y_program, INVARIANT_ID);
    assert_eq!(contract_amount_x, pos_amount);
    assert_eq!(contract_amount_y, y.get());

    let swap_amount = TokenAmount(mint_amount / 8);

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        user_amount_x,
    )
    .assert_success();
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        user_amount_y,
    )
    .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        user_amount_x,
        token_y,
        user_amount_y,
        None::<&str>,
    )
    .unwrap();

    for i in 1..=4 {
        let (_, sqrt_price_limit) = if i % 2 == 0 {
            (true, SqrtPrice::new(MIN_SQRT_PRICE))
        } else {
            (false, SqrtPrice::new(MAX_SQRT_PRICE))
        };

        swap(
            &invariant,
            REGULAR_USER_1,
            pool_key,
            i % 2 == 0,
            swap_amount,
            true,
            sqrt_price_limit,
        )
        .assert_success();
    }
}

#[test]
fn test_limits_full_range_with_max_liquidity() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let mint_amount = u128::MAX;
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (mint_amount, mint_amount));
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();
    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = get_max_tick(1);
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

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

    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    assert_eq!(pool.current_tick_index, init_tick);
    assert_eq!(pool.sqrt_price, calculate_sqrt_price(init_tick).unwrap());

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let liquidity_delta = Liquidity::new(2u128.pow(109) - 1);
    let slippage_limit_lower = pool.sqrt_price;
    let slippage_limit_upper = pool.sqrt_price;

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
        -MAX_TICK,
        MAX_TICK,
        liquidity_delta,
        slippage_limit_lower,
        slippage_limit_upper,
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

    let contract_amount_x = balance_of(&token_x_program, INVARIANT_ID);
    let contract_amount_y = balance_of(&token_y_program, INVARIANT_ID);

    let expected_x = 0;
    let expected_y = 42534896005851865508212194815854; // < 2^106
    assert_eq!(contract_amount_x, expected_x);
    assert_eq!(contract_amount_y, expected_y);
}
