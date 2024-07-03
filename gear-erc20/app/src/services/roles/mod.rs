use crate::services;
use core::{any::TypeId, marker::PhantomData};
use gstd::{ActorId, Decode, Encode, String, ToString, TypeInfo, Vec};
use sails_rtl::format;
use sails_rtl::gstd::gservice;
use sails_rtl::Box;
use storage::{RolesRegistryStorage, RolesStorage};

pub mod funcs;
pub mod storage;
pub(crate) mod utils;

pub use utils::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Event {
    RoleGranted {
        actor: sails_rtl::ActorId,
        role: String,
    },
    RoleRemoved {
        actor: sails_rtl::ActorId,
        role: String,
    },
}

// TODO (sails): impl Clone for gstd event depositor
#[derive(Clone)]
pub struct RolesService {}

impl RolesService {
    pub fn seed() -> Self {
        let _res = RolesStorage::default();
        debug_assert!(_res.is_ok());

        let _res = RolesRegistryStorage::default();
        debug_assert!(_res.is_ok());

        Self {}
    }

    pub fn register_role<T: Role>(&mut self) -> Result<()> {
        let role_name = T::name();
        let role_type_id = TypeId::of::<T>();

        let registry = RolesRegistryStorage::as_mut();

        let Some(&type_id) = registry.get(&role_name) else {
            registry.insert(role_name, role_type_id);
            return Ok(());
        };

        if type_id != role_type_id {
            Err(Error::DuplicateRole)
        } else {
            Ok(())
        }
    }

    pub fn ensure_role_registered<T: Role>(&self) -> Result<()> {
        let role_name = T::name();
        let role_type_id = TypeId::of::<T>();

        RolesRegistryStorage::as_ref()
            .get(&role_name)
            .ok_or(Error::UnknownRole)
            .and_then(|type_id| {
                type_id
                    .eq(&role_type_id)
                    .then_some(())
                    .ok_or(Error::DuplicateRole)
            })
    }

    pub fn ensure_has_role<T: Role>(&self, actor: ActorId) -> Result<()> {
        self.ensure_role_registered::<T>()?;

        funcs::has_role::<T>(RolesStorage::as_ref(), actor)
            .then_some(())
            .ok_or(Error::BadOrigin)
    }

    pub fn has_role_by_name(&self, actor: ActorId, role: String) -> Result<bool> {
        let type_id = RolesRegistryStorage::as_ref()
            .get(role.as_str())
            .ok_or(Error::UnknownRole)?;

        let res = RolesStorage::as_ref()
            .get(&actor)
            .map(|v| v.contains(type_id))
            .unwrap_or(false);

        Ok(res)
    }
}

impl RolesService {
    pub fn grant_role<T: Role>(&mut self, actor: ActorId) -> bool {
        let mutated = services::utils::panicking(move || -> Result<bool> {
            self.ensure_role_registered::<T>()?;

            let res = funcs::grant_role::<T>(RolesStorage::as_mut(), actor);

            Ok(res)
        });
        mutated
    }

    pub fn remove_role<T: Role>(&mut self, actor: ActorId) -> bool {
        let cloned = self.clone();
        let mutated = services::utils::panicking(move || -> Result<bool> {
            cloned.ensure_role_registered::<T>()?;

            let res = funcs::remove_role::<T>(RolesStorage::as_mut(), actor);

            Ok(res)
        });

        if mutated {
            self.notify_on(Event::RoleRemoved {
                actor: actor.into(),
                role: T::name().to_string(),
            })
            .unwrap();
        }

        mutated
    }
}

#[gservice(events=Event)]
impl RolesService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn has_role(&self, actor: sails_rtl::ActorId, role: String) -> bool {
        services::utils::panicking(move || self.has_role_by_name(actor.into(), role))
    }

    pub fn roles(&self) -> Vec<String> {
        RolesRegistryStorage::as_ref()
            .keys()
            .map(ToString::to_string)
            .collect()
    }

    // TODO (breathx): actors keys, actors role queries
}
