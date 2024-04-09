use crate::test_helpers::gclient::utils::*;
use gclient::GearApi;
use gstd::prelude::*;
use io::*;

#[allow(dead_code)]
pub async fn get_protocol_fee(
    api: &GearApi,
    invariant: ProgramId,
)-> u128{
    let payload = InvariantStateQuery::GetProtocolFee.encode();
    let state = api
        .read_state::<InvariantStateReply>(invariant.into(), payload)
        .await
        .expect("Failed to read state");
    match state {
        InvariantStateReply::ProtocolFee(protocol_fee) => {
            return protocol_fee;
        }
        _ => {
            panic!("Invalid state");
        }
    }
}
