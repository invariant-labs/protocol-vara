use crate::services::{self, pausable::roles::PauseAdmin, roles::storage::RolesStorage};
use core::marker::PhantomData;
use gstd::{msg, ActorId, Decode, Encode, String, TypeInfo, Vec};
use sails_rs::format;
use sails_rs::gstd::service;
use storage::StateStorage;
use sails_rs::Box;

pub use utils::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Event {
    Paused,
    Unpaused,
}

#[derive(Clone)]
pub struct Service {
    roles_service: services::roles::RolesService,
}

impl Service {
    pub fn seed(mut roles_service: services::roles::RolesService, admin: ActorId) -> Self {
        let _res = StateStorage::set(State::Active);
        debug_assert!(_res.is_ok());

        roles_service.register_role::<PauseAdmin>();

        let _res = roles_service.grant_role::<PauseAdmin>(admin);
        debug_assert!(_res);

        Self { roles_service }
    }

    pub fn ensure_unpaused(&self) -> Result<(), Error> {
        (!self.is_paused()).then_some(()).ok_or(Error::Paused)
    }
}

#[service(events=Event)]
impl Service {
    pub fn new(roles_service: services::roles::RolesService) -> Self {
        Self { roles_service }
    }

    pub fn is_paused(&self) -> bool {
        StateStorage::as_ref().paused()
    }

    pub fn pause(&mut self) -> bool {
        services::utils::panicking(move || -> services::roles::Result<bool> {
            self.roles_service
                .ensure_has_role::<PauseAdmin>(msg::source())?;

            let mutated = funcs::pause(StateStorage::as_mut());

            if mutated {
                self.notify_on(Event::Paused).unwrap();
            }

            Ok(mutated)
        })
    }

    pub fn unpause(&mut self) -> bool {
        services::utils::panicking(move || -> services::roles::Result<bool> {
            self.roles_service
                .ensure_has_role::<PauseAdmin>(msg::source())?;

            let mutated = funcs::unpause(StateStorage::as_mut());

            if mutated {
                self.notify_on(Event::Unpaused).unwrap();
            }

            Ok(mutated)
        })
    }

    // TODO (breathx): consider as atomic
    pub fn delegate_admin(&mut self, actor: sails_rs::ActorId) -> bool {
        services::utils::panicking(move || -> services::roles::Result<bool> {
            let source = msg::source();

            self.roles_service.ensure_has_role::<PauseAdmin>(source)?;

            if ActorId::from(actor) == source {
                return Ok(false);
            }

            let _res = self.roles_service.grant_role::<PauseAdmin>(actor.into());
            debug_assert!(_res);

            let _res = self.roles_service.remove_role::<PauseAdmin>(source);
            debug_assert!(_res);

            Ok(true)
        })
    }
}

pub mod funcs {
    use super::State;

    pub fn pause(state: &mut State) -> bool {
        if state.paused() {
            return false;
        }

        state.switch();

        true
    }

    pub fn unpause(state: &mut State) -> bool {
        if !state.paused() {
            return false;
        }

        state.switch();

        true
    }
}

pub mod roles {
    crate::declare_role!(PauseAdmin);
}

pub mod storage {
    use super::State;

    crate::declare_storage!(name: StateStorage, ty: State);
}

mod utils {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
    #[codec(crate = gstd::codec)]
    #[scale_info(crate = gstd::scale_info)]
    pub enum Error {
        Paused,
    }

    pub enum State {
        Active,
        Paused,
    }

    impl State {
        pub fn paused(&self) -> bool {
            matches!(self, Self::Paused)
        }

        pub fn switch(&mut self) {
            if self.paused() {
                *self = Self::Active
            } else {
                *self = Self::Paused
            }
        }
    }
}
