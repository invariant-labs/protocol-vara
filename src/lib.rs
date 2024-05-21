#![no_std]
extern crate alloc;
#[cfg(test)]
mod e2e;
#[cfg(test)]
mod test_helpers;
use contracts::{
    errors::InvariantError, FeeTier, FeeTiers, Pool, PoolKey, PoolKeys, Pools, Position, Positions,
    Tick, Tickmap, Ticks, UpdatePoolTick,
};
use decimal::*;
use fungible_token_io::{FTAction, FTError, FTEvent};
use gstd::{
    async_init, async_main,
    collections::HashMap,
    exec,
    msg::{self, reply},
    prelude::*,
    ActorId,
};
use io::*;
use math::{
    check_tick, compute_swap_step,
    liquidity::Liquidity,
    percentage::Percentage,
    sqrt_price::{get_max_tick, get_min_tick, SqrtPrice},
    token_amount::TokenAmount,
    MAX_SQRT_PRICE, MIN_SQRT_PRICE,
};
use traceable_result::*;

// TODO: Update once the SDK tests are in place and proper measurement is possible
pub const TRANSFER_GAS_LIMIT: u64 = 5_000_000 * 2;
pub const TRANSFER_REPLY_HANDLING_COST: u64 = 6_000_000 * 2;
pub const BALANCE_CHANGE_COST: u64 = 100_000 * 2;
pub const TRANSFER_COST: u64 =
    TRANSFER_GAS_LIMIT + TRANSFER_REPLY_HANDLING_COST + BALANCE_CHANGE_COST;
pub const CREATE_POSITION_COST: u64 = 100_000 * 2;
pub const SWAP_UPDATE_COST: u64 = 100_000 * 2;
pub const REMOVE_POSITION_COST: u64 = 100_000 * 2;
pub const WITHDRAW_PROTOCOL_FEE_COST: u64 = 100_000 * 2;

#[derive(Default)]
pub struct Invariant {
    pub config: InvariantConfig,
    pub fee_tiers: FeeTiers,
    pub pools: Pools,
    pub pool_keys: PoolKeys,
    pub positions: Positions,
    pub ticks: Ticks,
    pub tickmap: Tickmap,
    pub transaction_id: u64,
    pub balances: HashMap<ActorId, HashMap<ActorId, TokenAmount>>,
}
pub enum TransferError {
    FailedToSendMessage,
    RecoverableFTError,
    UnRecoverableFTError,
}

impl Invariant {
    pub fn change_protocol_fee(
        &mut self,
        protocol_fee: Percentage,
    ) -> Result<Percentage, InvariantError> {
        if !self.is_caller_admin() {
            return Err(InvariantError::NotAdmin);
        }

        self.config.protocol_fee = protocol_fee;
        Ok(self.config.protocol_fee)
    }

    pub fn get_protocol_fee(&self) -> Percentage {
        self.config.protocol_fee
    }

    pub fn add_fee_tier(&mut self, fee_tier: FeeTier) -> Result<FeeTier, InvariantError> {
        if fee_tier.tick_spacing == 0 || fee_tier.tick_spacing > 100 {
            return Err(InvariantError::InvalidTickSpacing);
        }

        if fee_tier.fee >= Percentage::from_integer(1) {
            return Err(InvariantError::InvalidFee);
        }

        if !self.is_caller_admin() {
            return Err(InvariantError::NotAdmin);
        }

        self.fee_tiers.add(&fee_tier)?;
        Ok(fee_tier)
    }

    pub fn fee_tier_exists(&self, fee_tier: FeeTier) -> bool {
        self.fee_tiers.contains(&fee_tier)
    }

    pub fn remove_fee_tier(&mut self, fee_tier: FeeTier) -> Result<FeeTier, InvariantError> {
        if !self.is_caller_admin() {
            return Err(InvariantError::NotAdmin);
        }

        self.fee_tiers.remove(&fee_tier)?;
        Ok(fee_tier)
    }

    pub fn get_fee_tiers(&self) -> Vec<FeeTier> {
        self.fee_tiers.get_all()
    }

