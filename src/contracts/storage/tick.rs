extern crate alloc;

use super::Pool;
use decimal::*;
use math::types::{
    fee_growth::FeeGrowth, liquidity::Liquidity, sqrt_price::calculate_sqrt_price,
    sqrt_price::SqrtPrice,
};
use sails_rs::prelude::*;
use traceable_result::*;

pub const LIQUIDITY_TICK_LIMIT: usize = 21544;
pub const POSITION_TICK_LIMIT: usize = 17872;

#[derive(Debug, Copy, Clone, Decode, Encode, PartialEq, Eq, TypeInfo)]
pub struct Tick {
    pub index: i32,
    pub sign: bool,
    pub liquidity_change: Liquidity,
    pub liquidity_gross: Liquidity,
    pub sqrt_price: SqrtPrice,
    pub fee_growth_outside_x: FeeGrowth,
    pub fee_growth_outside_y: FeeGrowth,
    pub seconds_outside: u64,
}

impl Default for Tick {
    fn default() -> Self {
        Tick {
            index: 0i32,
            sign: false,
            liquidity_change: Liquidity::new(U256::from(0)),
            liquidity_gross: Liquidity::new(U256::from(0)),
            sqrt_price: SqrtPrice::from_integer(1),
            fee_growth_outside_x: FeeGrowth::new(0),
            fee_growth_outside_y: FeeGrowth::new(0),
            seconds_outside: 0u64,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct LiquidityTick {
    pub index: i32,
    pub liquidity_change: Liquidity,
    pub sign: bool,
}

impl From<&Tick> for LiquidityTick {
    fn from(tick: &Tick) -> Self {
        Self {
            index: tick.index,
            liquidity_change: tick.liquidity_change,
            sign: tick.sign,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct PositionTick {
    pub index: i32,
    pub fee_growth_outside_x: FeeGrowth,
    pub fee_growth_outside_y: FeeGrowth,
    pub seconds_outside: u64,
}

impl From<&Tick> for PositionTick {
    fn from(tick: &Tick) -> Self {
        Self {
            index: tick.index,
            fee_growth_outside_x: tick.fee_growth_outside_x,
            fee_growth_outside_y: tick.fee_growth_outside_y,
            seconds_outside: tick.seconds_outside,
        }
    }
}

impl Tick {
    pub fn create(index: i32, pool: &Pool, current_timestamp: u64) -> Self {
        let below_current_tick = index <= pool.current_tick_index;

        Self {
            index,
            sign: true,
            sqrt_price: calculate_sqrt_price(index).unwrap(),
            fee_growth_outside_x: match below_current_tick {
                true => pool.fee_growth_global_x,
                false => FeeGrowth::new(0),
            },
            fee_growth_outside_y: match below_current_tick {
                true => pool.fee_growth_global_y,
                false => FeeGrowth::new(0),
            },
            seconds_outside: match below_current_tick {
                true => current_timestamp - pool.start_timestamp,
                false => 0,
            },
            ..Self::default()
        }
    }

    pub fn cross(&mut self, pool: &mut Pool, current_timestamp: u64) -> TrackableResult<()> {
        self.fee_growth_outside_x = pool
            .fee_growth_global_x
            .unchecked_sub(self.fee_growth_outside_x);
        self.fee_growth_outside_y = pool
            .fee_growth_global_y
            .unchecked_sub(self.fee_growth_outside_y);

        let seconds_passed: u64 = current_timestamp
            .checked_sub(pool.start_timestamp)
            .ok_or_else(|| err!("current_timestamp - pool.start_timestamp underflow"))?;
        self.seconds_outside = seconds_passed.wrapping_sub(self.seconds_outside);

        pool.last_timestamp = current_timestamp;

        // When going to higher tick net_liquidity should be added and for going lower subtracted
        if (pool.current_tick_index >= self.index) ^ self.sign {
            pool.liquidity = pool
                .liquidity
                .checked_add(self.liquidity_change)
                .map_err(|_| err!("pool.liquidity + tick.liquidity_change overflow"))?;
        } else {
            pool.liquidity = pool
                .liquidity
                .checked_sub(self.liquidity_change)
                .map_err(|_| err!("pool.liquidity - tick.liquidity_change underflow"))?
        }

        Ok(())
    }

    pub fn update(
        &mut self,
        liquidity_delta: Liquidity,
        max_liquidity_per_tick: Liquidity,
        is_upper: bool,
        is_deposit: bool,
    ) -> TrackableResult<()> {
        self.liquidity_gross = self.calculate_new_liquidity_gross(
            is_deposit,
            liquidity_delta,
            max_liquidity_per_tick,
        )?;

        self.update_liquidity_change(liquidity_delta, is_deposit ^ is_upper);
        Ok(())
    }

    fn update_liquidity_change(&mut self, liquidity_delta: Liquidity, add: bool) {
        if self.sign ^ add {
            if { self.liquidity_change } > liquidity_delta {
                self.liquidity_change -= liquidity_delta;
            } else {
                self.liquidity_change = liquidity_delta - self.liquidity_change;
                self.sign = !self.sign;
            }
        } else {
            self.liquidity_change += liquidity_delta;
        }
    }

    fn calculate_new_liquidity_gross(
        self,
        sign: bool,
        liquidity_delta: Liquidity,
        max_liquidity_per_tick: Liquidity,
    ) -> TrackableResult<Liquidity> {
        // validate in decrease liquidity case
        if !sign && { self.liquidity_gross } < liquidity_delta {
            return Err(err!("InvalidTickLiquidity"));
        }
        let new_liquidity = match sign {
            true => self
                .liquidity_gross
                .checked_add(liquidity_delta)
                .map_err(|_| err!("tick add liquidity overflow")),
            false => self
                .liquidity_gross
                .checked_sub(liquidity_delta)
                .map_err(|_| err!("tick sun liquidity overflow")),
        }?;
        // validate in increase liquidity case
        if sign && new_liquidity >= max_liquidity_per_tick {
            return Err(err!("InvalidTickLiquidity"));
        }

        Ok(new_liquidity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decimal::{Decimal, Factories};
    use math::clamm::calculate_max_liquidity_per_tick;

    #[test]
    fn test_cross() {
        {
            let mut pool = Pool {
                fee_growth_global_x: FeeGrowth::new(45),
                fee_growth_global_y: FeeGrowth::new(35),
                liquidity: Liquidity::from_integer(U256::from(4)),
                last_timestamp: 15,
                start_timestamp: 4,
                current_tick_index: 7,
                ..Default::default()
            };
            let mut tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(30),
                fee_growth_outside_y: FeeGrowth::new(25),
                index: 3,
                seconds_outside: 5,
                liquidity_change: Liquidity::from_integer(U256::from(1)),
                ..Default::default()
            };
            let result_pool = Pool {
                fee_growth_global_x: FeeGrowth::new(45),
                fee_growth_global_y: FeeGrowth::new(35),
                liquidity: Liquidity::from_integer(U256::from(5)),
                last_timestamp: 315360015,
                start_timestamp: 4,
                current_tick_index: 7,
                ..Default::default()
            };
            let result_tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(15),
                fee_growth_outside_y: FeeGrowth::new(10),
                index: 3,
                seconds_outside: 315360006,
                liquidity_change: Liquidity::from_integer(U256::from(1)),
                ..Default::default()
            };
            tick.cross(&mut pool, 315360015).ok();
            assert_eq!(tick, result_tick);
            assert_eq!(pool, result_pool);
        }
        {
            // let mut pool = Pool {
            let mut pool = Pool {
                fee_growth_global_x: FeeGrowth::new(68),
                fee_growth_global_y: FeeGrowth::new(59),
                liquidity: Liquidity::new(U256::from(0)),
                last_timestamp: 9,
                start_timestamp: 34,
                current_tick_index: 4,
                ..Default::default()
            };
            let mut tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(42),
                fee_growth_outside_y: FeeGrowth::new(14),
                index: 9,
                seconds_outside: 41,
                liquidity_change: Liquidity::new(U256::from(0)),
                ..Default::default()
            };
            // let result_pool = Pool {
            let result_pool = Pool {
                fee_growth_global_x: FeeGrowth::new(68),
                fee_growth_global_y: FeeGrowth::new(59),
                liquidity: Liquidity::new(U256::from(0)),
                last_timestamp: 315360000,
                start_timestamp: 34,
                current_tick_index: 4,
                ..Default::default()
            };
            // let result_tick = Tick {
            let result_tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(26),
                fee_growth_outside_y: FeeGrowth::new(45),
                index: 9,
                seconds_outside: 315359925,
                liquidity_change: Liquidity::from_integer(U256::from(0)),
                ..Default::default()
            };

            tick.cross(&mut pool, 315360000).ok();
            assert_eq!(tick, result_tick);
            assert_eq!(pool, result_pool);
        }
        // fee_growth_outside should underflow
        {
            let mut pool = Pool {
                fee_growth_global_x: FeeGrowth::new(3402),
                fee_growth_global_y: FeeGrowth::new(3401),
                liquidity: Liquidity::from_integer(U256::from(14)),
                last_timestamp: 9,
                start_timestamp: 15,
                current_tick_index: 9,
                ..Default::default()
            };
            let mut tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(26584),
                fee_growth_outside_y: FeeGrowth::new(1256588),
                index: 45,
                seconds_outside: 74,
                liquidity_change: Liquidity::new(U256::from(10)),
                ..Default::default()
            };
            let result_pool = Pool {
                fee_growth_global_x: FeeGrowth::new(3402),
                fee_growth_global_y: FeeGrowth::new(3401),
                liquidity: Liquidity::new(U256::from(1399990)),
                last_timestamp: 31536000,
                start_timestamp: 15,
                current_tick_index: 9,
                ..Default::default()
            };
            let result_tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(340282366920938463463374607431768188274u128),
                fee_growth_outside_y: FeeGrowth::new(340282366920938463463374607431766958269u128),
                index: 45,
                seconds_outside: 31535911,
                liquidity_change: Liquidity::new(U256::from(10)),
                ..Default::default()
            };

            tick.cross(&mut pool, 31536000).ok();
            assert_eq!(tick, result_tick);
            assert_eq!(pool, result_pool);
        }
        // seconds_per_liquidity_outside should underflow
        {
            let mut pool = Pool {
                fee_growth_global_x: FeeGrowth::new(145),
                fee_growth_global_y: FeeGrowth::new(364),
                liquidity: Liquidity::new(U256::from(14)),
                last_timestamp: 16,
                start_timestamp: 15,
                current_tick_index: 9,
                ..Default::default()
            };
            let mut tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(99),
                fee_growth_outside_y: FeeGrowth::new(256),
                index: 45,
                seconds_outside: 74,
                liquidity_change: Liquidity::new(U256::from(10)),
                ..Default::default()
            };
            let result_pool = Pool {
                fee_growth_global_x: FeeGrowth::new(145),
                fee_growth_global_y: FeeGrowth::new(364),
                liquidity: Liquidity::new(U256::from(4)),
                last_timestamp: 315360000,
                start_timestamp: 15,
                current_tick_index: 9,
                ..Default::default()
            };
            let result_tick = Tick {
                fee_growth_outside_x: FeeGrowth::new(46),
                fee_growth_outside_y: FeeGrowth::new(108),
                index: 45,
                seconds_outside: 315359911,
                liquidity_change: Liquidity::new(U256::from(10)),
                ..Default::default()
            };

            tick.cross(&mut pool, 315360000).ok();
            assert_eq!(tick, result_tick);
            assert_eq!(pool, result_pool);
        }
    }

    #[test]
    fn test_update_liquidity_change() {
        // update when tick sign and sign of liquidity change are the same
        {
            let mut tick = Tick {
                sign: true,
                liquidity_change: Liquidity::from_integer(2),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(3);
            let add = true;
            tick.update_liquidity_change(liquidity_delta, add);

            assert!(tick.sign);

            assert_eq!({ tick.liquidity_change }, Liquidity::from_integer(5));
        }
        {
            let mut tick = Tick {
                sign: false,
                liquidity_change: Liquidity::from_integer(2),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(3);
            let add = false;
            tick.update_liquidity_change(liquidity_delta, add);

            assert!(!tick.sign);

            assert_eq!({ tick.liquidity_change }, Liquidity::from_integer(5));
        }
        // update when tick sign and sign of liquidity change are different
        {
            let mut tick = Tick {
                sign: true,
                liquidity_change: Liquidity::from_integer(2),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(3);
            let add = false;
            tick.update_liquidity_change(liquidity_delta, add);

            assert!(!tick.sign);

            assert_eq!({ tick.liquidity_change }, Liquidity::from_integer(1));
        }
        {
            let mut tick = Tick {
                sign: false,
                liquidity_change: Liquidity::from_integer(2),
                ..Default::default()
            };
            let liquidity_delta = Liquidity::from_integer(3);
            let add = true;
            tick.update_liquidity_change(liquidity_delta, add);

            assert!(tick.sign);

            assert_eq!({ tick.liquidity_change }, Liquidity::from_integer(1));
        }
    }

    #[test]
    fn test_update() {
        let max_liquidity = Liquidity::max_instance();
        {
            let mut tick = Tick {
                index: 0,
                sign: true,
                liquidity_change: Liquidity::from_integer(2),
                liquidity_gross: Liquidity::from_integer(2),
                fee_growth_outside_x: FeeGrowth::from_integer(2),
                fee_growth_outside_y: FeeGrowth::from_integer(2),
                ..Default::default()
            };
            let liquidity_delta: Liquidity = Liquidity::from_integer(1);
            let is_upper: bool = false;
            let is_deposit: bool = true;

            tick.update(liquidity_delta, max_liquidity, is_upper, is_deposit)
                .unwrap();

            assert!(tick.sign);

            assert_eq!({ tick.liquidity_change }, Liquidity::from_integer(3));
            assert_eq!({ tick.liquidity_gross }, Liquidity::from_integer(3));
            assert_eq!({ tick.fee_growth_outside_x }, FeeGrowth::from_integer(2));
            assert_eq!({ tick.fee_growth_outside_y }, FeeGrowth::from_integer(2));
        }
        {
            let mut tick = Tick {
                index: 5,
                sign: true,
                liquidity_change: Liquidity::from_integer(3),
                liquidity_gross: Liquidity::from_integer(7),
                fee_growth_outside_x: FeeGrowth::from_integer(13),
                fee_growth_outside_y: FeeGrowth::from_integer(11),
                ..Default::default()
            };
            let liquidity_delta: Liquidity = Liquidity::from_integer(1);
            let is_upper: bool = true;
            let is_deposit: bool = true;

            tick.update(liquidity_delta, max_liquidity, is_upper, is_deposit)
                .unwrap();

            assert!(tick.sign);

            assert_eq!({ tick.liquidity_change }, Liquidity::from_integer(2));
            assert_eq!({ tick.liquidity_gross }, Liquidity::from_integer(8));
            assert_eq!({ tick.fee_growth_outside_x }, FeeGrowth::from_integer(13));
            assert_eq!({ tick.fee_growth_outside_y }, FeeGrowth::from_integer(11));
        }
        // exceed max tick liquidity
        {
            let mut tick = Tick {
                // index: 5,
                sign: true,
                liquidity_change: Liquidity::from_integer(100_000),
                liquidity_gross: Liquidity::from_integer(100_000),
                fee_growth_outside_x: FeeGrowth::from_integer(1000),
                fee_growth_outside_y: FeeGrowth::from_integer(1000),
                ..Default::default()
            };

            let max_liquidity_per_tick = calculate_max_liquidity_per_tick(1);
            let liquidity_delta = max_liquidity_per_tick + Liquidity::new(U256::from(1));
            let result = tick.update(liquidity_delta, max_liquidity_per_tick, false, true);
            assert!(result.is_err());
        }
    }
}
