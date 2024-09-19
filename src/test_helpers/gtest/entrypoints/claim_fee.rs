use crate::{send_request, test_helpers::gtest::*};
use contracts::{InvariantError, PoolKey};
use gtest::*;
use io::*;
use math::{sqrt_price::SqrtPrice, token_amount::TokenAmount};
use sails_rs::prelude::*;

#[track_caller]
pub fn claim_fee(
    invariant: &Program,
    from: u64,
    position_id: u32,
    expected_error: Option<impl Into<String>>,
) -> Option<(TokenAmount, TokenAmount)> {
    let res = send_request!(
        program: invariant,
        user: from,
        service_name: "Service",
        action: "ClaimFee",
        payload: (position_id)
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
