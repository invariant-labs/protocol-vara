use crate::errors::InvariantError;
use decimal::*;
use math::types::percentage::Percentage;
use sails_rs::prelude::*;

#[derive(Encode, Decode, TypeInfo, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct FeeTier {
    pub fee: Percentage,
    pub tick_spacing: u16,
}

impl Default for FeeTier {
    fn default() -> Self {
        Self {
            fee: Percentage::new(0),
            tick_spacing: 1,
        }
    }
}

impl FeeTier {
    pub fn new(fee: Percentage, tick_spacing: u16) -> Result<Self, InvariantError> {
        if tick_spacing == 0 || tick_spacing > 100 {
            return Err(InvariantError::InvalidTickSpacing);
        }

        if fee > Percentage::from_integer(1) {
            return Err(InvariantError::InvalidFee);
        }

        Ok(Self { fee, tick_spacing })
    }
}
