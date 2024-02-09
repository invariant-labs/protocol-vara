use gmeta::Metadata;
pub mod collections;
pub mod storage;

pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = ();
    type Handle = ();
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = ();
}
