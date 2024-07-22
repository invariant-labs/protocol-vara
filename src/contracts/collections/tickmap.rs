use crate::{InvariantError, PoolKey};
use sails_rtl::collections::HashMap;
use math::{
    types::sqrt_price::{calculate_sqrt_price, get_max_tick, SqrtPrice}, MAX_TICK
};

pub const TICK_SEARCH_RANGE: i32 = 256;
pub const CHUNK_SIZE: i32 = 64;

#[derive(Debug, Default)]
pub struct Tickmap {
    pub bitmap: HashMap<(u16, PoolKey), u64>,
}

pub fn get_max_chunk(tick_spacing: u16) -> u16 {
    let max_tick = get_max_tick(tick_spacing);
    let max_bitmap_index = (max_tick + MAX_TICK) / tick_spacing as i32;
    let max_chunk_index = max_bitmap_index / CHUNK_SIZE;
    max_chunk_index as u16
}

pub fn tick_to_position(tick: i32, tick_spacing: u16) -> (u16, u8) {
    assert!(
        (-MAX_TICK..=MAX_TICK).contains(&tick),
        "tick not in range of <{}, {}>",
        -MAX_TICK,
        MAX_TICK
    );

    assert_eq!(
        (tick % tick_spacing as i32),
        0,
        "tick not divisible by tick spacing"
    );

    let bitmap_index = (tick + MAX_TICK) / tick_spacing as i32;

    let chunk: u16 = (bitmap_index / CHUNK_SIZE) as u16;
    let bit: u8 = (bitmap_index % CHUNK_SIZE) as u8;

    (chunk, bit)
}

pub fn position_to_tick(chunk: u16, bit: u8, tick_spacing: u16) -> i32 {
    let tick_range_limit = get_max_tick(tick_spacing);
    (chunk as i32 * CHUNK_SIZE * tick_spacing as i32 + bit as i32 * tick_spacing as i32)
        - tick_range_limit
}

pub fn get_bit_at_position(value: u64, position: u8) -> u64 {
    (value >> position) & 1
}

fn flip_bit_at_position(value: u64, position: u8) -> u64 {
    value ^ (1 << position)
}

pub fn get_search_limit(tick: i32, tick_spacing: u16, up: bool) -> i32 {
    let index = tick / tick_spacing as i32;

    // limit unscaled
    let limit = if up {
        // search range is limited to 256 at the time ...
        let range_limit = index + TICK_SEARCH_RANGE;
        // ...also ticks for sqrt_prices over 2^64 aren't needed
        let sqrt_price_limit = MAX_TICK / tick_spacing as i32;

        range_limit.min(sqrt_price_limit)
    } else {
        let range_limit = index - TICK_SEARCH_RANGE;
        let sqrt_price_limit = -MAX_TICK / tick_spacing as i32;

        range_limit.max(sqrt_price_limit)
    };

    // scaled by tick_spacing
    limit * tick_spacing as i32
}

impl Tickmap {
    pub fn next_initialized(&self, tick: i32, tick_spacing: u16, pool_key: PoolKey) -> Option<i32> {
        let limit = get_search_limit(tick, tick_spacing, true);

        if tick + tick_spacing as i32 > MAX_TICK {
            return None;
        }

        // add 1 to not check current tick
        let (mut chunk, mut bit) =
            tick_to_position(tick.checked_add(tick_spacing as i32)?, tick_spacing);
        let (limiting_chunk, limiting_bit) = tick_to_position(limit, tick_spacing);

        while chunk < limiting_chunk || (chunk == limiting_chunk && bit <= limiting_bit) {
            let mut shifted = self.bitmap.get(&(chunk, pool_key)).copied().unwrap_or(0) >> bit;

            if shifted != 0 {
                while shifted.checked_rem(2)? == 0 {
                    shifted >>= 1;
                    bit = bit.checked_add(1)?;
                }

                return if chunk < limiting_chunk || (chunk == limiting_chunk && bit <= limiting_bit)
                {
                    // no possibility of overflow
                    let index: i32 = (chunk as i32 * CHUNK_SIZE) + bit as i32;

                    Some(
                        index
                            .checked_sub(MAX_TICK / tick_spacing as i32)?
                            .checked_mul(tick_spacing.into())?,
                    )
                } else {
                    None
                };
            }

            // go to the text chunk
            if let Some(value) = chunk.checked_add(1) {
                chunk = value;
            } else {
                return None;
            }
            bit = 0;
        }

        None
    }

