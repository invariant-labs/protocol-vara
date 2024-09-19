use crate::types::liquidity::Liquidity;
use decimal::*;
use sails_rs::prelude::*;
use traceable_result::*;

#[decimal(25, U256)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Encode, Decode, TypeInfo)]
pub struct SecondsPerLiquidity(pub u128);

impl SecondsPerLiquidity {
    pub fn unchecked_add(self, other: SecondsPerLiquidity) -> SecondsPerLiquidity {
        if other.get() > SecondsPerLiquidity::max_instance().get() - self.get() {
            SecondsPerLiquidity::new(
                (other.get() - (SecondsPerLiquidity::max_instance().get() - self.get())) - 1,
            )
        } else {
            SecondsPerLiquidity::new(self.get() + other.get())
        }
    }

    pub fn unchecked_sub(self, other: SecondsPerLiquidity) -> SecondsPerLiquidity {
        if other.get() > self.get() {
            SecondsPerLiquidity::new(
                SecondsPerLiquidity::max_instance().get() - (other.get() - self.get()) + 1,
            )
        } else {
            SecondsPerLiquidity::new(self.get() - other.get())
        }
    }
    pub fn calculate_seconds_per_liquidity_global(
        liquidity: Liquidity,
        current_timestamp: u64,
        last_timestamp: u64,
    ) -> TrackableResult<Self> {
        if current_timestamp <= last_timestamp {
            return Err(err!("current_timestamp > last_timestamp failed"));
        }
        let delta_time = current_timestamp - last_timestamp;

        Ok(Self::new(
            Self::checked_from_value(
                Self::from_value::<U256, u128>(
                    u128::from(delta_time)
                        .checked_mul(Self::one().cast())
                        .ok_or_else(|| err!(TrackableError::MUL))?
                        .checked_mul(Liquidity::one().cast())
                        .ok_or_else(|| err!(TrackableError::MUL))?,
                )
                .checked_div(liquidity.get())
                .ok_or_else(|| err!(TrackableError::DIV))?,
            )
            .map_err(|_| err!(TrackableError::cast::<u128>().as_str()))?,
        ))
    }
}

