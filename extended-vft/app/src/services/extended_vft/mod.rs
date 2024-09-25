use gstd::msg;
use sails_rs::{collections::HashSet, gstd::service, prelude::*};
mod funcs;
use crate::services;
use vft_service::{Service as VftService, Storage};

#[derive(Default)]
pub struct ExtendedStorage {
    minters: HashSet<ActorId>,
    burners: HashSet<ActorId>,
    admins: HashSet<ActorId>,
    transfer_fail: bool,
}

static mut EXTENDED_STORAGE: Option<ExtendedStorage> = None;

#[derive(Encode, Decode, TypeInfo)]
pub enum Event {
    Minted { to: ActorId, value: U256 },
    Burned { from: ActorId, value: U256 },
}
#[derive(Clone)]
pub struct ExtendedService {
    vft: VftService,
}

impl ExtendedService {
    pub fn seed(name: String, symbol: String, decimals: u8) -> Self {
        let admin = msg::source();
        unsafe {
            EXTENDED_STORAGE = Some(ExtendedStorage {
                admins: [admin].into(),
                minters: [admin].into(),
                burners: [admin].into(),
                transfer_fail: false,
            });
        };
        ExtendedService {
            vft: <VftService>::seed(name, symbol, decimals),
        }
    }

    pub fn get_mut(&mut self) -> &'static mut ExtendedStorage {
        unsafe {
            EXTENDED_STORAGE
                .as_mut()
                .expect("Extended vft is not initialized")
        }
    }
    pub fn get(&self) -> &'static ExtendedStorage {
        unsafe {
            EXTENDED_STORAGE
                .as_ref()
                .expect("Extended vft is not initialized")
        }
    }
}

#[service(extends = VftService, events = Event)]
impl ExtendedService {
    pub fn new() -> Self {
        Self {
            vft: VftService::new(),
        }
    }

    pub fn set_transfer_fail(&mut self, flag: bool) {
        #[cfg(feature = "test")]
        {
            self.ensure_is_admin();
            self.get_mut().transfer_fail = flag;
        }
    }

    pub fn transfer(&mut self, to: ActorId, value: U256) -> bool {
        #[cfg(feature = "test")]
        {
            if self.get().transfer_fail {
                panic!("Manually forced panic")
            }
        }
        self.vft.transfer(to, value)
    }

    pub fn transfer_from(&mut self, from: ActorId, to: ActorId, value: U256) -> bool {
        #[cfg(feature = "test")]
        {
            if self.get().transfer_fail {
                panic!("Manually forced panic")
            }
        }
        self.vft.transfer_from(from, to, value)
    }

    pub fn mint(&mut self, to: ActorId, value: U256) -> bool {
        if !self.get().minters.contains(&msg::source()) {
            panic!("Not allowed to mint")
        };

        let mutated = services::utils::panicking(|| {
            funcs::mint(Storage::balances(), Storage::total_supply(), to, value)
        });
        if mutated {
            self.notify_on(Event::Minted { to, value })
                .expect("Notification Error");
        }
        mutated
    }

    pub fn burn(&mut self, from: ActorId, value: U256) -> bool {
        if !self.get().burners.contains(&msg::source()) {
            panic!("Not allowed to burn")
        };

        let mutated = services::utils::panicking(|| {
            funcs::burn(Storage::balances(), Storage::total_supply(), from, value)
        });
        if mutated {
            self.notify_on(Event::Burned { from, value })
                .expect("Notification Error");
        }
        mutated
    }

    pub fn grant_admin_role(&mut self, to: ActorId) {
        self.ensure_is_admin();
        self.get_mut().admins.insert(to);
    }
    pub fn grant_minter_role(&mut self, to: ActorId) {
        self.ensure_is_admin();
        self.get_mut().minters.insert(to);
    }
    pub fn grant_burner_role(&mut self, to: ActorId) {
        self.ensure_is_admin();
        self.get_mut().burners.insert(to);
    }

    pub fn revoke_admin_role(&mut self, from: ActorId) {
        self.ensure_is_admin();
        self.get_mut().admins.remove(&from);
    }
    pub fn revoke_minter_role(&mut self, from: ActorId) {
        self.ensure_is_admin();
        self.get_mut().minters.remove(&from);
    }
    pub fn revoke_burner_role(&mut self, from: ActorId) {
        self.ensure_is_admin();
        self.get_mut().burners.remove(&from);
    }
    pub fn minters(&self) -> Vec<ActorId> {
        self.get().minters.clone().into_iter().collect()
    }

    pub fn burners(&self) -> Vec<ActorId> {
        self.get().burners.clone().into_iter().collect()
    }

    pub fn admins(&self) -> Vec<ActorId> {
        self.get().admins.clone().into_iter().collect()
    }
}

impl ExtendedService {
    fn ensure_is_admin(&self) {
        if !self.get().admins.contains(&msg::source()) {
            panic!("Not admin")
        };
    }
}
impl AsRef<VftService> for ExtendedService {
    fn as_ref(&self) -> &VftService {
        &self.vft
    }
}
