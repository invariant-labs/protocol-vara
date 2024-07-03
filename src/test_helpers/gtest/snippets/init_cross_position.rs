use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{liquidity::Liquidity, percentage::Percentage, token_amount::TokenAmount};
use sails_rtl::ActorId;

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

    let mint_amount = U256::from(10u128.pow(10));
    mint(token_x_program, REGULAR_USER_1, mint_amount).assert_success();
    mint(token_y_program, REGULAR_USER_1, mint_amount).assert_success();
    increase_allowance(token_x_program, REGULAR_USER_1, INVARIANT_ID, mint_amount).assert_success();
    increase_allowance(token_y_program, REGULAR_USER_1, INVARIANT_ID, mint_amount).assert_success();

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

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let lower_tick = -40;
    let upper_tick = -10;
    let liquidity = Liquidity::from_integer(1000000);

    let pool_before = get_pool(invariant, token_x, token_y, fee_tier).unwrap();

    let slippage_limit_lower = pool_before.sqrt_price;
    let slippage_limit_upper = pool_before.sqrt_price;

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick,
        upper_tick,
        liquidity,
        slippage_limit_lower,
        slippage_limit_upper,
    )
    .assert_success();

    let pool_after = get_pool(invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool_after.liquidity, pool_before.liquidity);

    assert!(
        withdraw_single_token(invariant, REGULAR_USER_1, TOKEN_X_ID, None, None::<&str>).is_some()
    );
    assert!(
        withdraw_single_token(invariant, REGULAR_USER_1, TOKEN_Y_ID, None, None::<&str>).is_some()
    );
}
