use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{
    percentage::Percentage, sqrt_price::SqrtPrice, token_amount::TokenAmount, MAX_SQRT_PRICE,
};
use sails_rs::ActorId;

#[test]
fn test_basic_slippage() {
    let sys = System::new();
    sys.init_logger();

    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (invariant, token_x_program, token_y_program) = init_slippage_invariant_and_tokens(&sys);

    let pool_key =
        init_slippage_pool_with_liquidity(&invariant, &token_x_program, &token_y_program);

    let amount = U256::from(10u128.pow(8));
    let swap_amount = TokenAmount(amount);
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        swap_amount.get(),
    )
    .assert_success();

    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_1,
            TOKEN_Y_ID,
            swap_amount.get(),
            None::<&str>
        ),
        Some(swap_amount)
    );

    let target_sqrt_price = SqrtPrice::new(1009940000000000000000001u128);
    swap(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        false,
        swap_amount,
        true,
        target_sqrt_price,
    )
    .assert_success();

    let expected_sqrt_price = SqrtPrice::new(1009940000000000000000000u128);
    let pool = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();
    assert_eq!(pool.sqrt_price, expected_sqrt_price);
}

#[test]
fn test_swap_close_to_limit() {
    let sys = System::new();
    sys.init_logger();

    let (invariant, token_x_program, token_y_program) = init_slippage_invariant_and_tokens(&sys);
    let pool_key =
        init_slippage_pool_with_liquidity(&invariant, &token_x_program, &token_y_program);
    let amount = U256::from(10u128.pow(8));
    let swap_amount = TokenAmount::new(amount);
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        swap_amount.get(),
    )
    .assert_success();
    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_1,
            TOKEN_Y_ID,
            swap_amount.get(),
            None::<&str>
        ),
        Some(swap_amount)
    );

    let target_sqrt_price = SqrtPrice::new(MAX_SQRT_PRICE.into());
    let quoted_target_sqrt_price = quote(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        false,
        swap_amount,
        true,
        target_sqrt_price,
    )
    .unwrap()
    .target_sqrt_price;

    let target_sqrt_price = quoted_target_sqrt_price - SqrtPrice::new(1);

    swap(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        false,
        swap_amount,
        true,
        target_sqrt_price,
    )
    .assert_panicked_with(InvariantError::PriceLimitReached);
}

#[test]
fn test_swap_exact_limit() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let (token_x_program, token_y_program) = init_tokens(&sys);

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&invariant, &token_x_program, &token_y_program);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    let amount = U256::from(1000);

    mint(&token_x_program, REGULAR_USER_2, amount).assert_success();

    let amount_x = balance_of(&token_x_program, REGULAR_USER_2);
    assert_eq!(amount_x, amount);
    increase_allowance(&token_x_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    let swap_amount = TokenAmount::new(amount);

    swap_exact_limit(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        true,
        swap_amount,
        true,
    );
}