    // tick_spacing - spacing already scaled by tick_spacing
    pub fn prev_initialized(&self, tick: i32, tick_spacing: u16, pool_key: PoolKey) -> Option<i32> {
        // don't subtract 1 to check the current tick
        let limit = get_search_limit(tick, tick_spacing, false); // limit scaled by tick_spacing
        let (mut chunk, mut bit) = tick_to_position(tick, tick_spacing);
        let (limiting_chunk, limiting_bit) = tick_to_position(limit, tick_spacing);

        while chunk > limiting_chunk || (chunk == limiting_chunk && bit >= limiting_bit) {
            // always safe due to limitated domain of bit variable
            let mut mask = 1u128 << bit; // left = MSB direction (increase value)
            let value = self.bitmap.get(&(chunk, pool_key)).copied().unwrap_or(0) as u128;

            // enter if some of previous bits are initialized in current chunk
            if value.checked_rem(mask.checked_shl(1)?)? > 0 {
                // skip uninitalized ticks
                while value & mask == 0 {
                    mask >>= 1;
                    bit = bit.checked_sub(1)?;
                }

                // return first initalized tick if limiit is not exceeded, otherswise return None
                return if chunk > limiting_chunk || (chunk == limiting_chunk && bit >= limiting_bit)
                {
                    // no possibility to overflow
                    let index: i32 = (chunk as i32 * CHUNK_SIZE) + bit as i32;

                    Some(
                        index
                            .checked_sub(MAX_TICK / tick_spacing as i32)?
                            .checked_mul(tick_spacing.into())?,
                    )
                } else {
                    None
                };
            }

            // go to the next chunk
            if let Some(value) = chunk.checked_sub(1) {
                chunk = value;
            } else {
                return None;
            }
            bit = CHUNK_SIZE as u8 - 1;
        }

        None
    }

    // Finds closes initialized tick in direction of trade
    // and compares its sqrt_price to the sqrt_price limit of the trade
    pub fn get_closer_limit(
        &self,
        sqrt_price_limit: SqrtPrice,
        x_to_y: bool,
        current_tick: i32,
        tick_spacing: u16,
        pool_key: PoolKey,
    ) -> Result<(SqrtPrice, Option<(i32, bool)>), InvariantError> {
        let closes_tick_index = if x_to_y {
            self.prev_initialized(current_tick, tick_spacing, pool_key)
        } else {
            self.next_initialized(current_tick, tick_spacing, pool_key)
        };

        match closes_tick_index {
            Some(index) => {
                let sqrt_price = calculate_sqrt_price(index).unwrap();

                if (x_to_y && sqrt_price > sqrt_price_limit)
                    || (!x_to_y && sqrt_price < sqrt_price_limit)
                {
                    Ok((sqrt_price, Some((index, true))))
                } else {
                    Ok((sqrt_price_limit, None).into())
                }
            }
            None => {
                let index = get_search_limit(current_tick, tick_spacing, !x_to_y);
                let sqrt_price = calculate_sqrt_price(index).unwrap();

                if current_tick == index {
                    Err(InvariantError::TickLimitReached)?
                }

                if (x_to_y && sqrt_price > sqrt_price_limit)
                    || (!x_to_y && sqrt_price < sqrt_price_limit)
                {
                    Ok((sqrt_price, Some((index, false))))
                } else {
                    Ok((sqrt_price_limit, None))
                }
            }
        }
    }

