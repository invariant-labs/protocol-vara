use contracts::PoolKey;
use gstd::*;
use gtest::*;
use io::*;
use math::{sqrt_price::SqrtPrice, token_amount::TokenAmount};

use crate::test_helpers::gtest::InvariantResult;

#[track_caller]
pub fn claim_fee(
    invariant: &Program,
    from: u64,
    position_id: u32,
    expected_error: Option<impl Into<String>>,
  ) -> Option<(TokenAmount, TokenAmount)> {
    let res = invariant.send(
        from,
        InvariantAction::ClaimFee {
          position_id,
        },
    );

    if let Some(err) = expected_error {
        res.assert_panicked_with(err);
        return None;
    }

    res.assert_success();
    let events = res.emitted_events();
    assert_eq!(events.len(), 1);
    let claim_return = events
        .last()
        .unwrap()
        .decoded_event::<InvariantEvent>()
        .unwrap();

    if let InvariantEvent::ClaimFee(x,y) = claim_return {
        Some((x,y))
    } else {
        panic!("unexpected event: {:?}", claim_return)
    }
}
