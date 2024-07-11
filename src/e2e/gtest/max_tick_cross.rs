use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{
    get_tick_at_sqrt_price, liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice,
    token_amount::TokenAmount, MIN_SQRT_PRICE,
};
use sails_rtl::ActorId;

#[test]
fn max_tick_cross() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    let mint_amount = U256::from(u128::MAX);

    let (token_x_program, token_y_program) = init_tokens(&sys);

    init_basic_pool(&invariant, &token_x, &token_y);

    mint(&token_x_program, REGULAR_USER_1, mint_amount).assert_success();
    mint(&token_y_program, REGULAR_USER_1, mint_amount).assert_success();

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();

    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_1,
            TOKEN_X_ID,
            mint_amount,
            None::<&str>
        ),
        Some(TokenAmount(mint_amount))
    );

    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_1,
            TOKEN_Y_ID,
            mint_amount,
            None::<&str>
        ),
        Some(TokenAmount(mint_amount))
    );

    let liquidity = Liquidity::from_integer(10000000);

    let tick_spacing = 10;
    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), tick_spacing).unwrap();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    for i in (-2560..20).step_by(tick_spacing as usize) {
        let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

        let slippage_limit_lower = pool.sqrt_price;
        let slippage_limit_upper = pool.sqrt_price;

        create_position(
            &invariant,
            REGULAR_USER_1,
            pool_key,
            i,
            i + tick_spacing as i32,
            liquidity,
            slippage_limit_lower,
            slippage_limit_upper,
        )
        .assert_success();
    }

    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    assert_eq!(pool.liquidity, liquidity);

    let amount = U256::from(760_000);
    mint(&token_x_program, REGULAR_USER_2, amount).assert_success();
    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), amount);

    increase_allowance(&token_x_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    assert_eq!(
        deposit_single_token(&invariant, REGULAR_USER_2, TOKEN_X_ID, amount, None::<&str>),
        Some(TokenAmount(amount))
    );

    let pool_before = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);
    let slippage = SqrtPrice::new(MIN_SQRT_PRICE.into());

    let quote_result = quote(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        true,
        swap_amount,
        true,
        slippage,
    )
    .unwrap();

    let pool_after_quote = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();

    let crosses_after_quote =
        ((pool_after_quote.current_tick_index - pool_before.current_tick_index) / 10).abs();
    assert_eq!(crosses_after_quote, 0);
    assert_eq!(quote_result.ticks.len() - 1, 145);

    swap(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        true,
        swap_amount,
        true,
        slippage,
    )
    .assert_success();

    let pool_after = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();

    let crosses = ((pool_after.current_tick_index - pool_before.current_tick_index) / 10).abs();
    assert_eq!(crosses, 146);
    assert_eq!(
        pool_after.current_tick_index,
        get_tick_at_sqrt_price(quote_result.target_sqrt_price, 10).unwrap()
    );
}
