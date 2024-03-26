use crate::test_helpers::gclient::{utils::*, fee_tier_exists};
use contracts::{FeeTier,InvariantError};
use gstd::{prelude::*, codec::decode_from_bytes};
use gclient::{EventListener, GearApi, EventProcessor};
use io::*;

pub async fn add_fee_tier(api: &GearApi, listener: &mut EventListener, invariant: ProgramId, fee_tier: FeeTier, expected_error: Option<InvariantError>) {
  let message_id = send_message(api, invariant, InvariantAction::AddFeeTier(fee_tier)).await;
  let res = listener.reply_bytes_on(message_id.into()).await.expect("Failed to get reply");
  let bytes = res.1.unwrap().into();
  match expected_error {
    Some(e) => {
      assert_eq!(decode_from_bytes::<InvariantEvent>(bytes).unwrap(), InvariantEvent::ActionFailed(e));
    }
    None => {
      assert!(fee_tier_exists(api, invariant, fee_tier).await)
    }
  }
}