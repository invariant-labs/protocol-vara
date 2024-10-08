use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{
    fee_growth::FeeGrowth, liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice,
    token_amount::TokenAmount, MIN_SQRT_PRICE,
};
use sails_rs::ActorId;
pub fn init_cross_swap(invariant: &Program, token_x_program: &Program, token_y_program: &Program) {
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee = Percentage::from_scale(6, 3);
    let tick_spacing = 10;
    let fee_tier = FeeTier { fee, tick_spacing };
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let lower_tick = -20;

    let amount = U256::from(1000);

    mint(&token_x_program, REGULAR_USER_2, amount).assert_success();

    assert_eq!(balance_of(token_x_program, REGULAR_USER_2), amount);
    assert_eq!(balance_of(token_x_program, INVARIANT_ID), U256::from(500));
    assert_eq!(balance_of(token_y_program, INVARIANT_ID), U256::from(2499));

    increase_allowance(token_x_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    let pool_before = get_pool(invariant, token_x, token_y, fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);

    assert_eq!(
        deposit_single_token(
            &invariant,
            REGULAR_USER_2,
            TOKEN_X_ID,
            swap_amount.get(),
            None::<&str>
        ),
        Some(swap_amount)
    );

    let slippage = SqrtPrice::new(MIN_SQRT_PRICE.into());

    swap(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        true,
        swap_amount,
        true,
        slippage,
    )
    .assert_success();

    assert_eq!(
        withdraw_single_token(invariant, REGULAR_USER_2, TOKEN_X_ID, None, None::<&str>).unwrap(),
        TokenAmount::new(0.into())
    );

    assert!(
        withdraw_single_token(invariant, REGULAR_USER_2, TOKEN_Y_ID, None, None::<&str>).is_some()
    );

    let pool_after = get_pool(invariant, token_x, token_y, fee_tier).unwrap();

    let position_liquidity = Liquidity::from_integer(1000000);
    assert_eq!(
        pool_after.liquidity - position_liquidity,
        pool_before.liquidity
    );
    assert_eq!(pool_after.current_tick_index, lower_tick);
    assert_ne!(pool_after.sqrt_price, pool_before.sqrt_price);

    assert_eq!(balance_of(token_x_program, REGULAR_USER_2), U256::from(0));
    assert_eq!(balance_of(token_y_program, REGULAR_USER_2), U256::from(990));

    assert_eq!(balance_of(token_x_program, INVARIANT_ID), U256::from(1500));
    assert_eq!(balance_of(token_y_program, INVARIANT_ID), U256::from(1509));

    assert_eq!(
        pool_after.fee_growth_global_x,
        FeeGrowth::new(40000000000000000000000u128)
    );
    assert_eq!(pool_after.fee_growth_global_y, FeeGrowth::new(0));

    assert_eq!(
        pool_after.fee_protocol_token_x,
        TokenAmount::new(U256::from(2))
    );
    assert_eq!(
        pool_after.fee_protocol_token_y,
        TokenAmount::new(U256::from(0))
    );
}
