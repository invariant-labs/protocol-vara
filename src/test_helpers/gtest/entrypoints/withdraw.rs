use contracts::PoolKey;
use gstd::*;
use gtest::*;
use io::*;
use math::{sqrt_price::SqrtPrice, token_amount::TokenAmount};

use crate::test_helpers::gtest::InvariantResult;

#[track_caller]
pub fn withdraw_single_token(
    invariant: &Program,
    from: u64,
    token: impl Into<ActorId>,
    amount: Option<u128>,
    expected_error: Option<impl Into<String>>,
) -> Option<TokenAmount> {
    let res = invariant.send(
        from,
        InvariantAction::WithdrawSingleToken {
            token: token.into(),
            amount: amount.and_then(|am| TokenAmount(am).into()),
        },
    );

    if let Some(err) = expected_error {
        res.assert_panicked_with(err);
        return None;
    }

    res.assert_success();
    let events = res.emitted_events();
    assert_eq!(events.len(), 1);
    let deposit_return = events
        .last()
        .unwrap()
        .decoded_event::<InvariantEvent>()
        .unwrap();

    if let InvariantEvent::TokenWithdrawn(x) = deposit_return {
        Some(x)
    } else {
        panic!("unexpected event: {:?}", deposit_return)
    }
}

#[track_caller]
pub fn withdraw_token_pair(
    invariant: &Program,
    from: u64,
    token_x: impl Into<ActorId>,
    token_x_amount: Option<u128>,
    token_y: impl Into<ActorId>,
    token_y_amount: Option<u128>,
    expected_error: Option<impl Into<String>>,
) -> Option<(TokenAmount, TokenAmount)> {
    let res = invariant.send(
        from,
        InvariantAction::WithdrawTokenPair {
            token_x: (
                token_x.into(),
                token_x_amount.and_then(|am| TokenAmount(am).into()),
            ),
            token_y: (
                token_y.into(),
                token_y_amount.and_then(|am| TokenAmount(am).into()),
            ),
        },
    );

    if let Some(err) = expected_error {
        res.assert_panicked_with(err);
        return None;
    }

    res.assert_success();
    let events = res.emitted_events();
    assert_eq!(events.len(), 1);
    let deposit_return = events
        .last()
        .unwrap()
        .decoded_event::<InvariantEvent>()
        .unwrap();

    if let InvariantEvent::TokenPairWithdrawn(x, y) = deposit_return {
        Some((x, y))
    } else {
        panic!("unexpected event: {:?}", deposit_return)
    }
}
