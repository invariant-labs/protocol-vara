use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gstd::prelude::*;
use gtest::*;
use math::{
    percentage::Percentage, sqrt_price::*, token_amount::*, MAX_SQRT_PRICE, MIN_SQRT_PRICE,
};
use sails_rtl::ActorId;

pub fn big_deposit_and_swap(sys: &System, x_to_y: bool) {
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);
    let invariant = init_invariant(sys, Percentage::from_scale(1, 2));

    let (token_x_program, token_y_program) =
        init_tokens_with_mint_user_1(sys, (U256::MAX.into(), U256::MAX.into()));
    let approved_amount =
        U256::from_dec_str("102844034832575377634685573909834406561420991602098741459288064") //2^206
            .unwrap();

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

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        approved_amount,
        token_y,
        approved_amount,
        None::<&str>,
    );

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

    let amount_x = balance_of(&token_x_program, REGULAR_USER_1);
    let amount_y = balance_of(&token_y_program, REGULAR_USER_1);
    if x_to_y {
        assert_eq!(
            amount_x,
            U256::from_dec_str(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935"
            )
            .unwrap()
        );
        assert_eq!(
            amount_y,
            U256::from_dec_str(
                "115792089237316092579536152433310273167696074831234002618465981909171670351871"
            )
            .unwrap()
        );
    } else {
        assert_eq!(
            amount_y,
            U256::from_dec_str(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935"
            )
            .unwrap()
        );
        assert_eq!(
            amount_x,
            U256::from_dec_str(
                "115792089237316092579536152433310273167696074831234002618465981909171670351871"
            )
            .unwrap()
        );
    }

    let sqrt_price_limit = if x_to_y {
        SqrtPrice::new(MIN_SQRT_PRICE.into())
    } else {
        SqrtPrice::new(MAX_SQRT_PRICE.into())
    };

    let (swapped_token, returned_token) = if x_to_y {
        increase_allowance(
            &token_x_program,
            REGULAR_USER_1,
            INVARIANT_ID,
            approved_amount,
        )
        .assert_success();

        (token_x, token_y)
    } else {
        increase_allowance(
            &token_y_program,
            REGULAR_USER_1,
            INVARIANT_ID,
            approved_amount,
        )
        .assert_success();

        (token_y, token_x)
    };

    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        swapped_token,
        approved_amount,
        None::<&str>,
    )
    .unwrap();

    swap(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        x_to_y,
        TokenAmount(approved_amount),
        true,
        sqrt_price_limit,
    )
    .assert_success();

    withdraw_single_token(
        &invariant,
        REGULAR_USER_1,
        returned_token,
        None,
        None::<&str>,
    )
    .unwrap();

    let amount_x = balance_of(&token_x_program, REGULAR_USER_1);
    let amount_y = balance_of(&token_y_program, REGULAR_USER_1);
    if x_to_y {
        assert_eq!(
            amount_x,
            U256::from_dec_str(
                "115792089237316092579536152433310273167696074831234002618465981909171670351871"
            )
            .unwrap()
        );
        assert_ne!(amount_y, U256::from(0));
    } else {
        assert_ne!(amount_x, U256::from(0));
        assert_eq!(
            amount_y,
            U256::from_dec_str(
                "115792089237316092579536152433310273167696074831234002618465981909171670351871"
            )
            .unwrap()
        );
    }
}
