extern crate alloc;

use crate::{FeeTier, InvariantError, PoolKey, Tick};
use decimal::*;
use math::{
    clamm::*,
    fee_growth::FeeGrowth,
    liquidity::Liquidity,
    log::get_tick_at_sqrt_price,
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, check_tick_to_sqrt_price_relationship, SqrtPrice},
    token_amount::TokenAmount,
};
use sails_rtl::{ActorId, Decode, Encode, TypeInfo};
use traceable_result::*;

#[derive(PartialEq, Debug, Clone, Decode, Encode, TypeInfo, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Pool {
    pub liquidity: Liquidity,
    pub sqrt_price: SqrtPrice,
    pub current_tick_index: i32,
    pub fee_growth_global_x: FeeGrowth,
    pub fee_growth_global_y: FeeGrowth,
    pub fee_protocol_token_x: TokenAmount,
    pub fee_protocol_token_y: TokenAmount,
    pub start_timestamp: u64,
    pub last_timestamp: u64,
    pub fee_receiver: ActorId,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UpdatePoolTick {
    NoTick,
    TickInitialized(Tick),
    TickUninitialized(i32),
}

impl Default for Pool {
    fn default() -> Self {
        Self {
            liquidity: Liquidity::default(),
            sqrt_price: SqrtPrice::default(),
            current_tick_index: i32::default(),
            fee_growth_global_x: FeeGrowth::default(),
            fee_growth_global_y: FeeGrowth::default(),
            fee_protocol_token_x: TokenAmount::new(U256::from(0u128)),
            fee_protocol_token_y: TokenAmount::new(U256::from(0u128)),
            start_timestamp: u64::default(),
            last_timestamp: u64::default(),
            fee_receiver: ActorId::from([0x0; 32]),
        }
    }
}

impl Pool {
    pub fn create(
        init_sqrt_price: SqrtPrice,
        init_tick: i32,
        current_timestamp: u64,
        tick_spacing: u16,
        fee_receiver: ActorId,
    ) -> Result<Self, InvariantError> {
        let is_in_relationship =
            check_tick_to_sqrt_price_relationship(init_tick, tick_spacing, init_sqrt_price)
                .map_err(|_| InvariantError::InvalidInitTick)?;

        if !is_in_relationship {
            return Err(InvariantError::InvalidInitSqrtPrice);
        }

        Ok(Self {
            sqrt_price: unwrap!(calculate_sqrt_price(init_tick)),
            current_tick_index: init_tick,
            start_timestamp: current_timestamp,
            last_timestamp: current_timestamp,
            fee_receiver,
            ..Self::default()
        })
    }

    pub fn add_fee(
        &mut self,
        amount: TokenAmount,
        in_x: bool,
        protocol_fee: Percentage,
    ) -> TrackableResult<()> {
        let protocol_fee = amount.big_mul_up(protocol_fee);

        let pool_fee = amount - protocol_fee;

        if (pool_fee.is_zero() && protocol_fee.is_zero()) || self.liquidity.is_zero() {
            return Ok(());
        }

        let fee_growth = ok_or_mark_trace!(FeeGrowth::from_fee(self.liquidity, pool_fee))?;

        if in_x {
            self.fee_growth_global_x = self.fee_growth_global_x.unchecked_add(fee_growth);
            self.fee_protocol_token_x += protocol_fee;
        } else {
            self.fee_growth_global_y = self.fee_growth_global_y.unchecked_add(fee_growth);
            self.fee_protocol_token_y += protocol_fee;
        }
        Ok(())
    }

