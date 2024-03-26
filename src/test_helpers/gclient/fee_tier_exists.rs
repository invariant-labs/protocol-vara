use crate::test_helpers::gclient::utils::*;
use contracts::FeeTier;
use gclient::GearApi;
use gstd::prelude::*;
use io::*;

pub async fn fee_tier_exists(
    api: &GearApi,
    invariant: ProgramId,
    fee_tier: FeeTier,
)-> bool {
    let payload = InvariantStateQuery::FeeTierExist(fee_tier).encode();
    let state = api
        .read_state::<InvariantState>(invariant.into(), payload)
        .await
        .expect("Failed to read state");
    match state {
        InvariantState::FeeTierExist(exists) => {
            return exists;
        }
        _ => {
            panic!("Invalid state");
        }
    }
}
