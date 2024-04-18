use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gstd::prelude::*;
use gtest::*;
use io::*;
use math::{sqrt_price::SqrtPrice, token_amount::TokenAmount, MAX_SQRT_PRICE, MIN_SQRT_PRICE};
#[track_caller]
pub fn swap_exact_limit(
    invariant: &Program,
    from: u64,
    pool_key: PoolKey,
    x_to_y: bool,
    amount: TokenAmount,
    by_amount_in: bool,
) {
    let sqrt_price_limit = if x_to_y {
        SqrtPrice::new(MIN_SQRT_PRICE)
    } else {
        SqrtPrice::new(MAX_SQRT_PRICE)
    };

    let quote_result = quote(
        &invariant,
        from,
        pool_key,
        x_to_y,
        amount,
        by_amount_in,
        sqrt_price_limit,
        None::<InvariantError>,
    )
    .unwrap();

    invariant
        .send(
            from,
            InvariantAction::Swap {
                pool_key,
                x_to_y,
                amount,
                by_amount_in,
                sqrt_price_limit: quote_result.target_sqrt_price,
            },
        )
        .assert_success();
}
