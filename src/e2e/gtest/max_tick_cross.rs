use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use io::*;
use math::{
    get_tick_at_sqrt_price, liquidity::Liquidity, percentage::Percentage,
    sqrt_price::calculate_sqrt_price, sqrt_price::SqrtPrice, token_amount::TokenAmount,
    MAX_SQRT_PRICE, MIN_SQRT_PRICE,
};

use sails_rs::ActorId;

#[test]
#[ignore]
fn max_tick_cross() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    let mint_amount = U256::from(U256::MAX);

    let (token_x_program, token_y_program) = init_tokens(&sys);

    let tick_spacing = 1;
    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), tick_spacing).unwrap();
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_success();

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

    let liquidity = Liquidity::from_integer(1000000000000000u128);

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    for i in ((-863 * 256)..(32 * 256)).step_by(256) {
        let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

        let slippage_limit_lower = pool.sqrt_price;
        let slippage_limit_upper = pool.sqrt_price;

        create_position(
            &invariant,
            REGULAR_USER_1,
            pool_key,
            i,
            i + 256 as i32,
            liquidity,
            slippage_limit_lower,
            slippage_limit_upper,
        )
        .assert_success();
    }

    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    assert_eq!(pool.liquidity, liquidity);
    let swap_amount = TokenAmount(U256::from(489951846302626_u128));
    let slippage = SqrtPrice::new(MAX_SQRT_PRICE.into());

    swap(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        false,
        swap_amount,
        true,
        slippage,
    )
    .assert_success();

    let pool_before = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();
    let swap_amount = TokenAmount::new(U256::from(63058587794151558883_u128));
    let slippage = SqrtPrice::new(MIN_SQRT_PRICE.into());

    let quote_result = quote(
        &invariant,
        REGULAR_USER_2,
        pool_key,
        true,
        swap_amount,
        true,
        slippage,
    )
    .unwrap();

    let pool_after_quote = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();

    assert_eq!(pool_after_quote, pool_before);
    assert_eq!(quote_result.ticks.len(), 893);

    let res = swap(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        true,
        swap_amount,
        true,
        slippage,
    );
    res.assert_success();

    let pool_after = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();
    let events = res.emitted_events();
    assert_eq!(events.len(), 3);
    let (cross_tick_event, swap_event, swap_return) = (
        events[0]
            .assert_to(EVENT_ADDRESS)
            .decoded_event::<CrossTickEvent>()
            .unwrap(),
        events[1]
            .assert_to(EVENT_ADDRESS)
            .decoded_event::<SwapEvent>()
            .unwrap(),
        events[2]
            .assert_to(REGULAR_USER_1)
            .decoded_event::<CalculateSwapResult>()
            .unwrap(),
    );
    assert_eq!(cross_tick_event.indexes.len(), 893);
    assert_eq!(swap_return.ticks.len(), 893);

    swap_events_are_identical_no_timestamp(
        &swap_event,
        &SwapEvent {
            timestamp: 0,
            address: REGULAR_USER_1.into(),
            pool_key,
            amount_in: TokenAmount(63058587794151558883u128.into()),
            amount_out: TokenAmount(1487013034191998_u128.into()),
            fee: TokenAmount(376123656973987223_u128.into()),
            start_sqrt_price: SqrtPrice(1487028987445999000000000),
            target_sqrt_price: SqrtPrice(15953254000000000001),
            x_to_y: true,
        },
    );

    assert_eq!(
        pool_after.current_tick_index,
        get_tick_at_sqrt_price(pool_after.sqrt_price, pool_key.fee_tier.tick_spacing).unwrap()
    );
}
