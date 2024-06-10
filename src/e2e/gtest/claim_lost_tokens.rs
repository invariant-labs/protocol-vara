use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gstd::*;
use gtest::*;
use io::*;
use math::{
    percentage::Percentage, sqrt_price::SqrtPrice, token_amount::TokenAmount, MAX_SQRT_PRICE,
    MIN_SQRT_PRICE,
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
    mint(&token_y_program, REGULAR_USER_2, amount).assert_success();

    increase_allowance(&token_y_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), 500);

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), 500);
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), 1000);

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);
    let slippage = SqrtPrice::new(MAX_SQRT_PRICE);

    set_transfer_fail(&token_x_program, true).assert_success();

    let res = invariant.send_and_assert_error(
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

    assert_eq!(res.emitted_events().len(), 2);

    let pool_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_ne!(pool_before, pool_after);

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), 0);
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), 0);
    let swap_outcome = 496;
    assert_eq!(
        vec![(ActorId::from(TOKEN_X_ID), TokenAmount(swap_outcome))],
        get_user_balances(&invariant, REGULAR_USER_2)
    );

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), 500);
    assert_eq!(
        balance_of(&token_y_program, INVARIANT_ID),
        1000 + swap_amount.get()
    );

    set_transfer_fail(&token_x_program, false).assert_success();

    invariant
        .send(
            REGULAR_USER_2,
            InvariantAction::ClaimLostTokens {
                token: TOKEN_X_ID.into(),
            },
        )
        .assert_success();

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), swap_outcome);
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), 0);

    assert_eq!(
        balance_of(&token_x_program, INVARIANT_ID),
        500 - swap_outcome
    );
    assert_eq!(
        balance_of(&token_y_program, INVARIANT_ID),
        1000 + swap_amount.get()
    );
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
    mint(&token_x_program, REGULAR_USER_2, amount).assert_success();

    increase_allowance(&token_x_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), 500);

    assert_eq!(balance_of(&token_x_program, INVARIANT_ID), 500);
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), 1000);

    let pool_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let swap_amount = TokenAmount::new(amount);
    let slippage = SqrtPrice::new(MIN_SQRT_PRICE);

    set_transfer_fail(&token_y_program, true).assert_success();

    let res = invariant.send_and_assert_error(
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

    assert_eq!(res.emitted_events().len(), 2);

    let pool_after = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    assert_ne!(pool_before, pool_after);

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), 0);
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), 0);

    let swap_outcome = 496;

    assert_eq!(
        vec![(ActorId::from(TOKEN_Y_ID), TokenAmount(swap_outcome))],
        get_user_balances(&invariant, REGULAR_USER_2)
    );

    assert_eq!(
        balance_of(&token_x_program, INVARIANT_ID),
        500 + swap_amount.get()
    );
    assert_eq!(balance_of(&token_y_program, INVARIANT_ID), 1000);

    set_transfer_fail(&token_y_program, false).assert_success();
    invariant
        .send(
            REGULAR_USER_2,
            InvariantAction::ClaimLostTokens {
                token: TOKEN_Y_ID.into(),
            },
        )
        .assert_success();

    assert_eq!(balance_of(&token_x_program, REGULAR_USER_2), 0);
    assert_eq!(balance_of(&token_y_program, REGULAR_USER_2), swap_outcome);

    assert_eq!(
        balance_of(&token_x_program, INVARIANT_ID),
        500 + swap_amount.get()
    );
    assert_eq!(
        balance_of(&token_y_program, INVARIANT_ID),
        1000 - swap_outcome
    );
}