    pub fn get(&self, tick: i32, tick_spacing: u16, pool_key: PoolKey) -> bool {
        let (chunk, bit) = tick_to_position(tick, tick_spacing);
        let returned_chunk = self.bitmap.get(&(chunk, pool_key)).copied().unwrap_or(0);
        get_bit_at_position(returned_chunk, bit) == 1
    }

    pub fn flip(&mut self, value: bool, tick: i32, tick_spacing: u16, pool_key: PoolKey) {
        let (chunk, bit) = tick_to_position(tick, tick_spacing);
        let returned_chunk = self.bitmap.get(&(chunk, pool_key)).copied().unwrap_or(0);

        assert_eq!(
            get_bit_at_position(returned_chunk, bit) == 0,
            value,
            "tick initialize tick again"
        );

        self.bitmap
            .insert((chunk, pool_key), flip_bit_at_position(returned_chunk, bit));
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::FeeTier;
    use decimal::*;
    use sails_rtl::ActorId;
    use math::{percentage::Percentage, sqrt_price::get_min_tick};


    #[test]
    fn test_get_closer_limit() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();
        tickmap.flip(true, 0, 1, pool_key);
        // tick limit closer
        {
            let (result, from_tick) = tickmap.get_closer_limit(SqrtPrice::from_integer(5), true, 100, 1, pool_key).unwrap();
            let expected = SqrtPrice::from_integer(5);
            assert_eq!(result, expected);
            assert_eq!(from_tick, None);
        }
        // trade limit closer
        {
            let (result, from_tick) =
                tickmap.get_closer_limit(SqrtPrice::from_scale(1, 1), true, 100, 1, pool_key).unwrap();
            let expected = SqrtPrice::from_integer(1);
            assert_eq!(result, expected);
            assert_eq!(from_tick, Some((0, true)));
        }
        // other direction
        {
            let (result, from_tick) =
                tickmap.get_closer_limit(SqrtPrice::from_integer(2), false, -5, 1, pool_key).unwrap();
            let expected = SqrtPrice::from_integer(1);
            assert_eq!(result, expected);
            assert_eq!(from_tick, Some((0, true)));
        }
        // other direction
        {
            let (result, from_tick) =
                tickmap.get_closer_limit(SqrtPrice::from_scale(1, 1), false, -100, 10, pool_key).unwrap();
            let expected = SqrtPrice::from_scale(1, 1);
            assert_eq!(result, expected);
            assert_eq!(from_tick, None);
        }
    }

