#![no_std]

use escrow_io::*;
use gstd::prelude::*;
use io::{InvariantConfig, InvariantState};
use gmeta::metawasm;

#[metawasm]
pub mod metafns {
    pub type State = InvariantState;

    pub fn protocol_fee(state: State) -> u128 {
        let config = *state.config;

        config.protocol_fee
    }
}
