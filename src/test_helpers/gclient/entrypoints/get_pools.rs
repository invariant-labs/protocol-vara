use crate::test_helpers::gclient::utils::*;
use contracts::{InvariantError, PoolKey} ;
use gclient::GearApi;
use gstd::prelude::*;
use io::*;

pub async fn get_pools(
    api: &GearApi,
    invariant: ProgramId,
    size: u8,
    offset: u16,
    expected_error: Option<InvariantError>,
)-> Option<Vec<PoolKey>>{
    let payload = InvariantStateQuery::GetPools(size, offset).encode();
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
          if let InvariantState::Pools(pool) = state {
            return pool.into();
          }
          panic!("Unexpected state {:?}", state);
        }
    }
}
