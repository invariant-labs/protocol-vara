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
    MAX_SQRT_PRICE, MIN_SQRT_PRICE,
};
use sails_rtl::ActorId;

#[test]
fn test_swap() {
    let sys = System::new();
    sys.init_logger();
    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (token_x_program, token_y_program) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);
    init_basic_swap(&sys, &invariant, &token_x_program, &token_y_program);
}
#[test]
fn test_swap_not_enough_tokens_x() {
    let sys = System::new();
    sys.init_logger();
    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (token_x_program, token_y_program) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);

    let fee = Percentage::from_scale(6, 3);
    let tick_spacing = 10;
    let fee_tier = FeeTier { fee, tick_spacing };
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    let amount = U256::from(1000);
    mint(&token_x_program, REGULAR_USER_2, amount - U256::one()).assert_success();

    increase_allowance(&token_x_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), U256::from(999));

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), U256::from(500));
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), U256::from(1000));

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);
    let slippage = SqrtPrice::new(MIN_SQRT_PRICE.into());

    let deposit_amount = swap_amount.get() - U256::from(1);
    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_2,
            TOKEN_X_ID,
            deposit_amount,
            None::<&str>
        ),
        Some(TokenAmount(deposit_amount))
    );

    swap(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        true,
        swap_amount,
        true,
        slippage,
    )
    .assert_panicked_with(InvariantError::FailedToChangeTokenBalance);

    assert_eq!(
        withdraw_single_token(&invariant, REGULAR_USER_2, TOKEN_X_ID, None, None::<&str>),
        Some(TokenAmount(deposit_amount))
    );
    assert_eq!(
        withdraw_single_token(
            &invariant,
            REGULAR_USER_2,
            TOKEN_Y_ID,
            None,
            InvariantError::NoBalanceForTheToken.into()
        ),
        None
    );

    let pool_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool_before, pool_after);

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), U256::from(999));
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), U256::from(0));

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), U256::from(500));
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), U256::from(1000));

    assert_eq!(
        pool_after.fee_growth_global_x,
        FeeGrowth::new(U128::from(0))
    );
    assert_eq!(
        pool_after.fee_growth_global_y,
        FeeGrowth::new(U128::from(0))
    );

    assert_eq!(
        pool_after.fee_protocol_token_x,
        TokenAmount::new(U256::from(0))
    );
    assert_eq!(
        pool_after.fee_protocol_token_y,
        TokenAmount::new(U256::from(0))
    );
}

#[test]
fn test_swap_not_enough_tokens_y() {
    let sys = System::new();
    sys.init_logger();
    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (token_x_program, token_y_program) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);

    let fee = Percentage::from_scale(6, 3);
    let tick_spacing = 10;
    let fee_tier = FeeTier { fee, tick_spacing };
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    let amount = U256::from(500);
    mint(&token_y_program, REGULAR_USER_2, amount - U256::one()).assert_success();

    increase_allowance(&token_y_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), U256::from(499));

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), U256::from(500));
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), U256::from(1000));

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);
    let slippage = SqrtPrice::new(MAX_SQRT_PRICE.into());

    let deposit_amount = swap_amount.get() - U256::from(1);

    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_2,
            TOKEN_Y_ID,
            deposit_amount,
            None::<&str>
        ),
        Some(TokenAmount(deposit_amount))
    );

    swap(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        false,
        swap_amount,
        true,
        slippage,
    )
    .assert_panicked_with(InvariantError::FailedToChangeTokenBalance);

    assert_eq!(
        withdraw_single_token(&invariant, REGULAR_USER_2, TOKEN_Y_ID, None, None::<&str>),
        Some(TokenAmount(deposit_amount))
    );

    let pool_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool_before, pool_after);

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), U256::from(0));
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), U256::from(499));

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), U256::from(500));
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), U256::from(1000));

    assert_eq!(
        pool_after.fee_growth_global_x,
        FeeGrowth::new(U128::from(0))
    );
    assert_eq!(
        pool_after.fee_growth_global_y,
        FeeGrowth::new(U128::from(0))
    );

    assert_eq!(
        pool_after.fee_protocol_token_x,
        TokenAmount::new(U256::from(0))
    );
    assert_eq!(
        pool_after.fee_protocol_token_y,
        TokenAmount::new(U256::from(0))
    );
}

