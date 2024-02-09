#![no_std]

use escrow_io::*;
use gstd::prelude::*;
use io::{InvariantConfig, InvariantState};
use primitive_types::U256;

#[gmeta::metawasm]
pub mod metafns {
    pub type State = InvariantState;

    pub fn protocol_fee(state: State) -> u128 {
        let config = *state.config;

        config.protocol_fee
    }
}
