#![no_std]

use io::*;
use gstd::prelude::*;
use gmeta::metawasm;

#[metawasm]
pub mod metafns {
    pub type State = InvariantState;

    pub fn protocol_fee(state: State) -> u128 {
        let config = state.config;

        config.protocol_fee
    }
}
