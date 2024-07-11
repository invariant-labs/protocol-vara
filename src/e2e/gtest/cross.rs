use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{fee_growth::FeeGrowth, liquidity::Liquidity, percentage::Percentage};
use sails_rtl::ActorId;
#[test]
fn test_cross() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);
    let (token_x_program, token_y_program) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);
    init_cross_position(&invariant, &token_x_program, &token_y_program);
    init_cross_swap(&invariant, &token_x_program, &token_y_program);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    let upper_tick_index = 10;
    let middle_tick_index = -10;
    let lower_tick_index = -20;

    let upper_tick = get_tick(&invariant, pool_key, upper_tick_index).unwrap();
    let middle_tick = get_tick(&invariant, pool_key, middle_tick_index).unwrap();
    let lower_tick = get_tick(&invariant, pool_key, lower_tick_index).unwrap();

    assert_eq!(
        upper_tick.liquidity_change,
        Liquidity::from_integer(1000000)
    );
    assert_eq!(
        middle_tick.liquidity_change,
        Liquidity::from_integer(1000000)
    );
    assert_eq!(
        lower_tick.liquidity_change,
        Liquidity::from_integer(1000000)
    );

    assert_eq!(upper_tick.fee_growth_outside_x, FeeGrowth::new(0));
    assert_eq!(
        middle_tick.fee_growth_outside_x,
        FeeGrowth::new(30000000000000000000000u128)
    );
    assert_eq!(lower_tick.fee_growth_outside_x, FeeGrowth::new(0));
}
