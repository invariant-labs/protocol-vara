use crate::{send_request, test_helpers::gtest::InvariantResult};
use contracts::{InvariantError, PoolKey};
use decimal::U256;
use gstd::{vec, String, ToString};
use gtest::*;
use io::*;
use math::{sqrt_price::SqrtPrice, token_amount::TokenAmount};
use sails_rs::ActorId;

#[track_caller]
pub fn deposit_single_token(
    invariant: &Program,
    from: u64,
    token: impl Into<ActorId>,
    amount: U256,
    expected_error: Option<impl Into<String>>,
) -> Option<TokenAmount> {
    let res = send_request!(
        program: invariant,
        user: from,
        service_name: "Service",
        action: "DepositSingleToken",
        payload: (token.into(), TokenAmount(amount))
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
        .decoded_event::<TokenAmount>()
        .unwrap()
        .into()
}

#[track_caller]
pub fn deposit_token_pair(
    invariant: &Program,
    from: u64,
    token_x: impl Into<ActorId>,
    token_x_amount: U256,
    token_y: impl Into<ActorId>,
    token_y_amount: U256,
    expected_error: Option<impl Into<String>>,
) -> Option<(TokenAmount, TokenAmount)> {
    let res = send_request!(
        program: invariant,
        user: from,
        service_name: "Service",
        action: "DepositTokenPair",
        payload: ((token_x.into(), TokenAmount(token_x_amount)), (token_y.into(), TokenAmount(token_y_amount)))
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
        .decoded_event::<(TokenAmount, TokenAmount)>()
        .unwrap()
        .into()
}

pub fn deposit_vara(
    invariant: &Program,
    from: u64,
    amount: u128,
    expected_error: Option<impl Into<String>>,
) -> Option<TokenAmount> {
    let res = send_request!(
        program: invariant,
        user: from,
        service_name: "Service",
        action: "DepositVara",
        payload: (),
        value: amount
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
        .decoded_event::<TokenAmount>()
        .unwrap()
        .into()
}
