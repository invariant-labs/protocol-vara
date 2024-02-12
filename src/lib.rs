#![no_std]
extern crate alloc;
#[cfg(test)]
mod e2e;
mod math;

use crate::errors::InvariantError;
use crate::math::percentage::Percentage;
use crate::InvariantQuery::*;
use decimal::*;
use gstd::{
    exec,
    msg::{self, reply},
    prelude::*,
};
use io::{collections::fee_tiers::FeeTiers, storage::fee_tier::FeeTier, *};

#[derive(Default, Clone)]
pub struct Invariant {
    pub fee_tiers: FeeTiers,
    pub config: InvariantConfig,
}

impl Invariant {
    pub fn change_protocol_fee(&mut self, protocol_fee: u128) -> () {
        self.config.protocol_fee = protocol_fee;
    }

    pub fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
        let caller = exec::program_id();

        if fee_tier.tick_spacing == 0 || fee_tier.tick_spacing > 100 {
            return Err(InvariantError::InvalidTickSpacing);
        }

        if fee_tier.fee >= Percentage::from_integer(1).get() {
            return Err(InvariantError::InvalidFee);
        }

        if caller != self.config.admin {
            return Err(InvariantError::NotAdmin);
        }

        self.fee_tiers.add(fee_tier)?;
        Ok(())
    }

    pub fn fee_tier_exist(&self, fee_tier: FeeTier) -> bool {
        self.fee_tiers.contains(fee_tier)
    }

    pub fn remove_fee_tier(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
        let caller = exec::program_id();

        if caller != self.config.admin {
            return Err(InvariantError::NotAdmin);
        }

        self.fee_tiers.remove(fee_tier)?;
        Ok(())
    }

    pub fn get_fee_tiers(&self) -> Vec<FeeTier> {
        self.fee_tiers.get_all()
    }
}

static mut INVARIANT: Option<Invariant> = None;

#[no_mangle]
extern "C" fn init() {
    let init: InitInvariant = msg::load().expect("Unable to decode InitInvariant");

    let invariant = Invariant {
        config: InvariantConfig {
            admin: exec::program_id(),
            protocol_fee: init.config.protocol_fee,
        },
        fee_tiers: FeeTiers::default(),
    };

    unsafe {
        INVARIANT = Some(invariant);
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: InvariantAction = msg::load().expect("Unable to decode InvariantAction");
    let invariant = unsafe { INVARIANT.get_or_insert(Default::default()) };

    match action {
        InvariantAction::ChangeProtocolFee(protocol_fee) => {
            let res = invariant.change_protocol_fee(protocol_fee);
            reply(res, 0).expect("Unable to reply");
        }
        InvariantAction::AddFeeTier(fee_tier) => {
            let res = invariant.add_fee_tier(fee_tier);
            match res {
                Ok(r) => {
                    reply(r, 0).expect("Unable to reply");
                }
                Err(e) => {
                    reply(e, 1).expect("Unable to reply");
                }
            };
        }
        InvariantAction::RemoveFeeTier(fee_tier) => {
            let res = invariant.remove_fee_tier(fee_tier);
            match res {
                Ok(r) => {
                    reply(r, 0).expect("Unable to reply");
                }
                Err(e) => {
                    reply(e, 1).expect("Unable to reply");
                }
            };
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let query: InvariantQuery = msg::load().expect("Unable to decode InvariantAction");
    let invariant = unsafe { INVARIANT.get_or_insert(Default::default()) };
    match query {
        InvariantQuery::FeeTierExist(fee_tier) => {
            let res = invariant.fee_tier_exist(fee_tier);
            reply(res, 1).expect("Unable to reply");
        }
        InvariantQuery::GetFeeTiers => {
            let res = invariant.get_fee_tiers();
            reply(res, 0).expect("Unable to reply");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InvariantAction::*;
    use gstd::ActorId;
    use gtest::{Log, Program, System};

    pub const USER: [u8; 32] = [0; 32];

    #[test]
    fn test_init() {
        let sys = System::new();
        sys.init_logger();

        let program_id = 105;
        let program = Program::from_file_with_id(
            &sys,
            program_id,
            "./target/wasm32-unknown-unknown/release/invariant.wasm",
        );

        assert!(!program
            .send(
                100001,
                InitInvariant {
                    config: InvariantConfig {
                        admin: ActorId::new(USER),
                        protocol_fee: 100,
                    },
                },
            )
            .main_failed());
    }

    #[test]
    fn test_add_fee_tier() {
        let sys = System::new();
        sys.init_logger();
        let _ = Log::builder();
        let wasm_path = "./target/wasm32-unknown-unknown/release/invariant.wasm";
        let program_id = 105;
        let program = Program::from_file_with_id(&sys, program_id, wasm_path);
        let fee_tier = FeeTier::default();

        assert!(!program
            .send(
                100001,
                InitInvariant {
                    config: InvariantConfig {
                        admin: ActorId::new(USER),
                        protocol_fee: 100,
                    },
                },
            )
            .main_failed());
        {
            let logs = program.send(100001, AddFeeTier(fee_tier));
            assert!(!logs.main_failed());
            assert!(!logs.others_failed());

            let expected_response = Log::builder()
                .source(program_id)
                .dest(100001)
                .payload_bytes(().encode());

            assert!(logs.contains(&expected_response))
        }

        {
            let logs = program.read_state(fee_tier);

            panic!("{:?}", logs);

            let expected_response = Log::builder()
                .source(program_id)
                .dest(100001)
                .payload_bytes(true.encode());

            // assert!(logs.contains(&expected_response))
        }
        {
            let logs = program.send(100001, GetFeeTiers);
            assert!(!logs.main_failed());
            assert!(!logs.others_failed());

            let expected_response = Log::builder()
                .source(program_id)
                .dest(100001)
                .payload_bytes(vec![fee_tier].encode());

            // assert!(logs.contains(&expected_response))
        }
        {
            let logs = program.send(100001, RemoveFeeTier(fee_tier));
            assert!(!logs.main_failed());
            assert!(!logs.others_failed());
        }
    }
}
