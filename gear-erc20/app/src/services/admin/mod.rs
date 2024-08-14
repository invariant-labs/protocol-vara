use self::utils::Role;
use crate::services::admin::roles::{FungibleAdmin, FungibleBurner, FungibleMinter};
use crate::{services, ServiceOf};
use core::marker::PhantomData;
use gstd::{exec, msg, String};
use gstd::{ActorId, Decode, Encode, ToString, TypeInfo, Vec};
use primitive_types::U256;
use sails_rs::gstd::service;
use sails_rs::Box;
use super::erc20::storage::{AllowancesStorage, BalancesStorage, TotalSupplyStorage};
use sails_rs::format;
pub mod funcs;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Event {
    Minted {
        to: sails_rs::ActorId,
        value: U256,
    },
    Burned {
        from: sails_rs::ActorId,
        value: U256,
    },
    Killed {
        inheritor: sails_rs::ActorId,
    },
}

// TODO (breathx): once supported in sails impl Clone here
pub struct AdminService {
    roles_service: services::roles::RolesService,
    pausable_service: ServiceOf<services::pausable::Service>,
}

impl AdminService {
    pub fn seed(
        mut roles_service: services::roles::RolesService,
        pausable_service: ServiceOf<services::pausable::Service>,
        admin: ActorId,
    ) -> Self {
        roles_service.register_role::<FungibleAdmin>();
        roles_service.register_role::<FungibleBurner>();
        roles_service.register_role::<FungibleMinter>();

        let _res = roles_service.grant_role::<FungibleAdmin>(admin);
        debug_assert!(_res);

        Self {
            roles_service,
            pausable_service,
        }
    }
}

#[service(events=Event)]
impl AdminService {
    pub fn new(
        roles_service: services::roles::RolesService,
        pausable_service: ServiceOf<services::pausable::Service>,
    ) -> Self {
        Self {
            roles_service,
            pausable_service,
        }
    }

    pub fn mint(&mut self, to: sails_rs::ActorId, value: U256) -> bool {
        services::utils::panicking(|| {
            (!self.pausable_service.is_paused())
                .then_some(())
                .ok_or(services::pausable::Error::Paused)
        });

        services::utils::panicking(|| {
            self.roles_service
                .ensure_has_role::<FungibleMinter>(msg::source())
        });
        let mutated = services::utils::panicking(|| {
            funcs::mint(
                BalancesStorage::as_mut(),
                TotalSupplyStorage::as_mut(),
                to.into(),
                value,
            )
        });

        if mutated {
            self.notify_on(
                Event::Minted { to, value }
            ).unwrap();
        }

        mutated
    }

    pub fn burn(&mut self, from: sails_rs::ActorId, value: U256) -> bool {
        services::utils::panicking(|| {
            (!self.pausable_service.is_paused())
                .then_some(())
                .ok_or(services::pausable::Error::Paused)
        });

        services::utils::panicking(|| {
            self.roles_service
                .ensure_has_role::<FungibleBurner>(msg::source())
        });

        let mutated = services::utils::panicking(|| {
            funcs::burn(
                BalancesStorage::as_mut(),
                TotalSupplyStorage::as_mut(),
                from.into(),
                value,
            )
        });

        if mutated {
            self.notify_on(Event::Burned { from, value }).unwrap();
        }

        mutated
    }

    // TODO (sails): consider `#[panicking]` and then it expect Result in return type, automatically wrapping closure
    pub fn allowances_reserve(&mut self, additional: u32) -> () {
        services::utils::panicking(|| {
            (!self.pausable_service.is_paused())
                .then_some(())
                .ok_or(services::pausable::Error::Paused)
        });

        funcs::allowances_reserve(AllowancesStorage::as_mut(), additional as usize)
    }

    pub fn balances_reserve(&mut self, additional: u32) -> () {
        services::utils::panicking(|| {
            (!self.pausable_service.is_paused())
                .then_some(())
                .ok_or(services::pausable::Error::Paused)
        });

        funcs::balances_reserve(BalancesStorage::as_mut(), additional as usize)
    }

    pub fn maps_data(&self) -> ((u32, u32), (u32, u32)) {
        let ((a_len, a_cap), (b_len, b_cap)) =
            funcs::maps_data(AllowancesStorage::as_ref(), BalancesStorage::as_ref());

        ((a_len as u32, a_cap as u32), (b_len as u32, b_cap as u32))
    }

    pub fn allowances(
        &self,
        skip: u32,
        take: u32,
    ) -> Vec<((sails_rs::ActorId, sails_rs::ActorId), U256)> {
        funcs::allowances(AllowancesStorage::as_ref(), skip as usize, take as usize)
            .into_iter()
            .map(|((id1, id2), v)| ((id1.into(), id2.into()), v.into()))
            .collect()
    }

    pub fn balances(&self, skip: u32, take: u32) -> Vec<(sails_rs::ActorId, U256)> {
        funcs::balances(BalancesStorage::as_ref(), skip as usize, take as usize)
            .into_iter()
            .map(|(id, v)| (id.into(), v.into()))
            .collect()
    }
    pub fn has_role(&self, actor: sails_rs::ActorId, role: String) -> bool {
        self.roles_service.has_role(actor, role)
    }

    pub fn roles(&self) -> Vec<String> {
        self.roles_service.roles()
    }

    pub fn grant_role(&mut self, to: sails_rs::ActorId, role: Role) -> bool {
        services::utils::panicking(|| {
            (!self.pausable_service.is_paused())
                .then_some(())
                .ok_or(services::pausable::Error::Paused)
        });

        services::utils::panicking(|| -> Result<bool, services::roles::Error> {
            self.roles_service
                .ensure_has_role::<FungibleAdmin>(msg::source())?;

            let res = match role {
                Role::Admin => self.roles_service.grant_role::<FungibleAdmin>(to.into()),
                Role::Minter => self.roles_service.grant_role::<FungibleMinter>(to.into()),
                Role::Burner => self.roles_service.grant_role::<FungibleBurner>(to.into()),
            };

            Ok(res)
        })
    }

    pub fn remove_role(&mut self, from: sails_rs::ActorId, role: Role) -> bool {
        services::utils::panicking(|| {
            (!self.pausable_service.is_paused())
                .then_some(())
                .ok_or(services::pausable::Error::Paused)
        });

        services::utils::panicking(|| -> Result<bool, services::roles::Error> {
            self.roles_service
                .ensure_has_role::<FungibleAdmin>(msg::source())?;

            let res = match role {
                Role::Admin => self.roles_service.remove_role::<FungibleAdmin>(from.into()),
                Role::Minter => self
                    .roles_service
                    .remove_role::<FungibleMinter>(from.into()),
                Role::Burner => self
                    .roles_service
                    .remove_role::<FungibleBurner>(from.into()),
            };

            Ok(res)
        })
    }

    // TODO (sails): self `pub fn kill(&mut self) -> !`
    pub fn kill(&mut self, inheritor: sails_rs::ActorId) -> () {
        services::utils::panicking(|| {
            self.roles_service
                .ensure_has_role::<FungibleAdmin>(msg::source())
        });

        self.notify_on(Event::Killed { inheritor }).unwrap();

        exec::exit(inheritor.into())
    }
}

pub mod roles {
    crate::declare_role!(FungibleAdmin);
    crate::declare_role!(FungibleBurner);
    crate::declare_role!(FungibleMinter);
}

pub mod utils {
    use core::fmt::Debug;

    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
    #[codec(crate = gstd::codec)]
    #[scale_info(crate = gstd::scale_info)]
    pub enum Role {
        Admin,
        Burner,
        Minter,
    }
}