pub fn calculate_seconds_per_liquidity_inside(
    tick_lower: i32,
    tick_upper: i32,
    tick_current: i32,
    tick_lower_seconds_per_liquidity_outside: SecondsPerLiquidity,
    tick_upper_seconds_per_liquidity_outside: SecondsPerLiquidity,
    pool_seconds_per_liquidity_global: SecondsPerLiquidity,
) -> TrackableResult<SecondsPerLiquidity> {
    let current_above_lower = tick_current >= tick_lower;
    let current_below_upper = tick_current < tick_upper;

    let seconds_per_liquidity_below = if current_above_lower {
        tick_lower_seconds_per_liquidity_outside
    } else {
        pool_seconds_per_liquidity_global.unchecked_sub(tick_lower_seconds_per_liquidity_outside)
    };

    let seconds_per_liquidity_above = if current_below_upper {
        tick_upper_seconds_per_liquidity_outside
    } else {
        pool_seconds_per_liquidity_global.unchecked_sub(tick_upper_seconds_per_liquidity_outside)
    };

    Ok(pool_seconds_per_liquidity_global
        .unchecked_sub(seconds_per_liquidity_below)
        .unchecked_sub(seconds_per_liquidity_above))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unchecked_add() {
        {
            let one = SecondsPerLiquidity::new(1);
            let max = SecondsPerLiquidity::max_instance();
            let expected_result = SecondsPerLiquidity::new(0);
            let result = max.unchecked_add(one);
            assert_eq!(result, expected_result);
        }
        {
            let max = SecondsPerLiquidity::max_instance();
            let other = SecondsPerLiquidity::new(1000000);
            let expected_result = SecondsPerLiquidity::new(999999);
            let max_other = max.unchecked_add(other);
            let other_max = other.unchecked_add(max);
            assert_eq!(max_other, expected_result);
            assert_eq!(other_max, expected_result);
        }
    }
    #[test]
    fn test_unchecked_sub() {
        {
            let one = SecondsPerLiquidity::new(1);
            let two = SecondsPerLiquidity::new(2);
            let expected_result = SecondsPerLiquidity::max_instance();
            let result = one.unchecked_sub(two);
            assert_eq!(result, expected_result);
        }
        {
            let max = SecondsPerLiquidity::max_instance();
            let other = SecondsPerLiquidity::new(1000000);
            let expected_result = SecondsPerLiquidity::new(1000001);
            let max_other = max.unchecked_sub(other);
            let other_max = other.unchecked_sub(max);
            assert_eq!(
                max_other,
                SecondsPerLiquidity::new(340282366920938463463374607431767211455u128)
            );
            assert_eq!(other_max, expected_result);
        }
    }

    #[test]
    fn test_domain_calculate_seconds_per_liquidity_global() {
        // current_timestamp <= last_timestamp
        {
            let liquidity = Liquidity::new(U256::from(1));
            let current_timestamp = 0;
            let last_timestamp = 100;
            let (_, cause, stack) = SecondsPerLiquidity::calculate_seconds_per_liquidity_global(
                liquidity,
                current_timestamp,
                last_timestamp,
            )
            .unwrap_err()
            .get();
            assert_eq!(cause, "current_timestamp > last_timestamp failed");
            assert_eq!(stack.len(), 1);
        }
        // L == 0
        {
            let liquidity = Liquidity::new(U256::from(0));
            let current_timestamp = 1;
            let last_timestamp = 0;
            let (_, cause, stack) = SecondsPerLiquidity::calculate_seconds_per_liquidity_global(
                liquidity,
                current_timestamp,
                last_timestamp,
            )
            .unwrap_err()
            .get();
            assert_eq!(cause, "division overflow or division by zero");
            assert_eq!(stack.len(), 1);
        }
        // min value
        {
            let liquidity = Liquidity::max_instance();
            let current_timestamp = 1;
            let last_timestamp = 0;
            let seconds_per_liquidity =
                SecondsPerLiquidity::calculate_seconds_per_liquidity_global(
                    liquidity,
                    current_timestamp,
                    last_timestamp,
                )
                .unwrap();
            assert_eq!(seconds_per_liquidity.get(), 0);
        }
        // max value
        {
            let liquidity = Liquidity::new(U256::from(1));
            let current_timestamp = 315360000;
            let last_timestamp = 0;
            let result = SecondsPerLiquidity::calculate_seconds_per_liquidity_global(
                liquidity,
                current_timestamp,
                last_timestamp,
            )
            .unwrap();
            assert_eq!(
                result,
                SecondsPerLiquidity::new(315360000000000000000000000000000000000u128)
            );
        }
        // max value outside domain
        {
            let liquidity = Liquidity::new(U256::from(1));
            let current_timestamp = u64::MAX;
            let last_timestamp = 0;
            let (_, cause, stack) = SecondsPerLiquidity::calculate_seconds_per_liquidity_global(
                liquidity,
                current_timestamp,
                last_timestamp,
            )
            .unwrap_err()
            .get();
            assert_eq!(cause, "multiplication overflow");
            assert_eq!(stack.len(), 1);
        }

        let one_liquidity = Liquidity::new(U256::from(1));
        let max_liquidity = Liquidity::max_instance();
        let max_delta_time = 315360000_u64;
        // max time, one liq
        {
            let result = SecondsPerLiquidity::calculate_seconds_per_liquidity_global(
                one_liquidity,
                max_delta_time,
                0,
            )
            .unwrap();
            assert_eq!(
                result,
                SecondsPerLiquidity::new(315360000000000000000000000000000000000u128)
            );
        }
        // big liquidity
        {
            let result = SecondsPerLiquidity::calculate_seconds_per_liquidity_global(
                max_liquidity,
                max_delta_time,
                0,
            )
            .unwrap();
            assert_eq!(result, SecondsPerLiquidity::new(0))
        }
        // min time max liq
        {
            let result = SecondsPerLiquidity::calculate_seconds_per_liquidity_global(
                max_liquidity,
                max_delta_time,
                max_delta_time - 1,
            )
            .unwrap();
            assert_eq!(result, SecondsPerLiquidity::new(0))
        }
        // min time and min liq
        {
            let result = SecondsPerLiquidity::calculate_seconds_per_liquidity_global(
                one_liquidity,
                max_delta_time,
                max_delta_time - 1,
            )
            .unwrap();
            assert_eq!(
                result,
                SecondsPerLiquidity::new(1000000000000000000000000000000u128)
            )
        }
    }

    #[test]
    fn test_calculate_seconds_per_liquidity_inside() {
        // upper tick
        let tick_lower_index = 0;
        let tick_lower_seconds_per_liquidity_outside = SecondsPerLiquidity::new(3012300000u128);

        // lower tick
        let tick_upper_index = 10;
        let tick_upper_seconds_per_liquidity_outside = SecondsPerLiquidity::new(2030400000u128);

        // pool
        let pool_seconds_per_liquidity_global = SecondsPerLiquidity::new(0);

        {
            let pool_current_tick_index = -10;

            let seconds_per_liquidity_inside = calculate_seconds_per_liquidity_inside(
                tick_lower_index,
                tick_upper_index,
                pool_current_tick_index,
                tick_lower_seconds_per_liquidity_outside,
                tick_upper_seconds_per_liquidity_outside,
                pool_seconds_per_liquidity_global,
            );
            assert_eq!(seconds_per_liquidity_inside.unwrap().get(), 981900000);
        }
        {
            let pool_current_tick_index = 0;

            let seconds_per_liquidity_inside = calculate_seconds_per_liquidity_inside(
                tick_lower_index,
                tick_upper_index,
                pool_current_tick_index,
                tick_lower_seconds_per_liquidity_outside,
                tick_upper_seconds_per_liquidity_outside,
                pool_seconds_per_liquidity_global,
            );
            assert_eq!(
                seconds_per_liquidity_inside.unwrap().get(),
                340282366920938463463374607426725511456u128
            );
        }
        {
            let tick_lower_seconds_per_liquidity_outside = SecondsPerLiquidity::new(2012333200);
            let tick_upper_seconds_per_liquidity_outside = SecondsPerLiquidity::new(3012333310u128);
            let pool_current_tick_index = 20;

            let seconds_per_liquidity_inside = calculate_seconds_per_liquidity_inside(
                tick_lower_index,
                tick_upper_index,
                pool_current_tick_index,
                tick_lower_seconds_per_liquidity_outside,
                tick_upper_seconds_per_liquidity_outside,
                pool_seconds_per_liquidity_global,
            );
            assert_eq!(seconds_per_liquidity_inside.unwrap().get(), 1000000110);
        }
        {
            let tick_lower_seconds_per_liquidity_outside =
                SecondsPerLiquidity::new(201233320000u128);
            let tick_upper_seconds_per_liquidity_outside =
                SecondsPerLiquidity::new(301233331000u128);
            let pool_current_tick_index = 20;

            let seconds_per_liquidity_inside = calculate_seconds_per_liquidity_inside(
                tick_lower_index,
                tick_upper_index,
                pool_current_tick_index,
                tick_lower_seconds_per_liquidity_outside,
                tick_upper_seconds_per_liquidity_outside,
                pool_seconds_per_liquidity_global,
            );
            assert_eq!(
                seconds_per_liquidity_inside.unwrap().get(),
                100000011000u128
            );
        }
        {
            let tick_lower_seconds_per_liquidity_outside =
                SecondsPerLiquidity::new(201233320000u128);
            let tick_upper_seconds_per_liquidity_outside =
                SecondsPerLiquidity::new(301233331000u128);
            let pool_current_tick_index = -20;

            let seconds_per_liquidity_inside = calculate_seconds_per_liquidity_inside(
                tick_lower_index,
                tick_upper_index,
                pool_current_tick_index,
                tick_lower_seconds_per_liquidity_outside,
                tick_upper_seconds_per_liquidity_outside,
                pool_seconds_per_liquidity_global,
            );
            assert_eq!(
                seconds_per_liquidity_inside.unwrap().get(),
                340282366920938463463374607331768200456u128
            );
        }
    }
}
