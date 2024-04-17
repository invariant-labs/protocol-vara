use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gstd::ActorId;
use gtest::*;
use io::*;
use math::{liquidity::Liquidity, percentage::Percentage};

pub fn init_cross_position(
    invariant: &Program,
    token_x_program: &Program,
    token_y_program: &Program,
) {
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 10,
    };

    let mint_amount = 10u128.pow(10);
    increase_allowance(token_x_program, REGULAR_USER_1, INVARIANT_ID, mint_amount).assert_success();
    increase_allowance(token_y_program, REGULAR_USER_1, INVARIANT_ID, mint_amount).assert_success();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let lower_tick = -40;
    let upper_tick = -10;
    let liquidity = Liquidity::from_integer(1000000);

    let pool_before = get_pool(invariant, token_x, token_y, fee_tier).unwrap();

    let slippage_limit_lower = pool_before.sqrt_price;
    let slippage_limit_upper = pool_before.sqrt_price;

    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePosition {
                pool_key,
                lower_tick,
                upper_tick,
                liquidity_delta: liquidity,
                slippage_limit_lower,
                slippage_limit_upper,
            },
        )
        .assert_success();

    let pool_after = get_pool(invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool_after.liquidity, pool_before.liquidity);
}