    #[test]
    fn test_flip() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();
        //zero
        {
            let index = 0;

            assert!(!tickmap.get(index, 1, pool_key));
            tickmap.flip(true, index, 1, pool_key);
            assert!(tickmap.get(index, 1, pool_key));
            tickmap.flip(false, index, 1, pool_key);
            assert!(!tickmap.get(index, 1, pool_key));
        }
        // small
        {
            let index = 7;

            assert!(!tickmap.get(index, 1, pool_key));
            tickmap.flip(true, index, 1, pool_key);
            assert!(tickmap.get(index, 1, pool_key));
            tickmap.flip(false, index, 1, pool_key);
            assert!(!tickmap.get(index, 1, pool_key));
        }
        // big
        {
            let index = MAX_TICK - 1;

            assert!(!tickmap.get(index, 1, pool_key));
            tickmap.flip(true, index, 1, pool_key);
            assert!(tickmap.get(index, 1, pool_key));
            tickmap.flip(false, index, 1, pool_key);
            assert!(!tickmap.get(index, 1, pool_key));
        }
        // negative
        {
            let index = MAX_TICK - 40;

            assert!(!tickmap.get(index, 1, pool_key));
            tickmap.flip(true, index, 1, pool_key);
            assert!(tickmap.get(index, 1, pool_key));
            tickmap.flip(false, index, 1, pool_key);
            assert!(!tickmap.get(index, 1, pool_key));
        }
        // tick spacing
        {
            let index = 20000;
            let tick_spacing = 1000;

            assert!(!tickmap.get(index, tick_spacing, pool_key));
            tickmap.flip(true, index, tick_spacing, pool_key);
            assert!(tickmap.get(index, tick_spacing, pool_key));
            tickmap.flip(false, index, tick_spacing, pool_key);
            assert!(!tickmap.get(index, tick_spacing, pool_key));
        }
    }

    #[test]
    fn test_next_initialized_simple() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();
        tickmap.flip(true, 5, 1, pool_key);
        assert_eq!(tickmap.next_initialized(0, 1, pool_key), Some(5));
    }

    #[test]
    fn test_next_initialized_multiple() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();
        tickmap.flip(true, 50, 10, pool_key);
        tickmap.flip(true, 100, 10, pool_key);
        assert_eq!(tickmap.next_initialized(0, 10, pool_key), Some(50));
        assert_eq!(tickmap.next_initialized(50, 10, pool_key), Some(100));
    }

    #[test]
    fn test_next_initialized_current_is_last() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();
        tickmap.flip(true, 0, 10, pool_key);
        assert_eq!(tickmap.next_initialized(0, 10, pool_key), None);
    }

    #[test]
    fn test_next_initialized_just_below_limit() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        tickmap.flip(true, 0, 1, pool_key);
        assert_eq!(
            tickmap.next_initialized(-TICK_SEARCH_RANGE, 1, pool_key),
            Some(0)
        );
    }

    #[test]
    fn test_next_initialized_at_limit() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        tickmap.flip(true, 0, 1, pool_key);
        assert_eq!(
            tickmap.next_initialized(-TICK_SEARCH_RANGE - 1, 1, pool_key),
            None
        );
    }

    #[test]
    fn test_next_initialized_further_than_limit() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        tickmap.flip(true, MAX_TICK - 10, 1, pool_key);
        assert_eq!(tickmap.next_initialized(-MAX_TICK + 1, 1, pool_key), None);
    }

    #[test]
    fn test_next_initialized_hitting_the_limit() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        assert_eq!(tickmap.next_initialized(MAX_TICK - 22, 4, pool_key), None);
    }

    #[test]
    fn test_next_initialized_already_at_limit() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        assert_eq!(tickmap.next_initialized(MAX_TICK - 2, 4, pool_key), None);
    }

    #[test]
    fn test_next_initialized_at_pos_63() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        tickmap.flip(true, MAX_TICK - 63, 1, pool_key);
        assert_eq!(
            tickmap.next_initialized(MAX_TICK - 128, 1, pool_key),
            Some(MAX_TICK - 63)
        );
    }

    #[test]
    fn test_prev_initialized_simple() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        tickmap.flip(true, -5, 1, pool_key);
        assert_eq!(tickmap.prev_initialized(0, 1, pool_key), Some(-5));
    }

    #[test]
    fn test_prev_initialized_multiple() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        tickmap.flip(true, -50, 10, pool_key);
        tickmap.flip(true, -100, 10, pool_key);
        assert_eq!(tickmap.prev_initialized(0, 10, pool_key), Some(-50));
        assert_eq!(tickmap.prev_initialized(-50, 10, pool_key), Some(-50));
    }

    #[test]
    fn test_prev_initialized_current_is_last() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        tickmap.flip(true, 0, 10, pool_key);
        assert_eq!(tickmap.prev_initialized(0, 10, pool_key), Some(0));
    }

    #[test]
    fn test_prev_initialized_next_is_last() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        tickmap.flip(true, 10, 10, pool_key);
        assert_eq!(tickmap.prev_initialized(0, 10, pool_key), None);
    }

    #[test]
    fn test_prev_initialized_just_below_limit() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        tickmap.flip(true, 0, 1, pool_key);
        assert_eq!(
            tickmap.prev_initialized(TICK_SEARCH_RANGE, 1, pool_key),
            Some(0)
        );
    }

    #[test]
    fn test_prev_initialized_at_limit() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        tickmap.flip(true, 0, 1, pool_key);
        assert_eq!(
            tickmap.prev_initialized(TICK_SEARCH_RANGE + 1, 1, pool_key),
            None
        );
    }

    #[test]
    fn test_prev_initialized_farther_than_limit() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        tickmap.flip(true, -MAX_TICK + 1, 1, pool_key);
        assert_eq!(tickmap.prev_initialized(MAX_TICK - 1, 1, pool_key), None);
    }

    #[test]
    fn test_prev_initialized_at_pos_63() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        let tickmap = &mut Tickmap::default();

        tickmap.flip(true, -MAX_TICK + 63, 1, pool_key);
        assert_eq!(
            tickmap.prev_initialized(-MAX_TICK + 128, 1, pool_key),
            Some(-MAX_TICK + 63)
        );
    }

    #[test]
    fn test_get_search_limit() {
        // Simple up
        {
            let result = get_search_limit(0, 1, true);
            assert_eq!(result, TICK_SEARCH_RANGE);
        }
        // Simple down
        {
            let result = get_search_limit(0, 1, false);
            assert_eq!(result, -TICK_SEARCH_RANGE);
        }
        // Less simple up
        {
            let start = 60;
            let step = 12;
            let result = get_search_limit(start, step, true);
            let expected = start + TICK_SEARCH_RANGE * step as i32;
            assert_eq!(result, expected);
        }
        // Less simple down
        {
            let start = 60;
            let step = 12;
            let result = get_search_limit(start, step, false);
            let expected = start - TICK_SEARCH_RANGE * step as i32;
            assert_eq!(result, expected);
        }
        // Up to price limit
        {
            let step = 5u16;
            let result = get_search_limit(MAX_TICK - 22, step, true);
            let expected = MAX_TICK - 3;
            assert_eq!(result, expected);
        }
        // At the price limit
        {
            let step = 5u16;
            let result = get_search_limit(MAX_TICK - 3, step, true);
            let expected = MAX_TICK - 3;
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_next_and_prev_initialized() {
        let token_0: ActorId = ActorId::from([0x01; 32]);
        let token_1: ActorId = ActorId::from([0x02; 32]);
        let fee_tier: FeeTier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let pool_key: PoolKey = PoolKey::new(token_0, token_1, fee_tier).unwrap();

        // initalized edges
        {
            for spacing in 1..=10 {
                let tickmap = &mut Tickmap::default();

                let max_index = get_max_tick(spacing as u16);
                let min_index = get_min_tick(spacing as u16);

                tickmap.flip(true, max_index, spacing as u16, pool_key);
                tickmap.flip(true, min_index, spacing as u16, pool_key);

                let tick_edge_diff = TICK_SEARCH_RANGE / spacing * spacing;

                let prev =
                    tickmap.prev_initialized(min_index + tick_edge_diff, spacing as u16, pool_key);
                let next =
                    tickmap.next_initialized(max_index - tick_edge_diff, spacing as u16, pool_key);

                assert_eq!((prev.is_some(), next.is_some()), (true, true));

                // cleanup
                {
                    tickmap.flip(false, max_index, spacing as u16, pool_key);
                    tickmap.flip(false, min_index, spacing as u16, pool_key);
                }
            }
        }
        // unintalized edges
        for spacing in 1..=1000 {
            let tickmap = &mut Tickmap::default();
            let max_index = get_max_tick(spacing as u16);
            let min_index = get_min_tick(spacing as u16);
            let tick_edge_diff = TICK_SEARCH_RANGE / spacing * spacing;

            let prev =
                tickmap.prev_initialized(min_index + tick_edge_diff, spacing as u16, pool_key);
            let next =
                tickmap.next_initialized(max_index - tick_edge_diff, spacing as u16, pool_key);

            assert_eq!((prev.is_some(), next.is_some()), (false, false));
        }
    }
}
