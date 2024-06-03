use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gstd::{prelude::*, ActorId};
use gtest::*;
use io::*;
use math::{
    fee_growth::FeeGrowth,
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
    token_amount::TokenAmount,
};

pub fn multiple_swap(x_to_y: bool) {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let mint_amount = 10u128.pow(10);

    let (token_x_program, token_y_program) = init_tokens(&sys);

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(1, 3),
        tick_spacing: 1,
    };

    invariant
        .send(ADMIN, InvariantAction::AddFeeTier(fee_tier))
        .assert_success();

    mint(&token_x_program, REGULAR_USER_1, u128::MAX).assert_success();
    mint(&token_y_program, REGULAR_USER_1, u128::MAX).assert_success();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
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

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let upper_tick = 953;
    let lower_tick = -upper_tick;

    let amount = 100;
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

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePosition {
                pool_key,
                lower_tick,
                upper_tick,
                liquidity_delta,
                slippage_limit_lower,
                slippage_limit_upper,
            },
        )
        .assert_success();

    if x_to_y {
        mint(&token_x_program, REGULAR_USER_2, amount).assert_success();

        assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), amount);
        increase_allowance(&token_x_program, REGULAR_USER_2, INVARIANT_ID, amount);
    } else {
        mint(&token_y_program, REGULAR_USER_2, amount).assert_success();
        assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), amount);
        increase_allowance(&token_y_program, REGULAR_USER_2, INVARIANT_ID, amount);
    }

    let swap_amount = TokenAmount(10);
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
        assert_eq!(pool.fee_protocol_token_x, TokenAmount(10));
        assert_eq!(pool.fee_protocol_token_y, TokenAmount(0));
    } else {
        assert_eq!(pool.fee_protocol_token_x, TokenAmount(0));
        assert_eq!(pool.fee_protocol_token_y, TokenAmount(10));
    }
    assert_eq!(pool.liquidity, liquidity_delta);
    if x_to_y {
        assert_eq!(pool.sqrt_price, SqrtPrice::new(959805958620596146276151));
    } else {
        assert_eq!(pool.sqrt_price, SqrtPrice::new(1041877257604411525269920));
    }

    let invariant_amount_x = balance_of(&token_x_program, INVARIANT_ID);
    let invariant_amount_y = balance_of(&token_y_program, INVARIANT_ID);
    if x_to_y {
        assert_eq!(invariant_amount_x, 200);
        assert_eq!(invariant_amount_y, 20);
    } else {
        assert_eq!(invariant_amount_x, 20);
        assert_eq!(invariant_amount_y, 200);
    }

    let user_amount_x = balance_of(&token_x_program, REGULAR_USER_2);
    let user_amount_y = balance_of(&token_y_program, REGULAR_USER_2);
    if x_to_y {
        assert_eq!(user_amount_x, 0);
        assert_eq!(user_amount_y, 80);
    } else {
        assert_eq!(user_amount_x, 80);
        assert_eq!(user_amount_y, 0);
    }
}
