use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{sqrt_price::SqrtPrice, token_amount::TokenAmount, MAX_SQRT_PRICE, MIN_SQRT_PRICE};
use sails_rs::prelude::*;

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
        SqrtPrice::new(MIN_SQRT_PRICE.into())
    } else {
        SqrtPrice::new(MAX_SQRT_PRICE.into())
    };

    let quote_result = quote(
        &invariant,
        from,
        pool_key,
        x_to_y,
        amount,
        by_amount_in,
        sqrt_price_limit,
    )
    .unwrap();

    let (swapped_token, returned_token) = if x_to_y {
        (pool_key.token_x, pool_key.token_y)
    } else {
        (pool_key.token_y, pool_key.token_x)
    };

    assert_eq!(
        deposit_single_token(
            &invariant,
            from,
            swapped_token,
            quote_result.amount_in.get(),
            None::<&str>
        ),
        Some(quote_result.amount_in)
    );

    swap(
        &invariant,
        from,
        pool_key,
        x_to_y,
        amount,
        by_amount_in,
        quote_result.target_sqrt_price,
    )
    .assert_success();

    assert_eq!(
        withdraw_single_token(
            &invariant,
            from,
            returned_token,
            quote_result.amount_out.get().into(),
            None::<&str>
        ),
        Some(quote_result.amount_out)
    );
}
