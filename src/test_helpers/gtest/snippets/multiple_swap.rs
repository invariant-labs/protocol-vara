use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gstd::prelude::*;
use gtest::*;
use math::{
    fee_growth::FeeGrowth,
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
    token_amount::TokenAmount,
};
use sails_rtl::ActorId;

pub fn multiple_swap(x_to_y: bool) {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let mint_amount = U256::from(10u128.pow(10));

    let (token_x_program, token_y_program) = init_tokens(&sys);

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(1, 3),
        tick_spacing: 1,
    };

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    mint(&token_x_program, REGULAR_USER_1, U256::from(u128::MAX)).assert_success();
    mint(&token_y_program, REGULAR_USER_1, U256::from(u128::MAX)).assert_success();

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

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let upper_tick = 953;
    let lower_tick = -upper_tick;

    let amount = U256::from(100);
    let pool_data = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    let result = get_liquidity(
        TokenAmount(amount),
        TokenAmount(amount),
        lower_tick,
        upper_tick,
        pool_data.sqrt_price,
        true,
    )
    .unwrap();
    let liquidity_delta = result.l;
    let slippage_limit_lower = pool_data.sqrt_price;
    let slippage_limit_upper = pool_data.sqrt_price;

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

    if x_to_y {
        mint(&token_x_program, REGULAR_USER_2, amount).assert_success();

        assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), amount);
        increase_allowance(&token_x_program, REGULAR_USER_2, INVARIANT_ID, amount);
    } else {
        mint(&token_y_program, REGULAR_USER_2, amount).assert_success();
        assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), amount);
        increase_allowance(&token_y_program, REGULAR_USER_2, INVARIANT_ID, amount);
    }

    let swap_amount = TokenAmount::new(U256::from(10));
    for _ in 1..=10 {
        swap_exact_limit(
            &invariant,
            REGULAR_USER_2,
            pool_key,
            x_to_y,
            swap_amount,
            true,
        );
    }

    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    if x_to_y {
        assert_eq!(pool.current_tick_index, -821);
    } else {
        assert_eq!(pool.current_tick_index, 820);
    }
    assert_eq!(pool.fee_growth_global_x, FeeGrowth::new(0));
    assert_eq!(pool.fee_growth_global_y, FeeGrowth::new(0));
    if x_to_y {
        assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(U256::from(10)));
        assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(U256::from(0)));
    } else {
        assert_eq!(pool.fee_protocol_token_x, TokenAmount::new(U256::from(0)));
        assert_eq!(pool.fee_protocol_token_y, TokenAmount::new(U256::from(10)));
    }
    assert_eq!(pool.liquidity, liquidity_delta);
    if x_to_y {
        assert_eq!(pool.sqrt_price, SqrtPrice::new(959805958530842759275220u128));
    } else {
        assert_eq!(pool.sqrt_price, SqrtPrice::new(1041877257701839564633600u128));
    }

    let invariant_amount_x = balance_of(&token_x_program, INVARIANT_ID);
    let invariant_amount_y = balance_of(&token_y_program, INVARIANT_ID);
    if x_to_y {
        assert_eq!(invariant_amount_x, U256::from(200));
        assert_eq!(invariant_amount_y, U256::from(20));
    } else {
        assert_eq!(invariant_amount_x, U256::from(20));
        assert_eq!(invariant_amount_y, U256::from(200));
    }

    let user_amount_x = balance_of(&token_x_program, REGULAR_USER_2);
    let user_amount_y = balance_of(&token_y_program, REGULAR_USER_2);
    if x_to_y {
        assert_eq!(user_amount_x, U256::from(0));
        assert_eq!(user_amount_y, U256::from(80));
    } else {
        assert_eq!(user_amount_x, U256::from(80));
        assert_eq!(user_amount_y, U256::from(0));
    }
}
