use contracts::PoolKey;
use gstd::*;
use gtest::*;
use io::*;
use math::{sqrt_price::SqrtPrice, token_amount::TokenAmount};

use crate::test_helpers::gtest::InvariantResult;

#[track_caller]
pub fn quote(
    invariant: &Program,
    from: u64,
    pool_key: PoolKey,
    x_to_y: bool,
    amount: TokenAmount,
    by_amount_in: bool,
    sqrt_price_limit: SqrtPrice,
    expected_error: Option<impl Into<String>>,
) -> Option<QuoteResult> {
    let res = invariant.send(
        from,
        InvariantAction::Quote {
            pool_key,
            x_to_y,
            amount,
            by_amount_in,
            sqrt_price_limit,
        },
    );

    if let Some(err) = expected_error {
        res.assert_panicked_with(err);
        return None;
    }
    res.assert_success();
    let events = res.emitted_events();
    assert_eq!(events.len(), 1);
    let quote_return = events
        .last()
        .unwrap()
        .decoded_event::<InvariantEvent>()
        .unwrap();

    if let InvariantEvent::Quote(quote_return) = quote_return {
        Some(quote_return)
    } else {
        panic!("unexpected event: {:?}", quote_return)
    }
}
