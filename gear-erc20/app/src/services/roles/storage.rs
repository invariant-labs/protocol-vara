use super::utils::{RolesMap, RolesRegistry};

crate::declare_storage!(module: roles, name: RolesStorage, ty: RolesMap);

impl RolesStorage {
    pub fn with_capacity(capacity: usize) -> Result<(), RolesMap> {
        Self::set(RolesMap::with_capacity(capacity))
    }

    pub fn default() -> Result<(), RolesMap> {
        Self::with_capacity(u16::MAX as usize)
    }
}

crate::declare_storage!(module: roles_registry, name: RolesRegistryStorage, ty: RolesRegistry);

impl RolesRegistryStorage {
    pub fn with_capacity(capacity: usize) -> Result<(), RolesRegistry> {
        Self::set(RolesRegistry::with_capacity(capacity))
    }

    pub fn default() -> Result<(), RolesRegistry> {
        Self::with_capacity(8)
    }
}
