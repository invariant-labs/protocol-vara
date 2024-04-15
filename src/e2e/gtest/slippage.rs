use crate::test_helpers::gtest::*;
use decimal::*;
use fungible_token_io::*;
use gstd::{prelude::*, ActorId};
use gtest::*;
use io::*;
use math::{sqrt_price::SqrtPrice, token_amount::TokenAmount};

#[test]
fn test_basic_slippage() {
    let sys = System::new();
    sys.init_logger();

    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (invariant, token_x_program, token_y_program) = init_slippage_invariant_and_tokens(&sys);

    let pool_key =
        init_slippage_pool_with_liquidity(&sys, &invariant, &token_x_program, &token_y_program);

    let amount = 10u128.pow(8);
    let swap_amount = TokenAmount(amount);
    token_x_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount,
            },
        )
        .assert_success();

    let target_sqrt_price = SqrtPrice::new(1009940000000000000000001);
    invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::Swap {
                pool_key,
                x_to_y: false,
                amount: swap_amount,
                by_amount_in: true,
                sqrt_price_limit: target_sqrt_price,
            },
        )
        .assert_success();

    let expected_sqrt_price = SqrtPrice::new(1009940000000000000000000);
    let pool = get_pool(&invariant, token_x, token_y, pool_key.fee_tier).unwrap();
    assert_eq!(pool.sqrt_price, expected_sqrt_price);
}
