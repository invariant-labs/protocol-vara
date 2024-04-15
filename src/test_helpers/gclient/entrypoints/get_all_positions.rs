use crate::test_helpers::gclient::utils::*;
use contracts::Position;
use gclient::GearApi;
use gstd::prelude::*;
use io::*;

#[allow(dead_code)]
pub async fn get_all_positions(
    api: &GearApi,
    invariant: ProgramId,
    owner_id: UserId
)-> Vec<Position>{
    let payload = InvariantStateQuery::GetAllPositions(owner_id.into()).encode();
    let state = api
        .read_state::<InvariantStateReply>(invariant.into(), payload)
        .await
        .expect("Failed to read state");
    match state {
        InvariantStateReply::Positions(positions) => {
            return positions;
        }
        _ => {
            panic!("Invalid state");
        }
    }
}
