use contracts::{declare_storage, get_max_chunk};
pub use contracts::{
    AwaitingTransfer, FeeTiers, InvariantError, PoolKey, PoolKeys, Pools, Positions, Tick, Tickmap,
    Ticks, UpdatePoolTick,
};
pub use decimal::*;
pub use gstd::exec;
pub use io::*;
pub use math::{
    check_tick, compute_swap_step,
    sqrt_price::{get_max_tick, get_min_tick, SqrtPrice},
    token_amount::TokenAmount,
    MAX_SQRT_PRICE, MIN_SQRT_PRICE,
};
pub use sails_rs::{collections::HashMap, prelude::*};
pub use traceable_result::*;

#[derive(Debug, Default)]
pub struct Invariant {
    pub config: InvariantConfig,
    pub fee_tiers: FeeTiers,
    pub pools: Pools,
    pub pool_keys: PoolKeys,
    pub positions: Positions,
    pub ticks: Ticks,
    pub tickmap: Tickmap,
    pub balances: HashMap<ActorId, HashMap<ActorId, TokenAmount>>,
    pub awaiting_transfers: HashMap<(MessageId, ActorId), AwaitingTransfer>,
}

impl Invariant {
    pub fn increase_token_balance(
        &mut self,
        token: &ActorId,
        caller: &ActorId,
        amount: TokenAmount,
    ) -> Result<(), InvariantError> {
        if amount.is_zero() {
            return Ok(());
        }

        let token_balance: &mut TokenAmount = self
            .balances
            .entry(*caller)
            .or_insert(HashMap::new())
            .entry(*token)
            .or_insert(TokenAmount::new(U256::from(0)));

        *token_balance = token_balance
            .checked_add(amount)
            .map_err(|_| InvariantError::FailedToChangeTokenBalance)?;

        Ok(())
    }

    pub fn can_increase_token_balance(
        &self,
        token: &ActorId,
        caller: &ActorId,
        amount: TokenAmount,
    ) -> bool {
        self.balances
            .get(caller)
            .and_then(|tokens| tokens.get(token))
            .and_then(|a| a.checked_add(amount).is_ok().into())
            .unwrap_or(true)
    }

    pub fn decrease_token_balance(
        &mut self,
        token: &ActorId,
        caller: &ActorId,
        amount: Option<TokenAmount>,
    ) -> Result<TokenAmount, InvariantError> {
        if matches!(amount, Some(TokenAmount(U256([0, 0, 0, 0])))) {
            return Ok(amount.unwrap());
        }

        let (balance, remove_balance) = {
            let token_balances = self
                .balances
                .get_mut(caller)
                .ok_or(InvariantError::NoBalanceForTheToken)?;

            let balance = token_balances
                .get_mut(token)
                .ok_or(InvariantError::NoBalanceForTheToken)?;

            if matches!(amount, Some(amount) if amount != *balance) {
                let amount = amount.unwrap();

                *balance = balance
                    .checked_sub(amount)
                    .map_err(|_| InvariantError::FailedToChangeTokenBalance)?;

                (amount, false)
            } else {
                let old_balance = token_balances.remove(token).unwrap();

                (old_balance, token_balances.is_empty())
            }
        };

        if remove_balance {
            self.balances.remove(caller);
        }

        Ok(balance)
    }

