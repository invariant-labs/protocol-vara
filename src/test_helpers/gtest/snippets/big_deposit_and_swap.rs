use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gstd::{prelude::*, ActorId};
use gtest::*;
use io::InvariantAction;
use math::{
    percentage::Percentage, sqrt_price::*, token_amount::*, MAX_SQRT_PRICE, MIN_SQRT_PRICE,
};

pub fn big_deposit_and_swap(sys: &System, x_to_y: bool) {
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);
    let invariant = init_invariant(sys, Percentage::from_scale(1, 2));

    let (token_x_program, token_y_program) = init_tokens_with_mint(sys, (u128::MAX, u128::MAX));
    let approved_amount = 2u128.pow(75) - 1;

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        approved_amount,
    );
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        approved_amount,
    );

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 1,
    };
    invariant
        .send(ADMIN, InvariantAction::AddFeeTier(fee_tier))
        .assert_success();

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

    let lower_tick = if x_to_y {
        -(fee_tier.tick_spacing as i32)
    } else {
        0
    };
    let upper_tick = if x_to_y {
        0
    } else {
        fee_tier.tick_spacing as i32
    };
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let liquidity_delta = if x_to_y {
        get_liquidity_by_y(
            TokenAmount(approved_amount),
            lower_tick,
            upper_tick,
            pool.sqrt_price,
            true,
        )
        .unwrap()
        .l
    } else {
        get_liquidity_by_x(
            TokenAmount(approved_amount),
            lower_tick,
            upper_tick,
            pool.sqrt_price,
            true,
        )
        .unwrap()
        .l
    };

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let slippage_limit_lower = pool.sqrt_price;
    let slippage_limit_upper = pool.sqrt_price;
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

    let amount_x = balance_of(&token_x_program, REGULAR_USER_1);
    let amount_y = balance_of(&token_y_program, REGULAR_USER_1);
    if x_to_y {
        assert_eq!(amount_x, 340282366920938463463374607431768211455);
        assert_eq!(amount_y, 340282366920938425684442744474606501888);
    } else {
        assert_eq!(amount_x, 340282366920938425684442744474606501888);
        assert_eq!(amount_y, 340282366920938463463374607431768211455);
    }

    let sqrt_price_limit = if x_to_y {
        SqrtPrice::new(MIN_SQRT_PRICE)
    } else {
        SqrtPrice::new(MAX_SQRT_PRICE)
    };

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::Swap {
                pool_key,
                x_to_y,
                amount: TokenAmount(approved_amount),
                by_amount_in: true,
                sqrt_price_limit,
            },
        )
        .assert_success();

    let amount_x = balance_of(&token_x_program, REGULAR_USER_1);
    let amount_y = balance_of(&token_y_program, REGULAR_USER_1);
    if x_to_y {
        assert_eq!(amount_x, 340282366920938425684442744474606501888);
        assert_ne!(amount_y, 0);
    } else {
        assert_ne!(amount_x, 0);
        assert_eq!(amount_y, 340282366920938425684442744474606501888);
    }
}
