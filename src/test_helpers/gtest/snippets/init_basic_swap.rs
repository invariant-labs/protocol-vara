use crate::{send_request, test_helpers::gtest::*};
use contracts::*;
use decimal::*;
use gtest::*;
use io::*;
use math::{
    fee_growth::FeeGrowth, percentage::Percentage, sqrt_price::SqrtPrice,
    token_amount::TokenAmount, MIN_SQRT_PRICE,
};
use sails_rs::prelude::*;

pub fn init_basic_swap(
    invariant: &Program<'_>,
    token_x_program: &Program<'_>,
    token_y_program: &Program<'_>,
) {
    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();
    let fee = Percentage::from_scale(6, 3);
    let tick_spacing = 10;
    let fee_tier = FeeTier { fee, tick_spacing };
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let lower_tick = -20;

    let amount = U256::from(1000);
    mint(&token_x_program, REGULAR_USER_2, amount).assert_success();
    increase_allowance(&token_x_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), amount);

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), U256::from(500));
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), U256::from(1000));

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);
    let slippage = SqrtPrice::new(MIN_SQRT_PRICE.into());

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

    let res = swap(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        true,
        swap_amount,
        true,
        slippage,
    );

    let pool_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let events = res.emitted_events();
    let swap_event = events[0]
        .assert_to(EVENT_ADDRESS)
        .decoded_event::<SwapEvent>()
        .unwrap();

    swap_events_are_identical_no_timestamp(
        &swap_event,
        &SwapEvent {
            timestamp: 0,
            address: REGULAR_USER_2.into(),
            pool_key,
            amount_in: TokenAmount::new(U256::from(1000)),
            amount_out: TokenAmount::new(U256::from(993)),
            fee: TokenAmount::new(U256::from(6)),
            start_sqrt_price: SqrtPrice::new(1000000000000000000000000u128),
            target_sqrt_price: SqrtPrice::new(999006987054867461743028u128),
            x_to_y: true,
        },
    );

    events[1]
        .assert_to(REGULAR_USER_2)
        .assert_with_payload(CalculateSwapResult {
            amount_in: TokenAmount::new(U256::from(1000)),
            amount_out: TokenAmount::new(U256::from(993)),
            fee: TokenAmount::new(U256::from(6)),
            start_sqrt_price: SqrtPrice::new(1000000000000000000000000u128),
            target_sqrt_price: SqrtPrice::new(999006987054867461743028u128),
            pool: pool_after.clone(),
            ticks: vec![],
        });

    assert!(
        withdraw_single_token(&invariant, REGULAR_USER_2, TOKEN_Y_ID, None, None::<&str>).is_some()
    );

    assert_eq!(pool_after.liquidity, pool_before.liquidity);
    assert_eq!(pool_after.current_tick_index, lower_tick);
    assert_ne!(pool_after.sqrt_price, pool_before.sqrt_price);

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), U256::from(0));
    assert_eq!(
        balance_of(&token_y_program, REGULAR_USER_2),
        U256::from(993)
    );

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), U256::from(1500));
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), U256::from(7));

    assert_eq!(
        pool_after.fee_growth_global_x,
        FeeGrowth::new(50000000000000000000000u128)
    );
    assert_eq!(pool_after.fee_growth_global_y, FeeGrowth::new(0));

    assert_eq!(
        pool_after.fee_protocol_token_x,
        TokenAmount::new(U256::from(1))
    );
    assert_eq!(
        pool_after.fee_protocol_token_y,
        TokenAmount::new(U256::from(0))
    );
}
