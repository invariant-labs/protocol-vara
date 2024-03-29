use crate::test_helpers::gclient::utils::*;
use contracts::{FeeTier, Pool, InvariantError} ;
use gclient::GearApi;
use gstd::{prelude::*, ActorId};
use io::*;

pub async fn get_pool(
    api: &GearApi,
    invariant: ProgramId,
    token_0: impl Into<ActorId>,
    token_1: impl Into<ActorId>,
    fee_tier: FeeTier,
    expected_error: Option<InvariantError>,
)-> Option<Pool>{
    let payload = InvariantStateQuery::GetPool(token_0.into(), token_1.into(), fee_tier).encode();
    let state = api
        .read_state::<InvariantState>(invariant.into(), payload)
        .await
        .expect("Failed to read state");
    match expected_error {
        Some(e) => {
            assert_eq!(
                state,
                InvariantState::QueryFailed(e)
            );
            return None;
        }
        None => {
          if let InvariantState::Pool(pool) = state {
            return pool.into();
          }
          panic!("Unexpected state {:?}", state);
        }
    
    }
}