    pub fn update_liquidity(
        &mut self,
        liquidity_delta: Liquidity,
        liquidity_sign: bool,
        upper_tick: i32,
        lower_tick: i32,
    ) -> TrackableResult<(TokenAmount, TokenAmount)> {
        let (x, y, update_liquidity) = ok_or_mark_trace!(calculate_amount_delta(
            self.current_tick_index,
            self.sqrt_price,
            liquidity_delta,
            liquidity_sign,
            upper_tick,
            lower_tick,
        ))?;

        if !update_liquidity {
            return Ok((x, y));
        }

        if liquidity_sign {
            self.liquidity = self
                .liquidity
                .checked_add(liquidity_delta)
                .map_err(|_| err!("update_liquidity: liquidity + liquidity_delta overflow"))?;
            Ok((x, y))
        } else {
            self.liquidity = self
                .liquidity
                .checked_sub(liquidity_delta)
                .map_err(|_| err!("update_liquidity: liquidity - liquidity_delta underflow"))?;
            Ok((x, y))
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_tick(
        &mut self,
        result: SwapResult,
        swap_limit: SqrtPrice,
        tick: &mut UpdatePoolTick,
        mut remaining_amount: TokenAmount,
        by_amount_in: bool,
        x_to_y: bool,
        current_timestamp: u64,
        protocol_fee: Percentage,
        fee_tier: FeeTier,
    ) -> (TokenAmount, TokenAmount, bool) {
        let mut has_crossed = false;
        let mut total_amount = TokenAmount::new(U256::from(0));

        if UpdatePoolTick::NoTick == *tick || swap_limit != result.next_sqrt_price {
            self.current_tick_index = unwrap!(get_tick_at_sqrt_price(
                result.next_sqrt_price,
                fee_tier.tick_spacing
            ));

            return (total_amount, remaining_amount, has_crossed);
        };

        let is_enough_amount_to_cross = unwrap!(is_enough_amount_to_change_price(
            remaining_amount,
            result.next_sqrt_price,
            self.liquidity,
            fee_tier.fee,
            by_amount_in,
            x_to_y,
        ));

        let tick_index = match tick {
            UpdatePoolTick::TickInitialized(tick) => {
                if !x_to_y || is_enough_amount_to_cross {
                    tick.cross(self, current_timestamp).unwrap();
                    has_crossed = true;
                } else if !remaining_amount.is_zero() {
                    if by_amount_in {
                        unwrap!(self.add_fee(remaining_amount, x_to_y, protocol_fee));
                        total_amount = remaining_amount;
                    }
                    remaining_amount = TokenAmount::new(U256::from(0));
                }

                tick.index
            }
            UpdatePoolTick::TickUninitialized(index) => *index,
            _ => unreachable!(),
        };

        self.current_tick_index = if x_to_y && is_enough_amount_to_cross {
            tick_index - fee_tier.tick_spacing as i32
        } else {
            tick_index
        };

        (total_amount, remaining_amount, has_crossed)
    }

    pub fn withdraw_protocol_fee(&mut self, _pool_key: PoolKey) -> (TokenAmount, TokenAmount) {
        let fee_protocol_token_x = self.fee_protocol_token_x;
        let fee_protocol_token_y = self.fee_protocol_token_y;

        self.fee_protocol_token_x = TokenAmount::default();
        self.fee_protocol_token_y = TokenAmount::default();

        (fee_protocol_token_x, fee_protocol_token_y)
    }
}

#[cfg(test)]
mod tests {
    use math::{sqrt_price::calculate_sqrt_price, MAX_TICK};

    use super::*;

    #[test]
    fn create() {
        let init_tick = 100;
        let current_timestamp = 100;
        let fee_receiver = ActorId::from([1; 32]);
        let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

        let pool = Pool::create(
            init_sqrt_price,
            init_tick,
            current_timestamp,
            1,
            fee_receiver,
        )
        .unwrap();

        assert_eq!(pool.sqrt_price, calculate_sqrt_price(init_tick).unwrap());
        assert_eq!(pool.current_tick_index, init_tick);
        assert_eq!(pool.start_timestamp, current_timestamp);
        assert_eq!(pool.last_timestamp, current_timestamp);
        assert_eq!(pool.fee_receiver, fee_receiver);

        {
            let init_tick = 0;
            let init_sqrt_price =
                calculate_sqrt_price(init_tick).unwrap() + SqrtPrice::new(U128::from(1));
            let tick_spacing = 3;
            let pool = Pool::create(
                init_sqrt_price,
                init_tick,
                current_timestamp,
                tick_spacing,
                fee_receiver,
            )
            .unwrap();
            assert_eq!(pool.current_tick_index, init_tick);
        }
        {
            let init_tick = 2;
            let init_sqrt_price = SqrtPrice::new(U128::from(1000175003749000000000000u128));
            let tick_spacing = 1;
            let pool = Pool::create(
                init_sqrt_price,
                init_tick,
                current_timestamp,
                tick_spacing,
                fee_receiver,
            );
            assert_eq!(pool, Err(InvariantError::InvalidInitSqrtPrice));
            let correct_init_tick = 3;
            let pool = Pool::create(
                init_sqrt_price,
                correct_init_tick,
                current_timestamp,
                tick_spacing,
                fee_receiver,
            )
            .unwrap();
            assert_eq!(pool.current_tick_index, correct_init_tick);
        }
        {
            let init_tick = 0;
            let init_sqrt_price = SqrtPrice::new(U128::from(1000225003749000000000000u128));
            let tick_spacing = 3;
            let pool = Pool::create(
                init_sqrt_price,
                init_tick,
                current_timestamp,
                tick_spacing,
                fee_receiver,
            );
            assert_eq!(pool, Err(InvariantError::InvalidInitSqrtPrice));
            let correct_init_tick = 3;
            let pool = Pool::create(
                init_sqrt_price,
                correct_init_tick,
                current_timestamp,
                tick_spacing,
                fee_receiver,
            )
            .unwrap();
            assert_eq!(pool.current_tick_index, correct_init_tick);
        }
        {
            let init_tick = MAX_TICK;
            let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
            let tick_spacing = 1;
            Pool::create(
                init_sqrt_price,
                init_tick,
                current_timestamp,
                tick_spacing,
                fee_receiver,
            )
            .unwrap();
        }
        {
            let init_tick = MAX_TICK;
            let init_sqrt_price =
                calculate_sqrt_price(init_tick).unwrap() - SqrtPrice::new(U128::from(1));
            let tick_spacing = 1;
            Pool::create(
                init_sqrt_price,
                init_tick,
                current_timestamp,
                tick_spacing,
                fee_receiver,
            )
            .unwrap_err();
        }
        {
            let init_tick = MAX_TICK;
            let init_sqrt_price = SqrtPrice::from_integer(1);
            let tick_spacing = 1;
            Pool::create(
                init_sqrt_price,
                init_tick,
                current_timestamp,
                tick_spacing,
                fee_receiver,
            )
            .unwrap_err();
        }
        {
            let init_tick = MAX_TICK - 1;
            let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
            let tick_spacing = 1;
            let pool = Pool::create(
                init_sqrt_price,
                init_tick,
                current_timestamp,
                tick_spacing,
                fee_receiver,
            )
            .unwrap();
            assert_eq!(pool.current_tick_index, init_tick);
        }
    }

    #[test]
    fn test_add_fee() {
        // fee is set to 20%
        let protocol_fee = Percentage::from_scale(2, 1);
        let pool = Pool {
            liquidity: Liquidity::from_integer(10),
            ..Default::default()
        };
        // in_x
        {
            let mut pool = pool.clone();
            let amount = TokenAmount::from_integer(6);
            pool.add_fee(amount, true, protocol_fee).unwrap();
            assert_eq!({ pool.fee_growth_global_x }, FeeGrowth::from_scale(4, 1));
            assert_eq!({ pool.fee_growth_global_y }, FeeGrowth::from_integer(0));
            assert_eq!(
                { pool.fee_protocol_token_x },
                TokenAmount::new(U256::from(2))
            );
            assert_eq!(
                { pool.fee_protocol_token_y },
                TokenAmount::new(U256::from(0))
            );
        }
        // in_y
        {
            let mut pool = pool.clone();
            let amount = TokenAmount::from_integer(200);
            pool.add_fee(amount, false, protocol_fee).unwrap();
            assert_eq!({ pool.fee_growth_global_x }, FeeGrowth::from_integer(0));
            assert_eq!({ pool.fee_growth_global_y }, FeeGrowth::from_scale(160, 1));
            assert_eq!(
                { pool.fee_protocol_token_x },
                TokenAmount::new(U256::from(0))
            );
            assert_eq!(
                { pool.fee_protocol_token_y },
                TokenAmount::new(U256::from(40))
            );
        }
        // some new comment
        {
            let mut pool = pool;
            let amount = TokenAmount::new(U256::from(1));
            pool.add_fee(amount, true, protocol_fee).unwrap();
            assert_eq!({ pool.fee_growth_global_x }, FeeGrowth::new(U128::from(0)));
            assert_eq!({ pool.fee_growth_global_y }, FeeGrowth::new(U128::from(0)));
            assert_eq!(
                { pool.fee_protocol_token_x },
                TokenAmount::new(U256::from(1))
            );
            assert_eq!(
                { pool.fee_protocol_token_y },
                TokenAmount::new(U256::from(0))
            );
        }
        //DOMAIN
        let max_amount = TokenAmount::max_instance();
        // let min_amount = TokenAmount(1);
        let max_liquidity = Liquidity::max_instance();
        // let min_liquidity = Liquidity::new(1);
        let max_protocol_fee = Percentage::from_integer(1);
        let min_protocol_fee = Percentage::from_integer(0);

        // max fee max amount max liquidity in x
        {
            let mut pool = Pool {
                liquidity: max_liquidity,
                ..Default::default()
            };
            pool.add_fee(max_amount, true, max_protocol_fee).unwrap();
            assert_eq!({ pool.fee_growth_global_x }, FeeGrowth::from_integer(0));
            assert_eq!({ pool.fee_growth_global_y }, FeeGrowth::from_integer(0));
            assert_eq!(
                { pool.fee_protocol_token_x },
                TokenAmount::new(U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap()),
            );
            assert_eq!(
                { pool.fee_protocol_token_y },
                TokenAmount::new(U256::from(0))
            );
        }
        // max fee max amount max liquidity in y
        {
            let mut pool = Pool {
                liquidity: max_liquidity,
                ..Default::default()
            };
            pool.add_fee(max_amount, false, max_protocol_fee).unwrap();
            assert_eq!({ pool.fee_growth_global_x }, FeeGrowth::from_integer(0));
            assert_eq!({ pool.fee_growth_global_y }, FeeGrowth::from_integer(0));
            assert_eq!(
                { pool.fee_protocol_token_x },
                TokenAmount::new(U256::from(0))
            );
            assert_eq!(
                { pool.fee_protocol_token_y },
                TokenAmount::new(U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap()),
            );
        }
        // min fee max amount max liquidity in x
        {
            let mut pool = Pool {
                liquidity: max_liquidity,
                ..Default::default()
            };
            pool.add_fee(max_amount, true, min_protocol_fee).unwrap();
            assert_eq!(
                { pool.fee_growth_global_x },
                FeeGrowth::new(U128::from(1000000000000000000000000000000000u128))
            );
            assert_eq!({ pool.fee_growth_global_y }, FeeGrowth::new(U128::from(0)));
            assert_eq!(
                { pool.fee_protocol_token_x },
                TokenAmount::new(U256::from(0))
            );
            assert_eq!(
                { pool.fee_protocol_token_y },
                TokenAmount::new(U256::from(0))
            );
        }
    }
    #[test]
    fn test_update_liquidity() {
        // Add liquidity
        // current tick between lower tick and upper tick
        {
            let mut pool = Pool {
                liquidity: Liquidity::from_integer(0),
                sqrt_price: SqrtPrice::new(U128::from(1_000_140_000_000_000_000_000_000u128)),
                current_tick_index: 2,
                ..Default::default()
            };

            let liquidity_delta = Liquidity::from_integer(5_000_000);
            let liquidity_sign = true;
            let upper_tick = 3;
            let lower_tick = 0;

            let (x, y) = pool
                .update_liquidity(liquidity_delta, liquidity_sign, upper_tick, lower_tick)
                .unwrap();

            assert_eq!(x, TokenAmount::new(U256::from(51)));
            assert_eq!(y, TokenAmount::new(U256::from(700)));

            assert_eq!(pool.liquidity, liquidity_delta)
        }
        {
            let mut pool = Pool {
                liquidity: Liquidity::from_integer(0),
                sqrt_price: SqrtPrice::new(U128::from(1_000_140_000_000_000_000_000_000_u128)),
                current_tick_index: 2,
                ..Default::default()
            };

            let liquidity_delta = Liquidity::from_integer(5_000_000);
            let liquidity_sign = true;
            let upper_tick = 4;
            let lower_tick = 0;

            let (x, y) = pool
                .update_liquidity(liquidity_delta, liquidity_sign, upper_tick, lower_tick)
                .unwrap();

            assert_eq!(x, TokenAmount::new(U256::from(300)));
            assert_eq!(y, TokenAmount::new(U256::from(700)));
            assert_eq!(pool.liquidity, liquidity_delta)
        }
        // delta liquidity = 0
        // No Change
        {
            {
                let mut pool = Pool {
                    liquidity: Liquidity::from_integer(1),
                    sqrt_price: SqrtPrice::new(U128::from(1_000_140_000_000_000_000_000_000_u128)),
                    current_tick_index: 6,
                    ..Default::default()
                };

                let liquidity_delta = Liquidity::from_integer(12);
                let liquidity_sign = true;
                let upper_tick = 4;
                let lower_tick = 0;

                let (x, y) = pool
                    .update_liquidity(liquidity_delta, liquidity_sign, upper_tick, lower_tick)
                    .unwrap();

                assert_eq!(x, TokenAmount::new(U256::from(0)));
                assert_eq!(y, TokenAmount::new(U256::from(1)));
                assert_eq!(pool.liquidity, Liquidity::from_integer(1))
            }
            {
                let mut pool = Pool {
                    liquidity: Liquidity::from_integer(1),
                    sqrt_price: SqrtPrice::new(U128::from(1_000_140_000_000_000_000_000_000_u128)),
                    current_tick_index: -2,
                    ..Default::default()
                };

                let liquidity_delta = Liquidity::from_integer(12);
                let liquidity_sign = true;
                let upper_tick = 4;
                let lower_tick = 0;

                let (x, y) = pool
                    .update_liquidity(liquidity_delta, liquidity_sign, upper_tick, lower_tick)
                    .unwrap();

                assert_eq!(x, TokenAmount::new(U256::from(1)));
                assert_eq!(y, TokenAmount::new(U256::from(0)));
                assert_eq!(pool.liquidity, Liquidity::from_integer(1))
            }
        }
        // Remove Liquidity
        {
            let mut pool = Pool {
                liquidity: Liquidity::from_integer(10),
                current_tick_index: 2,
                sqrt_price: SqrtPrice::new(U128::from(1)),
                ..Default::default()
            };

            let liquidity_delta = Liquidity::from_integer(5);
            let liquidity_sign = false;
            let upper_tick = 3;
            let lower_tick = 1;

            let (x, y) = pool
                .update_liquidity(liquidity_delta, liquidity_sign, upper_tick, lower_tick)
                .unwrap();

            assert_eq!(
                x,
                TokenAmount::new(U256::from(2500375009372499999999997u128))
            );
            assert_eq!(y, TokenAmount::new(U256::from(5)));
            assert_eq!(pool.liquidity, Liquidity::from_integer(5),)
        }
    }
}
