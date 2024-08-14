use crate::{InvariantError, PoolKey};
use sails_rs::{Vec, collections::HashMap};

#[derive(Debug, Default)]
pub struct PoolKeys {
  pool_keys: HashMap<PoolKey, u16>,
  pool_keys_by_index: HashMap<u16, PoolKey>,
  pool_keys_length: u16,
}

impl PoolKeys {
  pub fn get_index(&self, pool_key: &PoolKey) -> Option<u16> {
      self.pool_keys.get(pool_key).copied()
  }

  pub fn add(&mut self, pool_key: &PoolKey) -> Result<(), InvariantError> {
      if self.contains(pool_key) {
          return Err(InvariantError::PoolKeyAlreadyExist);
      }

      self.pool_keys.insert(*pool_key, self.pool_keys_length);
      self.pool_keys_by_index
          .insert(self.pool_keys_length, *pool_key);
      self.pool_keys_length += 1;

      Ok(())
  }

  #[allow(dead_code)]
  pub fn remove(&mut self, pool_key: &PoolKey) -> Result<(), InvariantError> {
      match self.get_index(pool_key) {
          Some(index) => {
              self.pool_keys_by_index.remove(&index);
              self.pool_keys_length -= 1;
              self.pool_keys.remove(pool_key);
              Ok(())
          }
          None => Err(InvariantError::PoolKeyNotFound),
      }
  }

  pub fn contains(&self, pool_key: &PoolKey) -> bool {
      self.pool_keys.get(pool_key).is_some()
  }

  pub fn get_all(&self, size: u16, offset: u16) -> Vec<PoolKey> {
    let offset_with_size = offset.checked_add(size).unwrap();

    let max = if offset_with_size > self.pool_keys_length {
        self.pool_keys_length
    } else {
        offset_with_size
    };

    (offset..max)
        .map(|index| *self.pool_keys_by_index.get(&index).unwrap())
        .collect()
  }

  pub fn count(&self) -> u16 {
    self.pool_keys_length
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use sails_rs::{ActorId, vec};
  use crate::FeeTier;
  use math::percentage::Percentage;
  use decimal::*;

  #[test]
  fn test_add() {
      let pool_keys = &mut PoolKeys::default();
      let pool_key = PoolKey::default();
      let token_x = ActorId::from([1; 32]);
      let token_y = ActorId::from([2; 32]);
      let fee_tier = FeeTier {
          fee: Percentage::new(0),
          tick_spacing: 1,
      };
      let new_pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

      pool_keys.add(&pool_key).unwrap();
      assert!(pool_keys.contains(&pool_key));
      assert!(!pool_keys.contains(&new_pool_key));

      let result = pool_keys.add(&pool_key);
      assert_eq!(result, Err(InvariantError::PoolKeyAlreadyExist));
  }

  #[test]
  fn test_remove() {
      let pool_keys = &mut PoolKeys::default();
      let pool_key = PoolKey::default();

      pool_keys.add(&pool_key).unwrap();

      pool_keys.remove(&pool_key).unwrap();
      assert!(!pool_keys.contains(&pool_key));

      let result = pool_keys.remove(&pool_key);
      assert_eq!(result, Err(InvariantError::PoolKeyNotFound));
  }

  #[test]
  fn test_get_all() {
      let pool_keys = &mut PoolKeys::default();
      let pool_key = PoolKey::default();
      let token_x = ActorId::from([1; 32]);
      let token_y = ActorId::from([2; 32]);
      let fee_tier = FeeTier {
          fee: Percentage::new(0),
          tick_spacing: 1,
      };
      let new_pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();

      let result = pool_keys.get_all(3, 0);
      assert_eq!(result, vec![]);
      assert_eq!(result.len(), 0);

      pool_keys.add(&pool_key).unwrap();
      pool_keys.add(&new_pool_key).unwrap();

      let result = pool_keys.get_all(3, 0);
      assert_eq!(result, vec![pool_key, new_pool_key]);
      assert_eq!(result.len(), 2);
  }
}