use crate::send_request;
use crate::test_helpers::gtest::InvariantResult;
use contracts::{InvariantError, PoolKey};
use gstd::*;
use gtest::*;
use io::*;
use math::{sqrt_price::SqrtPrice, token_amount::TokenAmount};

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
        .decoded_event::<(String, String, (TokenAmount, TokenAmount))>()
        .unwrap()
        .2
        .into()
}
