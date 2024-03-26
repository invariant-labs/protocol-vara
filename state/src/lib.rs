#![no_std]

use io::*;
use gstd::prelude::*;
use gmeta::metawasm;
use contracts::FeeTier;

#[metawasm]
pub mod metafns {
    pub type State = InvariantState;

    pub fn fee_tier_exists(state: State, fee_tier: FeeTier) -> bool {
        match state {
            InvariantState::QueriedFeeTiers(fee_tiers) => {
                fee_tiers.contains(&fee_tier)
            },
            _ => panic!("InvariantState is not {}", stringify!(InvariantState::QueriedFeeTiers)),
        }
    }
}
