use gstd::scale_info;
use scale::{Encode, Decode};
use scale_info::TypeInfo;
use gstd::Vec;
use crate::storage::fee_tier::FeeTier;
use crate::errors::InvariantError;

#[derive(Decode, Encode, TypeInfo, PartialEq, Eq, Clone, Debug, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct FeeTiers {
    fee_tiers: Vec<FeeTier>
}

impl FeeTiers {
    pub fn add(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
        if self.contains(fee_tier) {
            return Err(InvariantError::FeeTierAlreadyExist);
        }

        self.fee_tiers.push(fee_tier);
        Ok(())
    }

    pub fn remove(&mut self, fee_tier: FeeTier) -> Result<(), InvariantError> {
        let index = self
            .fee_tiers
            .iter()
            .position(|vec_fee_tier| *vec_fee_tier == fee_tier)
            .ok_or(InvariantError::FeeTierNotFound)?;

        self.fee_tiers.remove(index);
        Ok(())
    }

    pub fn contains(&self, fee_tier: FeeTier) -> bool {
        self.fee_tiers.contains(&fee_tier)
    }

    pub fn get_all(&self) -> Vec<FeeTier> {
        self.fee_tiers.clone()
    }
}