    pub fn calculate_swap(
        &self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<CalculateSwapResult, InvariantError> {
        let current_timestamp = exec::block_timestamp();

        if amount.is_zero() {
            return Err(InvariantError::AmountIsZero);
        }

        let mut ticks: Vec<Tick> = vec![];

        let mut pool = self.pools.get(&pool_key)?;

        if x_to_y {
            if pool.sqrt_price <= sqrt_price_limit
                || sqrt_price_limit > SqrtPrice::new(MAX_SQRT_PRICE.into())
            {
                return Err(InvariantError::WrongLimit);
            }
        } else if pool.sqrt_price >= sqrt_price_limit
            || sqrt_price_limit < SqrtPrice::new(MIN_SQRT_PRICE.into())
        {
            return Err(InvariantError::WrongLimit);
        }

        let tick_limit = if x_to_y {
            get_min_tick(pool_key.fee_tier.tick_spacing)
        } else {
            get_max_tick(pool_key.fee_tier.tick_spacing)
        };

        let mut remaining_amount = amount;

        let mut total_amount_in = TokenAmount::new(U256::from(0));
        let mut total_amount_out = TokenAmount::new(U256::from(0));

        let event_start_sqrt_price = pool.sqrt_price;
        let mut event_fee_amount = TokenAmount::new(U256::from(0));

        while !remaining_amount.is_zero() {
            let (swap_limit, limiting_tick) = self.tickmap.get_closer_limit(
                sqrt_price_limit,
                x_to_y,
                pool.current_tick_index,
                pool_key.fee_tier.tick_spacing,
                pool_key,
            )?;
            let result = unwrap!(compute_swap_step(
                pool.sqrt_price,
                swap_limit,
                pool.liquidity,
                remaining_amount,
                by_amount_in,
                pool_key.fee_tier.fee,
            ));

            // make remaining amount smaller
            if by_amount_in {
                remaining_amount -= result.amount_in + result.fee_amount;
            } else {
                remaining_amount -= result.amount_out;
            }

            unwrap!(pool.add_fee(result.fee_amount, x_to_y, self.config.protocol_fee));
            event_fee_amount += result.fee_amount;

            pool.sqrt_price = result.next_sqrt_price;

            total_amount_in += result.amount_in + result.fee_amount;
            total_amount_out += result.amount_out;

            // Fail if price would go over swap limit
            if pool.sqrt_price == sqrt_price_limit && !remaining_amount.is_zero() {
                return Err(InvariantError::PriceLimitReached);
            }

            let mut tick_update = {
                if let Some((tick_index, is_initialized)) = limiting_tick {
                    if is_initialized {
                        let tick = self.ticks.get(pool_key, tick_index).cloned()?;
                        UpdatePoolTick::TickInitialized(tick)
                    } else {
                        UpdatePoolTick::TickUninitialized(tick_index)
                    }
                } else {
                    UpdatePoolTick::NoTick
                }
            };

            let (amount_to_add, amount_after_tick_update, has_crossed) = pool.update_tick(
                result,
                swap_limit,
                &mut tick_update,
                remaining_amount,
                by_amount_in,
                x_to_y,
                current_timestamp,
                self.config.protocol_fee,
                pool_key.fee_tier,
            );

            remaining_amount = amount_after_tick_update;
            total_amount_in += amount_to_add;

            if let UpdatePoolTick::TickInitialized(tick) = tick_update {
                if has_crossed {
                    ticks.push(tick)
                }
            }

            let reached_tick_limit = match x_to_y {
                true => pool.current_tick_index <= tick_limit,
                false => pool.current_tick_index >= tick_limit,
            };

            if reached_tick_limit {
                return Err(InvariantError::TickLimitReached);
            }
        }

        if total_amount_out.get() == U256::from(0) {
            return Err(InvariantError::NoGainSwap);
        }

        Ok(CalculateSwapResult {
            amount_in: total_amount_in,
            amount_out: total_amount_out,
            start_sqrt_price: event_start_sqrt_price,
            target_sqrt_price: pool.sqrt_price,
            fee: event_fee_amount,
            pool,
            ticks,
        })
    }

    pub fn create_tick(&mut self, pool_key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
        let current_timestamp = exec::block_timestamp();

        check_tick(index, pool_key.fee_tier.tick_spacing)
            .map_err(|_| InvariantError::InvalidTickIndexOrTickSpacing)?;

        let pool = self.pools.get(&pool_key)?;

        let tick = Tick::create(index, &pool, current_timestamp);

        Ok(tick)
    }

    pub fn get_or_create_tick(&mut self, pool_key: PoolKey, index: i32) -> (Tick, bool) {
        if let Ok(tick) = self.ticks.get(pool_key, index).cloned() {
            return (tick, false);
        }

        (self.create_tick(pool_key, index).unwrap(), true)
    }

    pub fn add_tick(&mut self, pool_key: PoolKey, tick: Tick) -> Result<(), InvariantError> {
        self.ticks.add(pool_key, tick.index, tick)?;

        self.tickmap
            .flip(true, tick.index, pool_key.fee_tier.tick_spacing, pool_key);

        Ok(())
    }

    pub fn remove_tick(&mut self, key: PoolKey, tick: Tick) -> Result<(), InvariantError> {
        if !tick.liquidity_gross.is_zero() {
            return Err(InvariantError::NotEmptyTickDeinitialization);
        }

        self.tickmap
            .flip(false, tick.index, key.fee_tier.tick_spacing, key);
        self.ticks.remove(key, tick.index)?;
        Ok(())
    }

    pub fn tickmap_slice(
        &self,
        range: impl Iterator<Item = u16>,
        pool_key: PoolKey,
    ) -> Vec<(u16, u64)> {
        let mut tickmap_slice: Vec<(u16, u64)> = vec![];

        for chunk_index in range {
            if let Some(chunk) = self.tickmap.bitmap.get(&(chunk_index, pool_key)) {
                tickmap_slice.push((chunk_index, *chunk));
            }
        }

        tickmap_slice
    }

    pub fn liquidity_ticks_count(&self, pool_key: PoolKey) -> u32 {
        let mut sum = 0;
        for chunk_index in 0..get_max_chunk(pool_key.fee_tier.tick_spacing) {
            if let Some(chunk) = self.tickmap.bitmap.get(&(chunk_index, pool_key)) {
                sum += chunk.count_ones()
            }
        }

        sum
    }
}

declare_storage!(module: invariant, name: InvariantStorage, ty: Invariant);

impl InvariantStorage {
    pub fn with_config(config: InvariantConfig) -> Result<(), Invariant> {
        InvariantStorage::set(Invariant {
            config,
            ..Invariant::default()
        })
    }
}
