use crate::{services, ServiceOf};
use core::marker::PhantomData;
use gstd::String;
use gstd::{ActorId, Decode, Encode, ToString, TypeInfo, Vec};
use primitive_types::U256;
use sails_rtl::gstd::events::{EventTrigger, GStdEventTrigger};
use sails_rtl::gstd::gservice;

pub type GstdDrivenService = Service<GStdEventTrigger<services::erc20::Event>>;

// TODO (breathx): once supported in sails impl Clone here
pub struct Service<X> {
    pub erc20_service: services::erc20::GstdDrivenService,
    pub pausable_service: ServiceOf<services::pausable::GstdDrivenService>,
    _phantom: PhantomData<X>,
}

impl<X> Service<X> {
    pub fn seed(
        erc20_service: services::erc20::GstdDrivenService,
        pausable_service: ServiceOf<services::pausable::GstdDrivenService>,
    ) -> Self {
        Self {
            erc20_service,
            pausable_service,
            _phantom: PhantomData,
        }
    }
}

#[gservice]
impl<X> Service<X>
where
    X: EventTrigger<services::erc20::Event>,
{
    pub fn new(
        erc20_service: services::erc20::GstdDrivenService,
        pausable_service: ServiceOf<services::pausable::GstdDrivenService>,
    ) -> Self {
        Self {
            erc20_service,
            pausable_service,
            _phantom: PhantomData,
        }
    }

    pub fn allowance(&self, owner: sails_rtl::ActorId, spender: sails_rtl::ActorId) -> U256 {
        self.erc20_service.allowance(owner, spender)
    }

    pub fn approve(&mut self, spender: sails_rtl::ActorId, value: U256) -> bool {
        services::utils::panicking(|| {
            (!self.pausable_service.is_paused())
                .then_some(())
                .ok_or(services::pausable::Error::Paused)
        });
        self.erc20_service.approve(spender, value)
    }

    pub fn balance_of(&self, owner: sails_rtl::ActorId) -> U256 {
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

    pub fn transfer(&mut self, to: sails_rtl::ActorId, value: U256) -> bool {
        services::utils::panicking(|| {
            (!self.pausable_service.is_paused())
                .then_some(())
                .ok_or(services::pausable::Error::Paused)
        });
        self.erc20_service.transfer(to, value)
    }

    pub fn transfer_from(
        &mut self,
        from: sails_rtl::ActorId,
        to: sails_rtl::ActorId,
        value: U256,
    ) -> bool {
        services::utils::panicking(|| {
            (!self.pausable_service.is_paused())
                .then_some(())
                .ok_or(services::pausable::Error::Paused)
        });
        self.erc20_service.transfer_from(from, to, value)
    }

    pub fn set_fail_transfer(&mut self, fail: bool){
        self.erc20_service.set_fail_transfer(fail)
    }
}