#[test]
fn test_swap_not_enough_liquidity_token_y() {
    let sys = System::new();
    sys.init_logger();
    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (token_x_program, token_y_program) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);
    let fee = Percentage::from_scale(6, 3);
    let tick_spacing = 10;
    let fee_tier = FeeTier { fee, tick_spacing };
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    let amount = U256::from(1000);
    mint(&token_y_program, REGULAR_USER_2, amount).assert_success();

    increase_allowance(&token_y_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);
    let slippage = SqrtPrice::new(MAX_SQRT_PRICE.into());

    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), U256::from(1000));
    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), U256::from(500));
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), U256::from(1000));

    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_2,
            TOKEN_Y_ID,
            swap_amount.get(),
            None::<&str>
        ),
        Some(swap_amount)
    );

    swap(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        false,
        swap_amount,
        true,
        slippage,
    )
    .assert_panicked_with(InvariantError::TickLimitReached);

    assert_eq!(
        withdraw_single_token(&invariant, REGULAR_USER_2, TOKEN_Y_ID, None, None::<&str>),
        Some(swap_amount)
    );

    let pool_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool_before, pool_after);

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), U256::from(0));
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), U256::from(1000));

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), U256::from(500));
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), U256::from(1000));

    assert_eq!(
        pool_after.fee_growth_global_x,
        FeeGrowth::new(U128::from(0))
    );
    assert_eq!(
        pool_after.fee_growth_global_y,
        FeeGrowth::new(U128::from(0))
    );

    assert_eq!(
        pool_after.fee_protocol_token_x,
        TokenAmount::new(U256::from(0))
    );
    assert_eq!(
        pool_after.fee_protocol_token_y,
        TokenAmount::new(U256::from(0))
    );
}

#[test]
fn test_swap_not_enough_liquidity_token_x() {
    let sys = System::new();
    sys.init_logger();
    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (token_x_program, token_y_program) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);
    let fee = Percentage::from_scale(6, 3);
    let tick_spacing = 10;
    let fee_tier = FeeTier { fee, tick_spacing };
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    let amount = U256::from(2000);
    mint(&token_x_program, REGULAR_USER_2, amount).assert_success();

    increase_allowance(&token_x_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);
    let slippage = SqrtPrice::new(MIN_SQRT_PRICE.into());

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), U256::from(2000));
    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), U256::from(500));
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), U256::from(1000));

    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_2,
            TOKEN_X_ID,
            swap_amount.get(),
            None::<&str>
        ),
        Some(swap_amount)
    );

    swap(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        true,
        swap_amount,
        true,
        slippage,
    )
    .assert_panicked_with(InvariantError::TickLimitReached);

    assert_eq!(
        withdraw_single_token(&invariant, REGULAR_USER_2, TOKEN_X_ID, None, None::<&str>),
        Some(swap_amount)
    );
    assert_eq!(
        withdraw_single_token(
            &invariant,
            REGULAR_USER_2,
            TOKEN_Y_ID,
            None,
            InvariantError::NoBalanceForTheToken.into()
        ),
        None
    );

    let pool_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool_before, pool_after);

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), U256::from(2000));
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), U256::from(0));

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), U256::from(500));
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), U256::from(1000));

    assert_eq!(
        pool_after.fee_growth_global_x,
        FeeGrowth::new(U128::from(0))
    );
    assert_eq!(
        pool_after.fee_growth_global_y,
        FeeGrowth::new(U128::from(0))
    );

    assert_eq!(
        pool_after.fee_protocol_token_x,
        TokenAmount::new(U256::from(0))
    );
    assert_eq!(
        pool_after.fee_protocol_token_y,
        TokenAmount::new(U256::from(0))
    );
}

