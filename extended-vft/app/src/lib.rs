#![no_std]
#![allow(clippy::new_without_default)]

use sails_rs::prelude::*;
mod services;
use services::extended_vft::ExtendedService;
pub struct Program(());

#[program]
impl Program {
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        ExtendedService::seed(name, symbol, decimals);
        Self(())
    }

    pub fn vft(&self) -> ExtendedService {
        ExtendedService::new()
    }
}
