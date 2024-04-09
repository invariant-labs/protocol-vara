use contracts::*;

#[allow(dead_code)]
pub fn pools_are_identical_no_timestamp(pool: &Pool, other_pool: &Pool) {
  let Pool {
      liquidity,
      sqrt_price,
      current_tick_index,
      fee_growth_global_x,
      fee_growth_global_y,
      fee_protocol_token_x,
      fee_protocol_token_y,
      start_timestamp: _start_timestamp,
      last_timestamp: _last_timestamp,
      fee_receiver,
  } = pool;
  assert_eq!(*liquidity, other_pool.liquidity);
  assert_eq!(*sqrt_price, other_pool.sqrt_price);
  assert_eq!(*current_tick_index, other_pool.current_tick_index);
  assert_eq!(*fee_growth_global_x, other_pool.fee_growth_global_x);
  assert_eq!(*fee_growth_global_y, other_pool.fee_growth_global_y);
  assert_eq!(*fee_protocol_token_x, other_pool.fee_protocol_token_x);
  assert_eq!(*fee_protocol_token_y, other_pool.fee_protocol_token_y);
  assert_eq!(*fee_receiver, other_pool.fee_receiver);
}