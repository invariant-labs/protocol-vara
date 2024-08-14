use crate::{services, ServiceOf};
use core::marker::PhantomData;
use gstd::String;
use gstd::{ActorId, Decode, Encode, ToString, TypeInfo, Vec};
use primitive_types::U256;
use sails_rs::format;
use sails_rs::gstd::service;
use sails_rs::Box;

// TODO (breathx): once supported in sails impl Clone here
pub struct AggregatedService {
    pub erc20_service: services::erc20::ERC20Service,
    pub pausable_service: ServiceOf<services::pausable::Service>,
}

impl AggregatedService {
    pub fn seed(
        erc20_service: services::erc20::ERC20Service,
        pausable_service: ServiceOf<services::pausable::Service>,
    ) -> Self {
        Self {
            erc20_service,
            pausable_service,
        }
    }
}

#[service]
impl AggregatedService {
    pub fn new(
        erc20_service: services::erc20::ERC20Service,
        pausable_service: ServiceOf<services::pausable::Service>,
    ) -> Self {
        Self {
            erc20_service,
            pausable_service,
        }
    }

    pub fn allowance(&self, owner: sails_rs::ActorId, spender: sails_rs::ActorId) -> U256 {
        self.erc20_service.allowance(owner, spender)
    }

    pub fn approve(&mut self, spender: sails_rs::ActorId, value: U256) -> bool {
        services::utils::panicking(|| {
            (!self.pausable_service.is_paused())
                .then_some(())
                .ok_or(services::pausable::Error::Paused)
        });
        self.erc20_service.approve(spender, value)
    }

    pub fn balance_of(&self, owner: sails_rs::ActorId) -> U256 {
        self.erc20_service.balance_of(owner)
    }

    pub fn decimals(&self) -> u8 {
        self.erc20_service.decimals()
    }

    pub fn name(&self) -> String {
        self.erc20_service.name()
    }

    pub fn symbol(&self) -> String {
        self.erc20_service.symbol()
    }

    pub fn total_supply(&self) -> U256 {
        self.erc20_service.total_supply()
    }

    pub fn transfer(&mut self, to: sails_rs::ActorId, value: U256) -> bool {
        services::utils::panicking(|| {
            (!self.pausable_service.is_paused())
                .then_some(())
                .ok_or(services::pausable::Error::Paused)
        });
        self.erc20_service.transfer(to, value)
    }

    pub fn transfer_from(
        &mut self,
        from: sails_rs::ActorId,
        to: sails_rs::ActorId,
        value: U256,
    ) -> bool {
        services::utils::panicking(|| {
            (!self.pausable_service.is_paused())
                .then_some(())
                .ok_or(services::pausable::Error::Paused)
        });
        self.erc20_service.transfer_from(from, to, value)
    }

    pub fn set_fail_transfer(&mut self, fail: bool) {
        self.erc20_service.set_fail_transfer(fail)
    }
}
