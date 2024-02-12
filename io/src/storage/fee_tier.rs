use crate::errors::InvariantError;
use gstd::scale_info;
use scale::{Decode, Encode};
use scale_info::TypeInfo;
#[derive(gstd::Decode, gstd::Encode, TypeInfo, PartialEq, Eq, Clone, Copy, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct FeeTier {
    pub fee: u64,
    pub tick_spacing: u16,
}

impl Default for FeeTier {
    fn default() -> Self {
        Self {
            fee: 0u64,
            tick_spacing: 1,
        }
    }
}

impl FeeTier {
    pub fn new(fee: u64, tick_spacing: u16) -> Result<Self, InvariantError> {
        if tick_spacing == 0 || tick_spacing > 100 {
            return Err(InvariantError::InvalidTickSpacing);
        }

        // if fee > Percentage::from_integer(1) {
        //     return Err(InvariantError::InvalidFee);
        // }

        Ok(Self { fee, tick_spacing })
    }
}
