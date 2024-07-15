use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{liquidity::Liquidity, percentage::Percentage, sqrt_price::calculate_sqrt_price};
use sails_rtl::{prelude::*, ActorId};

#[test]
fn test_get_liquidity_ticks() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (U256::from(500), U256::from(500)));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 10).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);
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
    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        U256::from(500),
    )
    .assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        U256::from(500),
    )
    .assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        U256::from(500),
        token_y,
        U256::from(500),
        None::<&str>,
    )
    .unwrap();

    let _res = create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -10,
        10,
        Liquidity::new(U256::from(10)),
        pool.sqrt_price,
        pool.sqrt_price,
    );

    let liquidity_ticks = get_liquidity_ticks(&invariant, pool_key);
    
    assert_eq!(
        liquidity_ticks,
        vec![
            LiquidityTick {
                index: -10,
                liquidity_change: Liquidity(U256::from(10)),
                sign: true
            },
            LiquidityTick {
                index: 10,
                liquidity_change: Liquidity(U256::from(10)),
                sign: false
            }
        ]
    )
}
