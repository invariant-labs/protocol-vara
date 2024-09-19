#![no_std]
extern crate alloc;
#[cfg(test)]
mod e2e;
#[cfg(test)]
mod test_helpers;

use io::InvariantConfig;
use sails_rs::gstd::{program, GStdExecContext};
mod invariant_service;
mod invariant_storage;
pub use contracts::{
    AwaitingTransfer, FeeTier, FeeTiers, InvariantError, Pool, PoolKey, PoolKeys, Pools, Position,
    Positions, Tick, Tickmap, Ticks, TransferType, UpdatePoolTick,
};
use invariant_service::InvariantService;

pub struct InvariantProgram(());

#[program]
impl InvariantProgram {
    pub fn new(config: InvariantConfig) -> Self {
        InvariantService::<GStdExecContext>::seed(config);
        Self(())
    }

    pub fn service(&self) -> InvariantService<GStdExecContext> {
        InvariantService::new(GStdExecContext::new())
    }
}