#[test]
fn test_swap_x_to_y() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let invariant = init_invariant(&sys, Percentage::from_scale(6, 3));
    let initial_amount = U256::from(10u128.pow(10));
    let (token_x_program, token_y_program) = init_tokens(&sys);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

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

    mint(&token_x_program, REGULAR_USER_1, initial_amount).assert_success();
    mint(&token_y_program, REGULAR_USER_1, initial_amount).assert_success();
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

    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_1,
            TOKEN_X_ID,
            initial_amount,
            None::<&str>
        ),
        Some(TokenAmount(initial_amount))
    );
    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_1,
            TOKEN_Y_ID,
            initial_amount,
            None::<&str>
        ),
        Some(TokenAmount(initial_amount))
    );

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    let lower_tick_index = -20;
    let middle_tick_index = -10;
    let upper_tick_index = 10;

    let liquidity_delta = Liquidity::from_integer(1000000);

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        SqrtPrice::new(U128::from(0)),
        SqrtPrice::new(MAX_SQRT_PRICE.into()),
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick_index - 20,
        middle_tick_index,
        liquidity_delta,
        SqrtPrice::new(U128::from(0)),
        SqrtPrice::new(MAX_SQRT_PRICE.into()),
    )
    .assert_success();

    assert!(
        withdraw_single_token(&invariant, REGULAR_USER_1, TOKEN_X_ID, None, None::<&str>).is_some()
    );
    assert!(
        withdraw_single_token(&invariant, REGULAR_USER_1, TOKEN_Y_ID, None, None::<&str>).is_some()
    );

    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool.liquidity, liquidity_delta);

    let amount = U256::from(1000);
    let swap_amount = TokenAmount(amount);
    mint(&token_x_program, REGULAR_USER_2, amount).assert_success();

    increase_allowance(&token_x_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    let before_invariant_x = balance_of(&token_x_program, INVARIANT_ID);
    let before_invariant_y = balance_of(&token_y_program, INVARIANT_ID);
    swap_exact_limit(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        true,
        swap_amount,
        true,
    );

    // Load states
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    let lower_tick = get_tick(&invariant, pool_key, lower_tick_index).unwrap();
    let middle_tick = get_tick(&invariant, pool_key, middle_tick_index).unwrap();
    let upper_tick = get_tick(&invariant, pool_key, upper_tick_index).unwrap();
    let lower_tick_bit = is_tick_initialized(&invariant, pool_key, lower_tick_index);
    let middle_tick_bit = is_tick_initialized(&invariant, pool_key, middle_tick_index);
    let upper_tick_bit = is_tick_initialized(&invariant, pool_key, upper_tick_index);
    let user_x = balance_of(&token_x_program, REGULAR_USER_2);
    let user_y = balance_of(&token_y_program, REGULAR_USER_2);
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID);

    // Check balances
    let delta_invariant_y = before_invariant_y - invariant_y;
    let delta_invariant_x = invariant_x - before_invariant_x;
    let expected_x = U256::from(0);
    let expected_y = amount - U256::from(10);

    // Check balances
    assert_eq!(user_x, expected_x);
    assert_eq!(user_y, expected_y);
    assert_eq!(delta_invariant_x, amount);
    assert_eq!(delta_invariant_y, expected_y);

    // Check Pool
    assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(U128::from(0)));
    assert_eq!(
        pool.fee_growth_global_x,
        FeeGrowth::new(U128::from(40000000000000000000000u128))
    );
    assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(U256::from(0)));
    assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(U256::from(2)));

    // Check Ticks
    assert_eq!(lower_tick.liquidity_change, liquidity_delta);
    assert_eq!(middle_tick.liquidity_change, liquidity_delta);
    assert_eq!(upper_tick.liquidity_change, liquidity_delta);
    assert_eq!(
        upper_tick.fee_growth_outside_x,
        FeeGrowth::new(U128::from(0))
    );
    assert_eq!(
        middle_tick.fee_growth_outside_x,
        FeeGrowth::new(U128::from(30000000000000000000000u128))
    );
    assert_eq!(
        lower_tick.fee_growth_outside_x,
        FeeGrowth::new(U128::from(0))
    );
    assert!(lower_tick_bit);
    assert!(middle_tick_bit);
    assert!(upper_tick_bit);
}

