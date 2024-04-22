use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use fungible_token_io::*;
use gstd::{prelude::*, ActorId};
use gtest::*;
use io::*;
use math::{
    fee_growth::FeeGrowth, percentage::Percentage, sqrt_price::SqrtPrice,
    token_amount::TokenAmount, MIN_SQRT_PRICE,
};

pub fn init_basic_swap(
    sys: &System,
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

    let amount = 1000;
    assert!(!token_x_program
        .send(REGULAR_USER_2, FTAction::Mint(amount))
        .main_failed());

    assert!(!token_x_program
        .send(
            REGULAR_USER_2,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount
            }
        )
        .main_failed());

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), amount);

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), 500);
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), 1000);

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);
    let slippage = SqrtPrice::new(MIN_SQRT_PRICE);

    let res = invariant.send(
        REGULAR_USER_2,
        InvariantAction::Swap {
            pool_key,
            x_to_y: true,
            amount: swap_amount,
            by_amount_in: true,
            sqrt_price_limit: slippage,
        },
    );

    let pool_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert!(res.events_eq(vec![
        TestEvent::invariant_response(
            REGULAR_USER_2,
            InvariantEvent::SwapEvent {
                timestamp: sys.block_timestamp(),
                address: REGULAR_USER_2.into(),
                pool: pool_key,
                amount_in: TokenAmount(1000),
                amount_out: TokenAmount(993),
                fee: TokenAmount(6),
                start_sqrt_price: SqrtPrice(1000000000000000000000000),
                target_sqrt_price: SqrtPrice(999006987054867461743028),
                x_to_y: true,
            }
        ),
        TestEvent::invariant_response(
            REGULAR_USER_2,
            InvariantEvent::SwapReturn(CalculateSwapResult {
                amount_in: TokenAmount(1000),
                amount_out: TokenAmount(993),
                fee: TokenAmount(6),
                start_sqrt_price: SqrtPrice(1000000000000000000000000),
                target_sqrt_price: SqrtPrice(999006987054867461743028),
                pool: pool_after.clone(),
                ticks: vec![],
            })
        )
    ]));

    assert_eq!(pool_after.liquidity, pool_before.liquidity);
    assert_eq!(pool_after.current_tick_index, lower_tick);
    assert_ne!(pool_after.sqrt_price, pool_before.sqrt_price);

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), 0);
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), 993);

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), 1500);
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), 7);

    assert_eq!(
        pool_after.fee_growth_global_x,
        FeeGrowth::new(50000000000000000000000)
    );
    assert_eq!(pool_after.fee_growth_global_y, FeeGrowth::new(0));

    assert_eq!(pool_after.fee_protocol_token_x, TokenAmount::new(1));
    assert_eq!(pool_after.fee_protocol_token_y, TokenAmount::new(0));
}