    pub fn create_pool(
        &mut self,
        token_0: ActorId,
        token_1: ActorId,
        fee_tier: FeeTier,
        init_sqrt_price: SqrtPrice,
        init_tick: i32,
    ) -> Result<(), InvariantError> {
        let current_timestamp = exec::block_timestamp();

        if !self.fee_tiers.contains(&fee_tier) {
            return Err(InvariantError::FeeTierNotFound);
        };

        check_tick(init_tick, fee_tier.tick_spacing)
            .map_err(|_| InvariantError::InvalidInitTick)?;

        let pool_key = PoolKey::new(token_0, token_1, fee_tier)?;

        if self.pools.get(&pool_key).is_ok() {
            return Err(InvariantError::PoolAlreadyExist);
        };

        let pool = Pool::create(
            init_sqrt_price,
            init_tick,
            current_timestamp,
            fee_tier.tick_spacing,
            self.config.admin,
        )?;
        self.pools.add(&pool_key, &pool)?;
        self.pool_keys.add(&pool_key)?;

        Ok(())
    }

    pub fn get_pool(
        &self,
        token_0: ActorId,
        token_1: ActorId,
        fee_tier: FeeTier,
    ) -> Result<Pool, InvariantError> {
        let pool_key = PoolKey::new(token_0, token_1, fee_tier)?;
        self.pools.get(&pool_key)
    }

    pub fn get_pools(&self, size: u8, offset: u16) -> Result<Vec<PoolKey>, InvariantError> {
        self.pool_keys.get_all(size, offset)
    }

    pub fn change_fee_receiver(
        &mut self,
        pool_key: PoolKey,
        fee_receiver: ActorId,
    ) -> Result<(), InvariantError> {
        if !self.is_caller_admin() {
            return Err(InvariantError::NotAdmin);
        }

        let mut pool = self.pools.get(&pool_key)?;
        pool.fee_receiver = fee_receiver;
        self.pools.update(&pool_key, &pool)?;

        Ok(())
    }

    pub async fn create_position(
        &mut self,
        pool_key: PoolKey,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_delta: Liquidity,
        slippage_limit_lower: SqrtPrice,
        slippage_limit_upper: SqrtPrice,
    ) -> Result<Position, InvariantError> {
        if exec::gas_available() < CREATE_POSITION_COST + 2 * TRANSFER_COST {
            return Err(InvariantError::NotEnoughGasToExecute);
        }

        let caller = msg::source();
        let program = exec::program_id();
        let current_timestamp = exec::block_timestamp();
        let current_block_number = exec::block_height() as u64;

        // liquidity delta = 0 => return
        if liquidity_delta == Liquidity::new(0) {
            return Err(InvariantError::ZeroLiquidity);
        }

        if lower_tick == upper_tick {
            return Err(InvariantError::InvalidTickIndex);
        }

        let mut pool = self.pools.get(&pool_key)?;

        let (mut lower_tick, should_add_lower) = self.get_or_create_tick(pool_key, lower_tick);
        let (mut upper_tick, should_add_upper) = self.get_or_create_tick(pool_key, upper_tick);

        let (position, x, y) = Position::create(
            &mut pool,
            pool_key,
            &mut lower_tick,
            &mut upper_tick,
            current_timestamp,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
            current_block_number,
            pool_key.fee_tier.tick_spacing,
        )?;

        self.transfer_tokens(&pool_key.token_x, None, &caller, &program, x.get())
            .await
            .map_err(|_| InvariantError::TransferError)?;

        let res = self
            .transfer_tokens(&pool_key.token_y, None, &caller, &program, y.get())
            .await;

        if let Err(_err) = res {
            self.increase_token_balance(&caller, &pool_key.token_x, x)
                .unwrap();

            reply_with_err_and_exit(InvariantError::RecoverableTransferError);
        }

        self.pools.update(&pool_key, &pool)?;

        self.positions.add(&caller, &position);

        if should_add_lower {
            self.add_tick(pool_key, lower_tick)?;
        } else {
            self.ticks.update(pool_key, lower_tick.index, lower_tick)?;
        }

        if should_add_upper {
            self.add_tick(pool_key, upper_tick)?;
        } else {
            self.ticks.update(pool_key, upper_tick.index, upper_tick)?;
        }

        self.emit_event(InvariantEvent::PositionCreatedEvent {
            block_timestamp: exec::block_timestamp(),
            address: msg::source(),
            pool_key,
            liquidity_delta,
            lower_tick: lower_tick.index,
            upper_tick: upper_tick.index,
            current_sqrt_price: pool.sqrt_price,
        });

        Ok(position)
    }

    pub fn get_position(&self, owner_id: &ActorId, index: u32) -> Result<Position, InvariantError> {
        self.positions.get(owner_id, index).cloned()
    }

