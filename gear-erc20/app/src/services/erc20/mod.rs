// TODO (breathx): replace sails_rs::ActorId with gstd's one.
#![allow(clippy::unused_unit)]

use crate::services;
use core::{cmp::Ordering, fmt::Debug, marker::PhantomData};
use gstd::{ext, format, Decode, Encode, String, TypeInfo, Vec};
use primitive_types::U256;
use sails_rs::gstd::{service, msg};
use sails_rs::ActorId;
use sails_rs::Box;
#[cfg(feature = "test")]
use storage::TransferFailStorage;
use storage::{AllowancesStorage, BalancesStorage, MetaStorage, TotalSupplyStorage};

pub use utils::*;

pub mod funcs;
pub mod storage;
pub(crate) mod utils;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Event {
    Approval {
        owner: sails_rs::ActorId,
        spender: sails_rs::ActorId,
        value: U256,
    },
    Transfer {
        from: sails_rs::ActorId,
        to: sails_rs::ActorId,
        // TODO (breathx and team): use or not to use `NonZeroU256`?
        value: U256,
    },
}

// TODO (sails): isn't services - modules?
#[derive(Clone)]
pub struct ERC20Service {}

impl ERC20Service {
    pub fn seed(name: String, symbol: String, decimals: u8) -> Self {
        let _res = AllowancesStorage::default();
        debug_assert!(_res.is_ok());

        let _res = BalancesStorage::default();
        debug_assert!(_res.is_ok());

        let _res = MetaStorage::with_data(name, symbol, decimals);
        debug_assert!(_res.is_ok());

        let _res = TotalSupplyStorage::default();
        debug_assert!(_res.is_ok());

        #[cfg(feature = "test")]
        {
            let _res = TransferFailStorage::default();
            debug_assert!(_res.is_ok());
        }

        Self {}
    }
}

// TODO (sails): consider renaming `EventTrigger` -> `Notifier`/`Informer`.
// TODO (sails): fix that requires `Encode`, `Decode`, `TypeInfo` and `Vec` in scope.
// TODO (sails): fix that requires explicit `-> ()`. ALREADY EXISTS
// TODO (sails): let me specify error as subset of strings (Display of my Error) -> thats common flow for us.
// TODO (sails): gstd::ActorId, primitive_types::H256/U256, [u8; 32], NonZeroStuff are primitives!.
// TODO (sails): service(events = Event, error = Error)
// #[service(events = Event, error = Error)]
#[service(events = Event)]
impl ERC20Service {
    // TODO (sails): hide this into macro.
    pub fn new() -> Self {
        Self {}
    }

    pub fn allowance(&self, owner: sails_rs::ActorId, spender: sails_rs::ActorId) -> U256 {
        funcs::allowance(AllowancesStorage::as_ref(), owner.into(), spender.into())
    }

    pub fn approve(&mut self, spender: sails_rs::ActorId, value: U256) -> bool {
        let owner = msg::source().into();

        let mutated = funcs::approve(AllowancesStorage::as_mut(), owner, spender.into(), value);

        if mutated {
            self.notify_on(Event::Approval {
                owner: owner.into(),
                spender,
                value,
            })
            .unwrap();
        }

        mutated
    }

    pub fn balance_of(&self, owner: sails_rs::ActorId) -> U256 {
        funcs::balance_of(BalancesStorage::as_ref(), owner.into())
    }

    pub fn decimals(&self) -> u8 {
        MetaStorage::decimals()
    }

    // TODO (sails): allow using references.
    pub fn name(&self) -> String {
        MetaStorage::name()
    }

    pub fn symbol(&self) -> String {
        MetaStorage::symbol()
    }

    pub fn total_supply(&self) -> U256 {
        TotalSupplyStorage::get()
    }

    pub fn transfer(&mut self, to: sails_rs::ActorId, value: U256) -> bool {
        #[cfg(feature = "test")]
        {
            if *TransferFailStorage::as_ref() {
                panic!("Manually forced panic");
            }
        }

        let from = msg::source().into();

        let mutated = services::utils::panicking(move || {
            funcs::transfer(BalancesStorage::as_mut(), from, to.into(), value)
        });

        if mutated {
            // let value = value
            //     .try_into()
            //     .expect("Infallible since `transfer` executed successfully");

            self.notify_on(Event::Transfer {
                from: from.into(),
                to,
                value,
            })
            .unwrap();
        }

        mutated
    }

    // TODO (breathx): rename me once bug in sails fixed.
    pub fn transfer_from(
        &mut self,
        from: sails_rs::ActorId,
        to: sails_rs::ActorId,
        value: U256,
    ) -> bool {
        #[cfg(feature = "test")]
        {
            gstd::debug!("[ERC-20] TransferFrom {:?}", (from, to, value));
            if *TransferFailStorage::as_ref() {
                panic!("Manually forced panic");
            }
        }

        let spender = msg::source().into();

        let mutated = services::utils::panicking(move || {
            funcs::transfer_from(
                AllowancesStorage::as_mut(),
                BalancesStorage::as_mut(),
                spender,
                from.into(),
                to.into(),
                value,
            )
        });

        if mutated {
            // let value = value
            //     .try_into()
            //     .expect("Infallible since `transfer_from` executed successfully");

            self.notify_on(Event::Transfer { from, to, value }).unwrap();
        }

        mutated
    }

    // TODO (breathx): delete me once multi services are implemented.
    pub fn set_balance(&mut self, new_balance: U256) -> bool {
        let owner = msg::source().into();

        let balance = funcs::balance_of(BalancesStorage::as_ref(), owner);

        let new_total_supply = match balance.cmp(&new_balance) {
            Ordering::Greater => TotalSupplyStorage::get().saturating_sub(balance - new_balance),
            Ordering::Less => TotalSupplyStorage::get().saturating_add(new_balance - balance),
            Ordering::Equal => return false,
        };

        let non_zero_new_balance = new_balance
            .try_into()
            .expect("Infallible since NonZero b/c new_balance != balance");

        BalancesStorage::as_mut().insert(owner, non_zero_new_balance);
        *TotalSupplyStorage::as_mut() = new_total_supply;

        true
    }

    pub fn set_fail_transfer(&mut self, fail: bool) {
        #[cfg(feature = "test")]
        {
            *TransferFailStorage::as_mut() = fail;
        }
    }
}
