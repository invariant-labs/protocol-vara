use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gstd::prelude::*;
use gtest::*;
use math::{
    fee_growth::FeeGrowth, liquidity::Liquidity, percentage::Percentage, token_amount::TokenAmount,
};
use sails_rtl::ActorId;

pub fn init_basic_position(
    sys: &System,
    invariant: &Program<'_>,
    token_x_program: &Program<'_>,
    token_y_program: &Program<'_>,
) {
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 10,
    };

    let mint_amount = U256::from(10u128.pow(10));
    mint(&token_x_program, REGULAR_USER_1, mint_amount).assert_success();
    mint(&token_y_program, REGULAR_USER_1, mint_amount).assert_success();

    increase_allowance(&token_x_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_1, INVARIANT_ID, mint_amount)
        .assert_success();

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
    let lower_tick = -20;
    let upper_tick = 10;
    let liquidity = Liquidity::from_integer(1000000);

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let slippage_limit_lower = pool_before.sqrt_price;
    let slippage_limit_upper = pool_before.sqrt_price;

    let res = create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick,
        upper_tick,
        liquidity,
        slippage_limit_lower,
        slippage_limit_upper,
    );

    let events = res.emitted_events();
    events[0]
        .assert_to(EVENT_ADDRESS)
        .assert_with_payload(PositionCreatedEvent {
            address: REGULAR_USER_1.into(),
            pool_key,
            liquidity_delta: liquidity,
            timestamp: sys.block_timestamp(),
            lower_tick,
            upper_tick,
            current_sqrt_price: pool_before.sqrt_price,
        });
    events[1]
        .assert_to(REGULAR_USER_1)
        .assert_with_payload(Position {
            pool_key,
            liquidity,
            lower_tick_index: lower_tick,
            upper_tick_index: upper_tick,
            fee_growth_inside_x: FeeGrowth::new(U128::from(0)),
            fee_growth_inside_y: FeeGrowth::new(U128::from(0)),
            last_block_number: sys.block_height() as u64,
            tokens_owed_x: TokenAmount::new(U256::from(0)),
            tokens_owed_y: TokenAmount::new(U256::from(0)),
        });

    let pool_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    withdraw_single_token(&invariant, REGULAR_USER_1, TOKEN_X_ID, None, None::<&str>).unwrap();
    withdraw_single_token(&invariant, REGULAR_USER_1, TOKEN_Y_ID, None, None::<&str>).unwrap();

    assert_eq!(pool_after.liquidity, liquidity);
}
