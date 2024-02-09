use gmeta::Metadata;

pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = ();
    type Handle = ();
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = ();
}
