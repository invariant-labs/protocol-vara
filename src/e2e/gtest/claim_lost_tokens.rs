use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use fungible_token_io::*;
use gstd::*;
use gtest::*;
use io::*;
use math::{
    fee_growth::FeeGrowth, percentage::Percentage, sqrt_price::SqrtPrice,
    token_amount::TokenAmount, MAX_SQRT_PRICE, MIN_SQRT_PRICE,
};

#[test]
fn test_swap_transfer_fail_token_x() {
    let sys = System::new();
    sys.init_logger();
    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (token_x_program, token_y_program) = init_tokens(&sys);
    let mut invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);
    let fee = Percentage::from_scale(6, 3);
    let tick_spacing = 10;
    let fee_tier = FeeTier { fee, tick_spacing };
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    let amount = 500;
    assert!(!token_y_program
        .send(REGULAR_USER_2, FTAction::Mint(amount))
        .main_failed());

    increase_allowance(&token_y_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), 500);

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), 500);
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), 1000);

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);
    let slippage = SqrtPrice::new(MAX_SQRT_PRICE);

    token_x_program
        .send(REGULAR_USER_2, FTAction::FailNextTransfer)
        .assert_success();

    let _res = invariant.send_and_assert_error(
        REGULAR_USER_2,
        InvariantAction::Swap {
            pool_key,
            x_to_y: false,
            amount: swap_amount,
            by_amount_in: true,
            sqrt_price_limit: slippage,
        },
        InvariantError::RecoverableTransferError,
    );

    let pool_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool_before, pool_after);

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), 0);
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), 0);

    assert_eq!(
        vec![(ActorId::from(TOKEN_Y_ID), swap_amount)],
        get_user_balances(&invariant, REGULAR_USER_2)
    );

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), 500);
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), 1000 + 500);

    assert_eq!(pool_after.fee_growth_global_x, FeeGrowth::new(0));
    assert_eq!(pool_after.fee_growth_global_y, FeeGrowth::new(0));

    assert_eq!(pool_after.fee_protocol_token_x, TokenAmount::new(0));
    assert_eq!(pool_after.fee_protocol_token_y, TokenAmount::new(0));

    invariant
        .send(
            REGULAR_USER_2,
            InvariantAction::ClaimLostTokens {
                token: TOKEN_Y_ID.into(),
            },
        )
        .assert_success();

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), 0);
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), 500);

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), 500);
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), 1000);
}

#[test]
fn test_claim_lost_tokens_after_swap_token_y() {
    let sys = System::new();
    sys.init_logger();
    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (token_x_program, token_y_program) = init_tokens(&sys);
    let mut invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    init_basic_pool(&invariant, &token_x, &token_y);
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);
    let fee = Percentage::from_scale(6, 3);
    let tick_spacing = 10;
    let fee_tier = FeeTier { fee, tick_spacing };
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

    let amount = 500;
    assert!(!token_x_program
        .send(REGULAR_USER_2, FTAction::Mint(amount))
        .main_failed());

    increase_allowance(&token_x_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), 500);

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), 500);
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), 1000);

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);
    let slippage = SqrtPrice::new(MIN_SQRT_PRICE);

    token_y_program
        .send(REGULAR_USER_2, FTAction::FailNextTransfer)
        .assert_success();

    let _res = invariant.send_and_assert_error(
        REGULAR_USER_2,
        InvariantAction::Swap {
            pool_key,
            x_to_y: true,
            amount: swap_amount,
            by_amount_in: true,
            sqrt_price_limit: slippage,
        },
        InvariantError::RecoverableTransferError,
    );

    let pool_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_eq!(pool_before, pool_after);

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), 0);
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), 0);

    assert_eq!(
        vec![(ActorId::from(TOKEN_X_ID), swap_amount)],
        get_user_balances(&invariant, REGULAR_USER_2)
    );

    assert_eq!(
        balance_of(&token_x_program, INVARIANT_ID),
        500 + swap_amount.get()
    );
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), 1000);

    assert_eq!(pool_after.fee_growth_global_x, FeeGrowth::new(0));
    assert_eq!(pool_after.fee_growth_global_y, FeeGrowth::new(0));

    assert_eq!(pool_after.fee_protocol_token_x, TokenAmount::new(0));
    assert_eq!(pool_after.fee_protocol_token_y, TokenAmount::new(0));

    invariant
        .send(
            REGULAR_USER_2,
            InvariantAction::ClaimLostTokens {
                token: TOKEN_X_ID.into(),
            },
        )
        .assert_success();

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), 500);
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), 0);

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), 500);
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), 1000);
}
