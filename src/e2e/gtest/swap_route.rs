use crate::test_helpers::gtest::*;
use contracts::{FeeTier, InvariantError, PoolKey};
use decimal::*;
use gtest::System;
use io::SwapHop;
use math::types::{
    liquidity::Liquidity, percentage::Percentage, sqrt_price::calculate_sqrt_price,
    token_amount::TokenAmount,
};
use sails_rs::ActorId;
#[test]
fn swap_route_test() {
    let sys = System::new();
    sys.init_logger();

    let (invariant, token_x_program, token_y_program, token_z_program) =
        init_invariant_and_3_tokens(&sys);
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);
    let token_z = ActorId::from(TOKEN_Z_ID);

    mint(&token_x_program, REGULAR_USER_2, u64::MAX.into()).assert_success();
    mint(&token_y_program, REGULAR_USER_2, u64::MAX.into()).assert_success();
    mint(&token_z_program, REGULAR_USER_2, u64::MAX.into()).assert_success();

    increase_allowance(
        &token_x_program,
        REGULAR_USER_2,
        INVARIANT_ID,
        u64::MAX.into(),
    )
    .assert_success();
    increase_allowance(
        &token_y_program,
        REGULAR_USER_2,
        INVARIANT_ID,
        u64::MAX.into(),
    )
    .assert_success();
    increase_allowance(
        &token_z_program,
        REGULAR_USER_2,
        INVARIANT_ID,
        u64::MAX.into(),
    )
    .assert_success();

    deposit_single_token(
        &invariant,
        REGULAR_USER_2,
        token_x,
        u64::MAX.into(),
        None::<&str>,
    );
    deposit_single_token(
        &invariant,
        REGULAR_USER_2,
        token_y,
        u64::MAX.into(),
        None::<&str>,
    );
    deposit_single_token(
        &invariant,
        REGULAR_USER_2,
        token_z,
        u64::MAX.into(),
        None::<&str>,
    );

    let amount = 1000;
    mint(&token_x_program, REGULAR_USER_1, amount.into()).assert_success();

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        amount.into(),
    )
    .assert_success();

    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        token_x,
        amount.into(),
        None::<&str>,
    );

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 1).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    create_pool(
        &invariant,
        REGULAR_USER_2,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_success();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    create_pool(
        &invariant,
        REGULAR_USER_2,
        token_y,
        token_z,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_success();

    let pool_key_1 = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let pool_key_2 = PoolKey::new(token_y, token_z, fee_tier).unwrap();

    let liquidity_delta = Liquidity::new((2u128.pow(63) - 1).into());

    let pool_1 = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    let slippage_limit_lower = pool_1.sqrt_price;
    let slippage_limit_upper = pool_1.sqrt_price;
    create_position(
        &invariant,
        REGULAR_USER_2,
        pool_key_1,
        -1,
        1,
        liquidity_delta,
        slippage_limit_lower,
        slippage_limit_upper,
    )
    .assert_success();

    let pool_2 = get_pool(&invariant, token_y, token_z, fee_tier).unwrap();
    let slippage_limit_lower = pool_2.sqrt_price;
    let slippage_limit_upper = pool_2.sqrt_price;
    create_position(
        &invariant,
        REGULAR_USER_2,
        pool_key_2,
        -1,
        1,
        liquidity_delta,
        slippage_limit_lower,
        slippage_limit_upper,
    )
    .assert_success();

    let amount_in = TokenAmount(1000.into());
    let slippage = Percentage::new(0);
    let swaps = vec![
        SwapHop {
            pool_key: pool_key_1,
            x_to_y: true,
        },
        SwapHop {
            pool_key: pool_key_2,
            x_to_y: true,
        },
    ];

    let expected_token_amount = quote_route(&invariant, amount_in, swaps.clone()).unwrap();

    swap_route(
        &invariant,
        REGULAR_USER_1,
        amount_in,
        expected_token_amount,
        slippage,
        swaps.clone(),
    )
    .assert_success();

    withdraw_single_token(
        &invariant,
        REGULAR_USER_1,
        token_x,
        None,
        InvariantError::NoBalanceForTheToken.into(),
    );
    withdraw_single_token(
        &invariant,
        REGULAR_USER_1,
        token_y,
        None,
        InvariantError::NoBalanceForTheToken.into(),
    );

    withdraw_single_token(&invariant, REGULAR_USER_1, token_z, None, None::<&str>);

    withdraw_single_token(&invariant, REGULAR_USER_2, token_x, None, None::<&str>);
    withdraw_single_token(&invariant, REGULAR_USER_2, token_y, None, None::<&str>);
    withdraw_single_token(&invariant, REGULAR_USER_2, token_z, None, None::<&str>);

    let user_1_amount_x = balance_of(&token_x_program, REGULAR_USER_1);
    let user_1_amount_y = balance_of(&token_y_program, REGULAR_USER_1);
    let user_1_amount_z = balance_of(&token_z_program, REGULAR_USER_1);

    assert_eq!(user_1_amount_x, 0.into());
    assert_eq!(user_1_amount_y, 0.into());
    assert_eq!(user_1_amount_z, 986.into());

    let pool_1_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    assert_eq!(pool_1_after.fee_protocol_token_x, TokenAmount(1.into()));
    assert_eq!(pool_1_after.fee_protocol_token_y, TokenAmount(0.into()));

    let pool_2_after = get_pool(&invariant, token_y, token_z, fee_tier).unwrap();
    assert_eq!(pool_2_after.fee_protocol_token_x, TokenAmount(1.into()));
    assert_eq!(pool_2_after.fee_protocol_token_y, TokenAmount(0.into()));

    let user_2_amount_x_before = balance_of(&token_x_program, REGULAR_USER_2);
    let user_2_amount_y_before = balance_of(&token_y_program, REGULAR_USER_2);
    let user_2_amount_z_before = balance_of(&token_z_program, REGULAR_USER_2);

    claim_fee(&invariant, REGULAR_USER_2, 0, None::<&str>);
    claim_fee(&invariant, REGULAR_USER_2, 1, None::<&str>);

    withdraw_single_token(&invariant, REGULAR_USER_2, token_x, None, None::<&str>);
    withdraw_single_token(&invariant, REGULAR_USER_2, token_y, None, None::<&str>);
    withdraw_single_token(
        &invariant,
        REGULAR_USER_2,
        token_z,
        None,
        InvariantError::NoBalanceForTheToken.into(),
    );

    let user_2_amount_x_after = balance_of(&token_x_program, REGULAR_USER_2);
    let user_2_amount_y_after = balance_of(&token_y_program, REGULAR_USER_2);
    let user_2_amount_z_after = balance_of(&token_z_program, REGULAR_USER_2);

    assert_eq!(user_2_amount_x_after - user_2_amount_x_before, 4.into());
    assert_eq!(user_2_amount_y_after - user_2_amount_y_before, 4.into());
    assert_eq!(user_2_amount_z_after - user_2_amount_z_before, 0.into());
}