    pub fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
        self.ticks.get(key, index).cloned()
    }

    pub fn is_tick_initialized(&self, key: PoolKey, index: i32) -> bool {
        self.tickmap.get(index, key.fee_tier.tick_spacing, key)
    }
    pub async fn remove_position(
        &mut self,
        index: u32,
    ) -> Result<(TokenAmount, TokenAmount), InvariantError> {
        if exec::gas_available() < REMOVE_POSITION_COST + 2 * TRANSFER_COST {
            return Err(InvariantError::NotEnoughGasToExecute);
        }

        let caller = msg::source();
        let current_timestamp = exec::block_timestamp();

        let mut position = self.positions.get(&caller, index).cloned()?;
        let Position {
            pool_key,
            lower_tick_index,
            upper_tick_index,
            liquidity: withdrawn_liquidity,
            ..
        } = position;

        let mut lower_tick = self.ticks.get(pool_key, lower_tick_index).cloned()?;

        let mut upper_tick = self.ticks.get(pool_key, upper_tick_index).cloned()?;

        let pool = &mut self.pools.get(&pool_key)?;

        let (amount_x, amount_y, remove_lower_tick, remove_upper_tick) = position.remove(
            pool,
            current_timestamp,
            &mut lower_tick,
            &mut upper_tick,
            pool_key.fee_tier.tick_spacing,
        );

        self.pools.update(&pool_key, pool)?;

        if remove_lower_tick {
            self.remove_tick(pool_key, lower_tick)?;
        } else {
            self.ticks.update(pool_key, lower_tick_index, lower_tick)?;
        }

        if remove_upper_tick {
            self.remove_tick(pool_key, upper_tick)?;
        } else {
            self.ticks.update(pool_key, upper_tick_index, upper_tick)?;
        }

        self.positions.remove(&caller, index)?;

        let token_x = pool_key.token_x;
        let token_y = pool_key.token_y;

        self.withdraw_token_pair(&caller, &(token_x, amount_x), &(token_y, amount_y))
            .await?;

        self.emit_event(InvariantEvent::PositionRemovedEvent {
            block_timestamp: exec::block_timestamp(),
            caller: msg::source(),
            pool_key,
            liquidity: withdrawn_liquidity,
            lower_tick_index: lower_tick.index,
            upper_tick_index: upper_tick.index,
            sqrt_price: pool.sqrt_price,
        });

        Ok((amount_x, amount_y))
    }

    pub fn transfer_position(
        &mut self,
        index: u32,
        receiver: &ActorId,
    ) -> Result<(), InvariantError> {
        let caller = msg::source();

        self.positions.transfer(&caller, index, &receiver)?;

        Ok(())
    }

    pub fn get_all_positions(&self, owner_id: &ActorId) -> Vec<Position> {
        self.positions.get_all(&owner_id)
    }

    pub async fn swap(
        &mut self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<CalculateSwapResult, InvariantError> {
        let caller = msg::source();
        let program = exec::program_id();

        let calculate_swap_result =
            self.calculate_swap(pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit)?;

        let mut crossed_tick_indexes: Vec<i32> = vec![];

        for tick in calculate_swap_result.ticks.iter() {
            crossed_tick_indexes.push(tick.index);
        }

        let update = |invariant: &mut Self| {
            for tick in calculate_swap_result.ticks.iter() {
                invariant.ticks.update(pool_key, tick.index, *tick)?;
            }

            invariant
                .pools
                .update(&pool_key, &calculate_swap_result.pool)?;

            Ok::<(), InvariantError>(())
        };

        if exec::gas_available() < SWAP_UPDATE_COST + 2 * TRANSFER_COST {
            return Err(InvariantError::NotEnoughGasToExecute);
        }

        let (swapped_token, returned_token) = if x_to_y {
            (&pool_key.token_x, &pool_key.token_y)
        } else {
            (&pool_key.token_y, &pool_key.token_x)
        };

        self.transfer_tokens(
            swapped_token,
            None,
            &caller,
            &program,
            calculate_swap_result.amount_in.get(),
        )
        .await
        .map_err(|_| InvariantError::TransferError)?;

        let res = self
            .transfer_tokens(
                &returned_token,
                None,
                &program,
                &caller,
                calculate_swap_result.amount_out.get(),
            )
            .await;

        if let Err(_err) = res {
            self.increase_token_balance(&caller, &swapped_token, calculate_swap_result.amount_in)?;

            reply_with_err_and_exit(InvariantError::RecoverableTransferError);
        }

        update(self)?;

        if !crossed_tick_indexes.is_empty() {
            self.emit_event(InvariantEvent::CrossTickEvent {
                timestamp: exec::block_timestamp(),
                address: caller,
                pool: pool_key,
                indexes: crossed_tick_indexes,
            });
        }

        self.emit_event(InvariantEvent::SwapEvent {
            timestamp: exec::block_timestamp(),
            address: caller,
            pool: pool_key,
            amount_in: calculate_swap_result.amount_in,
            amount_out: calculate_swap_result.amount_out,
            fee: calculate_swap_result.fee,
            start_sqrt_price: calculate_swap_result.start_sqrt_price,
            target_sqrt_price: calculate_swap_result.target_sqrt_price,
            x_to_y,
        });

        Ok(calculate_swap_result)
    }

    pub fn quote(
        &self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<QuoteResult, InvariantError> {
        let calculate_swap_result =
            self.calculate_swap(pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit)?;

        Ok(QuoteResult {
            amount_in: calculate_swap_result.amount_in,
            amount_out: calculate_swap_result.amount_out,
            target_sqrt_price: calculate_swap_result.pool.sqrt_price,
            ticks: calculate_swap_result.ticks,
        })
    }

    pub fn quote_route(
        &mut self,
        amount_in: TokenAmount,
        swaps: Vec<SwapHop>,
    ) -> Result<TokenAmount, InvariantError> {
        let amount_out = self.route(amount_in, swaps)?;

        Ok(amount_out)
    }

    pub fn route(
        &mut self,
        amount_in: TokenAmount,
        swaps: Vec<SwapHop>,
    ) -> Result<TokenAmount, InvariantError> {
        let mut next_swap_amount = amount_in;

        for swap in swaps.iter() {
            let SwapHop { pool_key, x_to_y } = *swap;

            let sqrt_price_limit = if x_to_y {
                SqrtPrice::new(MIN_SQRT_PRICE)
            } else {
                SqrtPrice::new(MAX_SQRT_PRICE)
            };

            let result =
                self.calculate_swap(pool_key, x_to_y, next_swap_amount, true, sqrt_price_limit)?;

            next_swap_amount = result.amount_out;
        }

        Ok(next_swap_amount)
    }

    pub async fn claim_fee(
        &mut self,
        index: u32,
    ) -> Result<(TokenAmount, TokenAmount), InvariantError> {
        let caller = msg::source();
        let current_timestamp = exec::block_timestamp();

        let mut position = self.positions.get(&caller, index).cloned()?;

        let mut lower_tick = self
            .ticks
            .get(position.pool_key, position.lower_tick_index)
            .cloned()?;

        let mut upper_tick = self
            .ticks
            .get(position.pool_key, position.upper_tick_index)
            .cloned()?;

        let mut pool = self.pools.get(&position.pool_key)?;

        let (x, y) = position.claim_fee(
            &mut pool,
            &mut upper_tick,
            &mut lower_tick,
            current_timestamp,
        );

        self.positions.update(&caller, index, &position)?;
        self.pools.update(&position.pool_key, &pool)?;
        self.ticks
            .update(position.pool_key, upper_tick.index, upper_tick)?;
        self.ticks
            .update(position.pool_key, lower_tick.index, lower_tick)?;

        let both_positive = x.get() > 0 && y.get() > 0;

        let required_gas = if both_positive {
            2 * TRANSFER_COST
        } else {
            TRANSFER_COST
        };
        if exec::gas_available() < required_gas {
            return Err(InvariantError::NotEnoughGasToExecute);
        }

        if both_positive {
            self.withdraw_token_pair(
                &caller,
                &(position.pool_key.token_x, x),
                &(position.pool_key.token_y, y),
            )
            .await?;
        } else if x.get() > 0 {
            self.withdraw_single_token(&position.pool_key.token_x, &caller, x)
                .await?;
        } else if y.get() > 0 {
            self.withdraw_single_token(&position.pool_key.token_y, &caller, y)
                .await?;
        }

        Ok((x, y))
    }

    pub async fn withdraw_protocol_fee(&mut self, pool_key: PoolKey) -> Result<(), InvariantError> {
        if exec::gas_available() < WITHDRAW_PROTOCOL_FEE_COST + 2 * TRANSFER_COST {
            return Err(InvariantError::NotEnoughGasToExecute);
        }

        let caller = msg::source();

        let mut pool = self.pools.get(&pool_key)?;

        if pool.fee_receiver != caller {
            return Err(InvariantError::NotFeeReceiver);
        }

        let (amount_x, amount_y) = pool.withdraw_protocol_fee(pool_key);
        self.pools.update(&pool_key, &pool)?;

        if exec::gas_available() < 2 * TRANSFER_COST {
            return Err(InvariantError::NotEnoughGasToExecute);
        }

        self.withdraw_token_pair(
            &caller,
            &(pool_key.token_x, amount_x),
            &(pool_key.token_y, amount_y),
        )
        .await?;

        Ok(())
    }

    pub async fn claim_lost_tokens(&mut self, token: &ActorId) -> Result<(), InvariantError> {
        let caller = msg::source();
        let (balance, remove_balance) = {
            let token_balances = self
                .balances
                .get_mut(&caller)
                .ok_or(InvariantError::NoBalanceForTheToken)?;

            let balance = *token_balances
                .get(token)
                .ok_or(InvariantError::NoBalanceForTheToken)?;

            token_balances.remove(token).unwrap();

            (balance, token_balances.is_empty())
        };

        if remove_balance {
            self.balances.remove(&caller);
        }

        if exec::gas_available() < TRANSFER_COST {
            return Err(InvariantError::NotEnoughGasToExecute);
        }

        self.withdraw_single_token(token, &caller, balance).await?;

        Ok(())
    }

    pub fn get_user_balances(&self, user: &ActorId) -> Vec<(ActorId, TokenAmount)> {
        self.balances
            .get(user)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect()
    }

    fn increase_token_balance(
        &mut self,
        source: &ActorId,
        token: &ActorId,
        amount: TokenAmount,
    ) -> Result<(), InvariantError> {
        let token_balance: &mut TokenAmount = self
            .balances
            .entry(*source)
            .or_insert(HashMap::new())
            .entry(*token)
            .or_insert(TokenAmount(0));

        *token_balance = token_balance
            .checked_add(amount)
            .map_err(|_| InvariantError::FailedToChangeTokenBalance)?;

        Ok(())
    }

    async fn transfer_tokens(
        &mut self,
        token_address: &ActorId,
        tx_id: Option<u64>,
        from: &ActorId,
        to: &ActorId,
        amount_tokens: u128,
    ) -> Result<(), TransferError> {
        let tx_id = tx_id.or(self.generate_transaction_id().into());
        let reply = msg::send_with_gas_for_reply_as::<_, Result<FTEvent, FTError>>(
            *token_address,
            FTAction::Transfer {
                tx_id,
                from: *from,
                to: *to,
                amount: amount_tokens,
            },
            TRANSFER_GAS_LIMIT,
            0,
            TRANSFER_REPLY_HANDLING_COST,
        )
        // Sending the message does not save the state so we must use a different error type
        .map_err(|_| TransferError::FailedToSendMessage)?
        .await
        // This error occurs in cases when the program panics
        .map_err(|_| TransferError::RecoverableFTError)?;

        match reply {
            Ok(ft_event) => match ft_event {
                FTEvent::Transfer {
                    from: _,
                    to: _,
                    amount: _,
                } => return Ok(()),
                // Token doesn't match the GRC-20 standard
                _ => return Err(TransferError::UnRecoverableFTError),
            },
            Err(ft_error) => match ft_error {
                FTError::NotEnoughBalance | FTError::NotAllowedToTransfer => {
                    return Err(TransferError::RecoverableFTError)
                }
                FTError::TxAlreadyExists | FTError::ZeroAddress => {
                    return Err(TransferError::UnRecoverableFTError)
                }
            },
        }
    }

    async fn withdraw_single_token(
        &mut self,
        token: &ActorId,
        caller: &ActorId,
        balance: TokenAmount,
    ) -> Result<(), InvariantError> {
        let res = self
            .transfer_tokens(&token, None, &exec::program_id(), &caller, balance.get())
            .await;

        if let Err(err) = res {
            match err {
                TransferError::RecoverableFTError => {
                    self.increase_token_balance(&caller, &token, balance)
                        .unwrap();

                    reply_with_err_and_exit(InvariantError::RecoverableTransferError)
                }
                TransferError::FailedToSendMessage => {
                    return Err(InvariantError::TransferError);
                }
                _ => {
                    return Err(InvariantError::UnrecoverableTransferError);
                }
            }
        }

        Ok(())
    }
    async fn withdraw_token_pair(
        &mut self,
        caller: &ActorId,
        token_x: &(ActorId, TokenAmount),
        token_y: &(ActorId, TokenAmount),
    ) -> Result<(), InvariantError> {
        let program = exec::program_id();

        let first_transaction = self
            .transfer_tokens(&token_x.0, None, &program, &caller, token_x.1.get())
            .await;

        if let Err(err) = first_transaction {
            match err {
                TransferError::FailedToSendMessage => {
                    // It is still possible to revert the state at this point
                    return Err(InvariantError::TransferError);
                }
                TransferError::RecoverableFTError => {
                    self.increase_token_balance(&caller, &token_x.0, token_x.1)
                        .unwrap();
                    self.increase_token_balance(&caller, &token_y.0, token_y.1)
                        .unwrap();

                    reply_with_err_and_exit(InvariantError::RecoverableTransferError);
                }
                TransferError::UnRecoverableFTError => {
                    self.increase_token_balance(&caller, &token_y.0, token_y.1)
                        .unwrap();

                    reply_with_err_and_exit(InvariantError::RecoverableTransferError);
                }
            }
        }

        let second_transaction = self
            .transfer_tokens(&token_y.0, None, &program, &caller, token_y.1.get())
            .await;
        if let Err(err) = second_transaction {
            match err {
                TransferError::RecoverableFTError | TransferError::FailedToSendMessage => {
                    self.increase_token_balance(&caller, &token_y.0, token_y.1)
                        .unwrap();

                    reply_with_err_and_exit(InvariantError::RecoverableTransferError)
                }
                TransferError::UnRecoverableFTError => {
                    return Err(InvariantError::UnrecoverableTransferError);
                }
            }
        }

        Ok(())
    }

    fn calculate_swap(
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
                || sqrt_price_limit > SqrtPrice::new(MAX_SQRT_PRICE)
            {
                return Err(InvariantError::WrongLimit);
            }
        } else if pool.sqrt_price >= sqrt_price_limit
            || sqrt_price_limit < SqrtPrice::new(MIN_SQRT_PRICE)
        {
            return Err(InvariantError::WrongLimit);
        }

        let tick_limit = if x_to_y {
            get_min_tick(pool_key.fee_tier.tick_spacing)
        } else {
            get_max_tick(pool_key.fee_tier.tick_spacing)
        };

        let mut remaining_amount = amount;

        let mut total_amount_in = TokenAmount(0);
        let mut total_amount_out = TokenAmount(0);

        let event_start_sqrt_price = pool.sqrt_price;
        let mut event_fee_amount = TokenAmount(0);

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
                &mut tick_update,
                swap_limit,
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

        if total_amount_out.get() == 0 {
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

    fn generate_transaction_id(&mut self) -> u64 {
        let transaction_id = self.transaction_id;
        self.transaction_id = self.transaction_id.wrapping_add(1);
        transaction_id
    }

    fn create_tick(&mut self, pool_key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
        let current_timestamp = exec::block_timestamp();

        check_tick(index, pool_key.fee_tier.tick_spacing)
            .map_err(|_| InvariantError::InvalidTickIndexOrTickSpacing)?;

        let pool = self.pools.get(&pool_key)?;

        let tick = Tick::create(index, &pool, current_timestamp);

        Ok(tick)
    }

    fn get_or_create_tick(&mut self, pool_key: PoolKey, index: i32) -> (Tick, bool) {
        if let Ok(tick) = self.ticks.get(pool_key, index).cloned() {
            return (tick, false);
        }

        (self.create_tick(pool_key, index).unwrap(), true)
    }

    fn add_tick(&mut self, pool_key: PoolKey, tick: Tick) -> Result<(), InvariantError> {
        self.ticks.add(pool_key, tick.index, tick)?;

        self.tickmap
            .flip(true, tick.index, pool_key.fee_tier.tick_spacing, pool_key);

        Ok(())
    }

    fn remove_tick(&mut self, key: PoolKey, tick: Tick) -> Result<(), InvariantError> {
        if !tick.liquidity_gross.is_zero() {
            return Err(InvariantError::NotEmptyTickDeinitialization);
        }

        self.tickmap
            .flip(false, tick.index, key.fee_tier.tick_spacing, key);
        self.ticks.remove(key, tick.index)?;
        Ok(())
    }

    fn emit_event(&self, event: InvariantEvent) {
        msg::send(msg::source(), event, 0).expect("Unable to emit event");
    }

    fn is_caller_admin(&self) -> bool {
        msg::source() == self.config.admin
    }
}

static mut INVARIANT: Option<Invariant> = None;

fn reply_with_err(err: InvariantError) {
    panic!("InvariantError: {:?}", err);
}

fn reply_with_err_and_exit(err: InvariantError) {
    reply(InvariantEvent::ActionFailed(err), 0).expect("Failed to send reply");
    exec::leave();
}

#[async_init]
async fn init() {
    let init: InitInvariant = msg::load().expect("Unable to decode InitInvariant");

    let invariant = Invariant {
        config: init.config,
        ..Invariant::default()
    };

    unsafe {
        INVARIANT = Some(invariant);
    }
}
//'handle' endpoint
#[async_main]
async fn main() {
    let action: InvariantAction = msg::load().expect("Unable to decode InvariantAction");
    let invariant = unsafe { INVARIANT.get_or_insert(Default::default()) };

    match action {
        InvariantAction::ChangeProtocolFee(protocol_fee) => {
            match invariant.change_protocol_fee(protocol_fee) {
                Ok(protocol_fee) => {
                    reply(InvariantEvent::ProtocolFeeChanged(protocol_fee), 0)
                        .expect("Unable to reply");
                }
                Err(e) => {
                    reply_with_err(e);
                }
            };
        }
        InvariantAction::AddFeeTier(fee_tier) => {
            match invariant.add_fee_tier(fee_tier) {
                Ok(_fee_tier) => {}
                Err(e) => {
                    reply_with_err(e);
                }
            };
        }
        InvariantAction::RemoveFeeTier(fee_tier) => {
            match invariant.remove_fee_tier(fee_tier) {
                Ok(_fee_tier) => {}
                Err(e) => {
                    reply_with_err(e);
                }
            };
        }
        InvariantAction::CreatePool {
            token_0,
            token_1,
            fee_tier,
            init_sqrt_price,
            init_tick,
        } => match invariant.create_pool(token_0, token_1, fee_tier, init_sqrt_price, init_tick) {
            Ok(_) => {}
            Err(e) => {
                reply_with_err(e);
            }
        },
        InvariantAction::ChangeFeeReceiver(pool_key, fee_receiver) => {
            match invariant.change_fee_receiver(pool_key, fee_receiver) {
                Ok(_) => {}
                Err(e) => {
                    reply_with_err(e);
                }
            }
        }
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick,
            upper_tick,
            liquidity_delta,
            slippage_limit_lower,
            slippage_limit_upper,
        } => {
            match invariant
                .create_position(
                    pool_key,
                    lower_tick,
                    upper_tick,
                    liquidity_delta,
                    slippage_limit_lower,
                    slippage_limit_upper,
                )
                .await
            {
                Ok(position) => {
                    reply(InvariantEvent::PositionCreatedReturn(position), 0)
                        .expect("Unable to reply");
                }
                Err(e) => {
                    reply_with_err(e);
                }
            }
        }
        InvariantAction::RemovePosition { position_id } => {
            match invariant.remove_position(position_id).await {
                Ok((amount_x, amount_y)) => {
                    reply(InvariantEvent::PositionRemovedReturn(amount_x, amount_y), 0)
                        .expect("Unable to reply");
                }
                Err(e) => {
                    reply_with_err(e);
                }
            }
        }
        InvariantAction::TransferPosition { index, receiver } => {
            match invariant.transfer_position(index, &receiver) {
                Ok(_) => {}
                Err(e) => {
                    reply_with_err(e);
                }
            }
        }
        InvariantAction::Swap {
            pool_key,
            x_to_y,
            amount,
            by_amount_in,
            sqrt_price_limit,
        } => {
            match invariant
                .swap(pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit)
                .await
            {
                Ok(result) => {
                    reply(InvariantEvent::SwapReturn(result), 0).expect("Unable to reply");
                }
                Err(e) => {
                    reply_with_err(e);
                }
            }
        }
        InvariantAction::Quote {
            pool_key,
            x_to_y,
            amount,
            by_amount_in,
            sqrt_price_limit,
        } => match invariant.quote(pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit) {
            Ok(result) => {
                reply(InvariantEvent::Quote(result), 0).expect("Unable to reply");
            }
            Err(e) => {
                reply_with_err(e);
            }
        },
        InvariantAction::ClaimFee { position_id } => match invariant.claim_fee(position_id).await {
            Ok((amount_x, amount_y)) => {
                reply(InvariantEvent::ClaimFee(amount_x, amount_y), 0).expect("Unable to reply");
            }
            Err(e) => {
                reply_with_err(e);
            }
        },
        InvariantAction::QuoteRoute { amount_in, swaps } => {
            match invariant.quote_route(amount_in, swaps) {
                Ok(amount_out) => {
                    reply(InvariantEvent::QuoteRoute(amount_out), 0).expect("Unable to reply");
                }
                Err(e) => {
                    reply_with_err(e);
                }
            }
        }
        InvariantAction::WithdrawProtocolFee(pool_key) => {
            match invariant.withdraw_protocol_fee(pool_key).await {
                Ok(_) => {}
                Err(e) => {
                    reply_with_err(e);
                }
            }
        }
        InvariantAction::ClaimLostTokens { token } => {
            match invariant.claim_lost_tokens(&token).await {
                Ok(_) => {}
                Err(e) => {
                    reply_with_err(e);
                }
            }
        }
    }
}
#[no_mangle]
extern "C" fn state() {
    let query: InvariantStateQuery = msg::load().expect("Unable to decode InvariantStateQuery");
    let invariant = unsafe { INVARIANT.get_or_insert(Default::default()) };
    match query {
        InvariantStateQuery::FeeTierExist(fee_tier) => {
            let exists = invariant.fee_tier_exists(fee_tier);
            reply(InvariantStateReply::FeeTierExist(exists), 0).expect("Unable to reply");
        }
        InvariantStateQuery::GetFeeTiers => {
            let fee_tiers = invariant.get_fee_tiers();
            reply(InvariantStateReply::QueriedFeeTiers(fee_tiers), 0).expect("Unable to reply");
        }
        InvariantStateQuery::GetProtocolFee => {
            let protocol_fee = invariant.get_protocol_fee();
            reply(InvariantStateReply::ProtocolFee(protocol_fee), 0).expect("Unable to reply");
        }
        InvariantStateQuery::GetPool(token_0, token_1, fee_tier) => {
            match invariant.get_pool(token_0, token_1, fee_tier) {
                Ok(pool) => {
                    reply(InvariantStateReply::Pool(pool), 0).expect("Unable to reply");
                }
                Err(e) => {
                    reply(InvariantStateReply::QueryFailed(e), 0).expect("Unable to reply");
                }
            }
        }
        InvariantStateQuery::GetPools(size, offset) => match invariant.get_pools(size, offset) {
            Ok(pool_keys) => {
                reply(InvariantStateReply::Pools(pool_keys), 0).expect("Unable to reply");
            }
            Err(e) => {
                reply(InvariantStateReply::QueryFailed(e), 0).expect("Unable to reply");
            }
        },
        InvariantStateQuery::GetPosition(owner_id, index) => {
            match invariant.get_position(&owner_id, index) {
                Ok(position) => {
                    reply(InvariantStateReply::Position(position), 0).expect("Unable to reply");
                }
                Err(e) => {
                    reply(InvariantStateReply::QueryFailed(e), 0).expect("Unable to reply");
                }
            }
        }
        InvariantStateQuery::GetTick(pool_key, index) => {
            match invariant.get_tick(pool_key, index) {
                Ok(tick) => {
                    reply(InvariantStateReply::Tick(tick), 0).expect("Unable to reply");
                }
                Err(e) => {
                    reply(InvariantStateReply::QueryFailed(e), 0).expect("Unable to reply");
                }
            }
        }
        InvariantStateQuery::IsTickInitialized(pool_key, index) => {
            reply(
                InvariantStateReply::IsTickInitialized(
                    invariant.is_tick_initialized(pool_key, index),
                ),
                0,
            )
            .expect("Unable to reply");
        }
        InvariantStateQuery::GetAllPositions(owner_id) => {
            reply(
                InvariantStateReply::Positions(invariant.get_all_positions(&owner_id)),
                0,
            )
            .expect("Unable to reply");
        }
        InvariantStateQuery::GetUserBalances(user) => {
            reply(
                InvariantStateReply::UserBalances(invariant.get_user_balances(&user)),
                0,
            )
            .expect("Unable to reply");
        }
    }
}
