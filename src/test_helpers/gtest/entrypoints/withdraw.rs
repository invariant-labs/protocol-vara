use contracts::{InvariantError, PoolKey};
use decimal::U256;
use gstd::*;
use gtest::*;
use io::*;
use math::{sqrt_price::SqrtPrice, token_amount::TokenAmount};

use crate::{send_request, test_helpers::gtest::InvariantResult};

#[track_caller]
pub fn withdraw_single_token(
    invariant: &Program,
    from: u64,
    token: impl Into<ActorId>,
    amount: Option<U256>,
    expected_error: Option<impl Into<String>>,
) -> Option<TokenAmount> {
    let res = send_request!(
        program: invariant,
        user: from,
        service_name: "Service",
        action: "WithdrawSingleToken",
        payload: (token.into(), amount.and_then(|am| TokenAmount(am).into()))
    );

    if let Some(err) = expected_error {
        res.assert_panicked_with(err);
        return None;
    }

    res.assert_success();
    let events = res.emitted_events();
    assert_eq!(events.len(), 1);
    events
        .last()
        .unwrap()
        .decoded_event::<(String, String, TokenAmount)>()
        .unwrap()
        .2
        .into()
}

#[track_caller]
pub fn withdraw_token_pair(
    invariant: &Program,
    from: u64,
    token_x: impl Into<ActorId>,
    token_x_amount: Option<U256>,
    token_y: impl Into<ActorId>,
    token_y_amount: Option<U256>,
    expected_error: Option<impl Into<String>>,
) -> Option<(TokenAmount, TokenAmount)> {
    let res = send_request!(
        program: invariant,
        user: from,
        service_name: "Service",
        action: "WithdrawTokenPair",
        payload: ((token_x.into(), token_x_amount.and_then(|am| TokenAmount(am).into())), (token_y.into(), token_y_amount.and_then(|am| TokenAmount(am).into())))
    );

    if let Some(err) = expected_error {
        res.assert_panicked_with(err);
        return None;
    }

    res.assert_success();
    let events = res.emitted_events();
    assert_eq!(events.len(), 1);

    events
        .last()
        .unwrap()
        .decoded_event::<(String, String, (TokenAmount, TokenAmount))>()
        .unwrap()
        .2
        .into()
}
