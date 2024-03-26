use crate::test_helpers::gclient::utils::*;
use contracts::FeeTier;
use gclient::GearApi;
use gstd::prelude::*;
use io::*;


pub async fn get_fee_tiers(
    api: &GearApi,
    invariant: ProgramId,
)-> Vec<FeeTier>{
    let payload = InvariantStateQuery::GetFeeTiers.encode();
    let state = api
        .read_state::<InvariantState>(invariant.into(), payload)
        .await
        .expect("Failed to read state");
    match state {
        InvariantState::QueriedFeeTiers(fee_tiers) => {
            return fee_tiers;
        }
        _ => {
            panic!("Invalid state");
        }
    }
}
