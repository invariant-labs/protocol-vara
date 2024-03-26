use crate::errors::InvariantError;
use crate::storage::fee_tier::FeeTier;
use gstd::{Decode, Encode, TypeInfo, Vec};

#[derive(Decode, Encode, TypeInfo, PartialEq, Eq, Clone, Debug, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct FeeTiers {
    fee_tiers: Vec<FeeTier>,
}

impl FeeTiers {
    pub fn add(&mut self, fee_tier: &FeeTier) -> Result<(), InvariantError> {
        if self.contains(fee_tier) {
            return Err(InvariantError::FeeTierAlreadyExist);
        }

        self.fee_tiers.push(*fee_tier);
        Ok(())
    }

    pub fn remove(&mut self, fee_tier: &FeeTier) -> Result<(), InvariantError> {
        let index = self
            .fee_tiers
            .iter()
            .position(|vec_fee_tier| vec_fee_tier == fee_tier)
            .ok_or(InvariantError::FeeTierNotFound)?;

        self.fee_tiers.remove(index);
        Ok(())
    }

    pub fn contains(&self, fee_tier: &FeeTier) -> bool {
        self.fee_tiers.contains(fee_tier)
    }

    pub fn get_all(&self) -> Vec<FeeTier> {
        self.fee_tiers.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gstd::vec;
    use math::percentage::Percentage;
    use decimal::*;

    #[test]
    fn test_add() {
        let fee_tier_keys = &mut FeeTiers::default();
        let fee_tier_key = FeeTier::default();
        let new_fee_tier_key = FeeTier::new(Percentage::new(0), 2).unwrap();

        fee_tier_keys.add(&fee_tier_key).unwrap();
        assert!(fee_tier_keys.contains(&fee_tier_key));
        assert!(!fee_tier_keys.contains(&new_fee_tier_key));

        let result = fee_tier_keys.add(&fee_tier_key);
        assert_eq!(result, Err(InvariantError::FeeTierAlreadyExist));
    }

    #[test]
    fn test_remove() {
        let fee_tier_keys = &mut FeeTiers::default();
        let fee_tier_key = FeeTier::default();

        fee_tier_keys.add(&fee_tier_key).unwrap();

        fee_tier_keys.remove(&fee_tier_key).unwrap();
        assert!(!fee_tier_keys.contains(&fee_tier_key));

        let result = fee_tier_keys.remove(&fee_tier_key);
        assert_eq!(result, Err(InvariantError::FeeTierNotFound));
    }

    #[test]
    fn test_get_all() {
        let fee_tier_keys = &mut FeeTiers::default();
        let fee_tier_key = FeeTier::default();
        let new_fee_tier_key = FeeTier::new(Percentage::new(0), 2).unwrap();

        let result = fee_tier_keys.get_all();
        assert_eq!(result, vec![]);
        assert_eq!(result.len(), 0);

        fee_tier_keys.add(&fee_tier_key).unwrap();
        fee_tier_keys.add(&new_fee_tier_key).unwrap();

        let result = fee_tier_keys.get_all();
        assert_eq!(result, vec![fee_tier_key, new_fee_tier_key]);
        assert_eq!(result.len(), 2);
    }
}