use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use fungible_token_io::*;
use gstd::{prelude::*, ActorId};
use gtest::*;
use io::*;
use math::{
    liquidity::Liquidity, percentage::Percentage, sqrt_price::calculate_sqrt_price,
    token_amount::TokenAmount, fee_growth::FeeGrowth
};

pub fn init_slippage_pool_with_liquidity(
    sys: &System,
    invariant: &Program<'_>,
    token_0_program: &Program<'_>,
    token_1_program: &Program<'_>,
) -> PoolKey {
    let token_0 = ActorId::from(TOKEN_X_ID);
    let token_1 = ActorId::from(TOKEN_Y_ID);
    
    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 10,
    };

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));
    
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0,
            token_1,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    let mint_amount = 10u128.pow(10);
    assert!(!token_0_program
        .send(REGULAR_USER_1, FTAction::Mint(mint_amount))
        .main_failed());
    assert!(!token_1_program
        .send(REGULAR_USER_1, FTAction::Mint(mint_amount))
        .main_failed());

    assert!(!token_0_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: mint_amount
            }
        )
        .main_failed());
    assert!(!token_1_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: mint_amount
            }
        )
        .main_failed());

    let pool_key = PoolKey::new(token_0, token_1, fee_tier).unwrap();
    let lower_tick = -1000;
    let upper_tick = 1000;
    let liquidity = Liquidity::from_integer(10u128.pow(10));

    let pool_before = get_pool(&invariant, token_0, token_1, fee_tier).unwrap();

    let slippage_limit_lower = pool_before.sqrt_price;
    let slippage_limit_upper = pool_before.sqrt_price;

    let res = invariant.send(REGULAR_USER_1, InvariantAction::CreatePosition {
        pool_key,
        lower_tick,
        upper_tick,
        liquidity_delta: liquidity,
        slippage_limit_lower,
        slippage_limit_upper,
    });

    assert!(res.events_eq(vec![
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedEvent {
                address: REGULAR_USER_1.into(),
                pool_key,
                liquidity_delta: liquidity,
                block_timestamp: sys.block_timestamp(),
                lower_tick,
                upper_tick,
                current_sqrt_price: init_sqrt_price,
            }
        ),
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedReturn(Position {
                pool_key,
                liquidity,
                lower_tick_index: lower_tick,
                upper_tick_index: upper_tick,
                fee_growth_inside_x: FeeGrowth::new(0),
                fee_growth_inside_y: FeeGrowth::new(0),
                last_block_number: sys.block_height() as u64,
                tokens_owed_x: TokenAmount(0),
                tokens_owed_y: TokenAmount(0)
            })
        )
    ]));

    let pool_after = get_pool(&invariant, token_0, token_1, fee_tier).unwrap();

    assert_eq!(pool_after.liquidity, liquidity);

    pool_key
}