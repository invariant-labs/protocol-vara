use super::utils::*;
use core::any::TypeId;
use gstd::ActorId;

pub fn grant_role<T: Role>(roles: &mut RolesMap, actor: ActorId) -> bool {
    let set = roles.entry(actor).or_default();
    set.insert(TypeId::of::<T>())
}

// TODO (breathx): optimize me
pub fn remove_role<T: Role>(roles: &mut RolesMap, actor: ActorId) -> bool {
    let Some(set) = roles.get_mut(&actor) else {
        return false;
    };

    let res = set.remove(&TypeId::of::<T>());

    if set.is_empty() {
        roles.remove(&actor);
    }

    res
}

pub fn has_role<T: Role>(roles: &RolesMap, actor: ActorId) -> bool {
    roles
        .get(&actor)
        .map(|s| s.contains(&TypeId::of::<T>()))
        .unwrap_or(false)
}
