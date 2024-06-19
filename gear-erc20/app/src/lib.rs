#![no_std]
#![allow(clippy::new_without_default)]
#![allow(dead_code)]
#![allow(unused)]

// TODO (sails): consider sub-services inside (for observing state, for example)
// TODO (sails): rename gstd event depositor here to use `notifier`::`Notifier`/`Informer`.
use gstd::{msg, ActorId, String};
use sails_rtl::gstd::{gprogram, groute};
use services::{admin, aggregated, erc20, pausable, roles};

pub mod services;

type ServiceOf<T> = <T as sails_rtl::gstd::services::Service>::Exposure;

pub struct Program(());

// TODO (sails): allow to import all necessary macros at once (gprogram, grout, etc).
// TODO (sails): stop forcing deriving default on `Program`.
#[gprogram]
impl Program {
    pub fn handle() -> u8 {
        return 1;
    }
    // TODO (sails): fix arguments are unused.
    // TODO (sails): `#[gconstructor]`
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        let source = msg::source();
        let program = Self(());

        let roles_service = roles::RolesService::seed();

        let erc20_service = <erc20::ERC20Service>::seed(name, symbol, decimals);

        let pausable_service = <pausable::Service>::seed(roles_service.clone(), source);

        let aggregated_service =
            <aggregated::AggregatedService>::seed(erc20_service, program.pausable());

        <admin::AdminService>::seed(roles_service, program.pausable(), source);

        Self(())
    }

    pub fn admin(&self) -> admin::AdminService {
        admin::AdminService::new(self.roles(), self.pausable())
    }

    // TODO (sails): service Erc20: Pausable [pipeline]
    // TODO (sails): Should reflect on multiple names as pipeline (aliasing)
    #[groute("erc20")]
    pub fn aggregated(&self) -> aggregated::AggregatedService {
        aggregated::AggregatedService::new(self.erc20(), self.pausable())
    }

    pub fn pausable(&self) -> pausable::Service {
        pausable::Service::new(self.roles())
    }

    fn roles(&self) -> roles::RolesService {
        roles::RolesService::new()
    }

    fn erc20(&self) -> erc20::ERC20Service {
        erc20::ERC20Service::new()
    }
}
