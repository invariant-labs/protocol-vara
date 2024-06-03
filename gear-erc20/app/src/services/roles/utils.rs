use core::any::{Any, TypeId};
use gstd::{
    collections::{BTreeSet, HashMap},
    ActorId, Decode, Encode, String, TypeInfo,
};
use primitive_types::U256;

// Replace with NonEmptySet
// Consider array of [bit; RoleNumber]
pub type RolesSet = BTreeSet<TypeId>;
pub type RolesMap = HashMap<ActorId, RolesSet>;
pub type RolesRegistry = HashMap<&'static str, TypeId>;
pub(crate) type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Error {
    BadOrigin,
    DuplicateRole,
    UnknownRole,
}

pub trait Role: Any {
    fn name() -> &'static str;
}
