use crate::{Pool, PoolKey, InvariantError};
use sails_rtl::collections::HashMap;

#[derive(Debug, Default)]
pub struct Pools {
  pools: HashMap<PoolKey, Pool>,
}

impl Pools {
  pub fn add(&mut self, pool_key: &PoolKey, pool: &Pool) -> Result<(), InvariantError> {
      self.pools
          .get(pool_key)
          .map_or(Ok(()), |_| Err(InvariantError::PoolAlreadyExist))?;

      self.pools.insert(pool_key.clone(), pool.clone());
      Ok(())
  }

  pub fn update(&mut self, pool_key: &PoolKey, pool: &Pool) -> Result<(), InvariantError> {
      self.get(pool_key)?;

      self.pools.insert(pool_key.clone(), pool.clone());
      Ok(())
  }

  #[allow(dead_code)]
  pub fn remove(&mut self, pool_key: &PoolKey) -> Result<(), InvariantError> {
      self.get(pool_key)?;

      self.pools.remove(pool_key);
      Ok(())
  }

  pub fn get(&self, pool_key: &PoolKey) -> Result<Pool, InvariantError> {
      let pool = self
          .pools
          .get(pool_key)
          .cloned()
          .ok_or(InvariantError::PoolNotFound)?;

      Ok(pool)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::FeeTier;
  use math::percentage::Percentage;
  use decimal::*;
  use sails_rtl::ActorId;

  #[test]
  fn test_add() {
      let pools = &mut Pools::default();
      let token_x = ActorId::from([0x01; 32]);
      let token_y = ActorId::from([0x02; 32]);
      let fee_tier = FeeTier {
          fee: Percentage::new(0),
          tick_spacing: 1,
      };
      let new_fee_tier = FeeTier {
          fee: Percentage::new(0),
          tick_spacing: 2,
      };
      let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
      let new_pool_key = PoolKey::new(token_x, token_y, new_fee_tier).unwrap();
      let pool = Pool::default();

      pools.add(&pool_key, &pool).unwrap();
      assert_eq!(pools.get(&pool_key), Ok(pool.clone()));
      assert_eq!(pools.get(&new_pool_key), Err(InvariantError::PoolNotFound));

      let result = pools.add(&pool_key, &pool);
      assert_eq!(result, Err(InvariantError::PoolAlreadyExist));
  }

  #[test]
  fn test_update() {
      let pools = &mut Pools::default();
      let token_x = ActorId::from([0x01; 32]);
      let token_y = ActorId::from([0x02; 32]);
      let fee_tier = FeeTier {
          fee: Percentage::new(0),
          tick_spacing: 1,
      };
      let new_fee_tier = FeeTier {
          fee: Percentage::new(0),
          tick_spacing: 2,
      };
      let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
      let new_pool_key = PoolKey::new(token_x, token_y, new_fee_tier).unwrap();
      let pool = Pool::default();
      let new_pool = Pool {
          current_tick_index: 1,
          ..Pool::default()
      };

      pools.add(&pool_key, &pool).unwrap();

      pools.update(&pool_key, &new_pool).unwrap();
      assert_eq!(pools.get(&pool_key), Ok(new_pool.clone()));

      let result = pools.update(&new_pool_key, &new_pool);
      assert_eq!(result, Err(InvariantError::PoolNotFound));
  }

  #[test]
  fn test_remove() {
      let pools = &mut Pools::default();
      let token_x = ActorId::from([0x01; 32]);
      let token_y = ActorId::from([0x02; 32]);
      let fee_tier = FeeTier {
          fee: Percentage::new(0),
          tick_spacing: 1,
      };
      let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
      let pool = Pool::default();

      pools.add(&pool_key, &pool).unwrap();

      pools.remove(&pool_key).unwrap();
      assert_eq!(pools.get(&pool_key), Err(InvariantError::PoolNotFound));

      let result = pools.remove(&pool_key);
      assert_eq!(result, Err(InvariantError::PoolNotFound));
  }
}