#[test]
fn test_swap_y_to_x() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let invariant = init_invariant(&sys, Percentage::from_scale(6, 3));
    let initial_amount = U256::from(10u128.pow(10));
    let (token_x_program, token_y_program) = init_tokens(&sys);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

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

    mint(&token_x_program, REGULAR_USER_1, initial_amount).assert_success();
    mint(&token_y_program, REGULAR_USER_1, initial_amount).assert_success();
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_1), initial_amount);
    assert_eq!(balance_of(&token_x_program, REGULAR_USER_1), initial_amount);

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

    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_1,
            TOKEN_X_ID,
            initial_amount,
            None::<&str>
        ),
        Some(TokenAmount(initial_amount))
    );
    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_1,
            TOKEN_Y_ID,
            initial_amount,
            None::<&str>
        ),
        Some(TokenAmount(initial_amount))
    );

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    let lower_tick_index = -10;
    let middle_tick_index = 10;
    let upper_tick_index = 20;

    let liquidity_delta = Liquidity::from_integer(1000000);

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        SqrtPrice::new(U128::from(0)),
        SqrtPrice::new(MAX_SQRT_PRICE.into()),
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        middle_tick_index,
        upper_tick_index + 20,
        liquidity_delta,
        SqrtPrice::new(U128::from(0)),
        SqrtPrice::new(MAX_SQRT_PRICE.into()),
    )
    .assert_success();

    assert!(
        withdraw_single_token(&invariant, REGULAR_USER_1, TOKEN_X_ID, None, None::<&str>).is_some()
    );
    assert!(
        withdraw_single_token(&invariant, REGULAR_USER_1, TOKEN_Y_ID, None, None::<&str>).is_some()
    );
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool.liquidity, liquidity_delta);

    let amount = U256::from(1000);
    let swap_amount = TokenAmount(amount);

    mint(&token_y_program, REGULAR_USER_2, amount).assert_success();

    increase_allowance(&token_y_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    let before_invariant_x = balance_of(&token_x_program, INVARIANT_ID);
    let before_invariant_y = balance_of(&token_y_program, INVARIANT_ID);
    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), U256::from(2499));
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), U256::from(500));
    swap_exact_limit(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        false,
        swap_amount,
        true,
    );

    // Load states
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    let lower_tick = get_tick(&invariant, pool_key, lower_tick_index).unwrap();
    let middle_tick = get_tick(&invariant, pool_key, middle_tick_index).unwrap();
    let upper_tick = get_tick(&invariant, pool_key, upper_tick_index).unwrap();
    let lower_tick_bit = is_tick_initialized(&invariant, pool_key, lower_tick_index);
    let middle_tick_bit = is_tick_initialized(&invariant, pool_key, middle_tick_index);
    let upper_tick_bit = is_tick_initialized(&invariant, pool_key, upper_tick_index);
    let user_x = balance_of(&token_x_program, REGULAR_USER_2);
    let user_y = balance_of(&token_y_program, REGULAR_USER_2);
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID);
    let delta_invariant_x = before_invariant_x - invariant_x;
    let delta_invariant_y = invariant_y - before_invariant_y;
    let expected_x = amount - U256::from(10);
    let expected_y = U256::from(0);

    // Check balances
    assert_eq!(user_x, expected_x);
    assert_eq!(user_y, expected_y);
    assert_eq!(delta_invariant_x, expected_x);
    assert_eq!(delta_invariant_y, amount);

    // Check Pool
    assert_eq!(pool.fee_growth_global_x, FeeGrowth::new(U128::from(0)));
    assert_eq!(
        pool.fee_growth_global_y,
        FeeGrowth::new(U128::from(40000000000000000000000u128))
    );
    assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(U256::from(0)));
    assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(U256::from(2)));

    // Check Ticks
    assert_eq!(lower_tick.liquidity_change, liquidity_delta);
    assert_eq!(middle_tick.liquidity_change, liquidity_delta);
    assert_eq!(upper_tick.liquidity_change, liquidity_delta);
    assert_eq!(
        upper_tick.fee_growth_outside_y,
        FeeGrowth::new(U128::from(0))
    );
    assert_eq!(
        middle_tick.fee_growth_outside_y,
        FeeGrowth::new(U128::from(30000000000000000000000u128))
    );
    assert_eq!(
        lower_tick.fee_growth_outside_y,
        FeeGrowth::new(U128::from(0))
    );
    assert!(lower_tick_bit);
    assert!(middle_tick_bit);
    assert!(upper_tick_bit);
}
