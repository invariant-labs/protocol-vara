use crate::test_helpers::gclient::utils::*;
use contracts::{InvariantError, Position};
use gclient::GearApi;
use gstd::prelude::*;
use io::*;

pub async fn get_position(
    api: &GearApi,
    invariant: ProgramId,
    owner: UserId,
    index: u32,
    expected_error: Option<InvariantError>,
) -> Option<Position> {
    let payload = InvariantStateQuery::GetPosition(owner.into(), index).encode();
    let state = api
        .read_state::<InvariantState>(invariant.into(), payload)
        .await
        .expect("Failed to read state");
    match expected_error {
        Some(e) => {
            assert_eq!(state, InvariantState::QueryFailed(e));
            return None;
        }
        None => {
            if let InvariantState::Position(position) = state {
                return position.into();
            }
            panic!("Unexpected state {:?}", state);
        }
    }
}
