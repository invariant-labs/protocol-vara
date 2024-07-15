extern crate alloc;
use crate::invariant_storage::Invariant;
use crate::invariant_storage::InvariantStorage;
use contracts::{
    get_bit_at_position, get_max_chunk, get_min_chunk, position_to_tick, tick_to_position,
    AwaitingTransfer, FeeTier, InvariantError, LiquidityTick, Pool, PoolKey, Position, Tick,
    TransferType, CHUNK_SIZE,
};
use decimal::*;
use futures;
use gstd::{exec, format, prelude::*, String};
use io::*;
use math::sqrt_price::get_max_tick;
use math::{
    check_tick, liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice,
    token_amount::TokenAmount, MAX_SQRT_PRICE, MIN_SQRT_PRICE,
};
use sails_rtl::gstd::{
    gservice,
    msg::{self, reply, CodecMessageFuture},
    ExecContext,
};
use sails_rtl::{ActorId, Decode, Encode, MessageId};

fn program_id() -> ActorId {
    exec::program_id().into()
}
pub fn panic(err: InvariantError) -> ! {
    let str: String = err.into();
    panic!("{}", str)
}

macro_rules! panicking {
    ($expr:expr) => {{
        let result: Result<_, InvariantError> = $expr();
        match result {
            Ok(value) => value,
            Err(e) => panic(e),
        }
    }};
}

macro_rules! panicking_async {
    ($expr:expr) => {{
        let result: Result<_, InvariantError> = $expr().await;
        match result {
            Ok(value) => value,
            Err(e) => panic(e),
        }
    }};
}

type TokenTransferResponse = (String, String, bool);

// TODO: Update once the SDK tests are in place and proper measurement is possible
pub const TRANSFER_GAS_LIMIT: u64 = 10_600_000_000 * 2;
pub const TRANSFER_REPLY_HANDLING_COST: u64 = 10_600_000_000 * 2;
pub const BALANCE_CHANGE_COST: u64 = 100_000 * 2;
pub const TRANSFER_COST: u64 =
    TRANSFER_GAS_LIMIT + TRANSFER_REPLY_HANDLING_COST + BALANCE_CHANGE_COST;

pub struct InvariantService<TExecContext> {
    exec_context: TExecContext,
}

// methods are split from main impl block to avoid having them added as methods by gservice macro
impl<TExecContext> InvariantService<TExecContext> {
    pub fn seed(config: InvariantConfig) {
        InvariantStorage::with_config(config).expect("Invariant storage already set")
    }
}

#[gservice(events=InvariantEvent)]
impl<TExecContext> InvariantService<TExecContext>
where
    TExecContext: ExecContext,
{
    pub fn new(exec_context: TExecContext) -> Self {
        Self { exec_context }
    }

    pub fn change_protocol_fee(&mut self, protocol_fee: Percentage) -> Percentage {
        panicking!(move || {
            let invariant = InvariantStorage::as_mut();

            if !self.is_caller_admin(&invariant) {
                return Err(InvariantError::NotAdmin);
            }

            invariant.config.protocol_fee = protocol_fee;

            Ok(invariant.config.protocol_fee)
        })
    }

    pub fn get_protocol_fee(&self) -> Percentage {
        InvariantStorage::as_ref().config.protocol_fee
    }

    pub fn add_fee_tier(&mut self, fee_tier: FeeTier) -> FeeTier {
        panicking!(move || {
            let invariant = InvariantStorage::as_mut();

            if fee_tier.tick_spacing == 0 || fee_tier.tick_spacing > 100 {
                return Err(InvariantError::InvalidTickSpacing);
            }

            if fee_tier.fee >= Percentage::from_integer(1) {
                return Err(InvariantError::InvalidFee);
            }

            if !self.is_caller_admin(&invariant) {
                return Err(InvariantError::NotAdmin);
            }

            invariant.fee_tiers.add(&fee_tier)?;
            Ok(fee_tier)
        })
    }

    pub fn fee_tier_exists(&self, fee_tier: FeeTier) -> bool {
        InvariantStorage::as_ref().fee_tiers.contains(&fee_tier)
    }

    pub fn remove_fee_tier(&mut self, fee_tier: FeeTier) -> FeeTier {
        panicking!(move || {
            let invariant = InvariantStorage::as_mut();

            if !self.is_caller_admin(&invariant) {
                return Err(InvariantError::NotAdmin);
            }

            invariant.fee_tiers.remove(&fee_tier)?;
            Ok(fee_tier)
        })
    }

    pub fn get_fee_tiers(&self) -> Vec<FeeTier> {
        InvariantStorage::as_mut().fee_tiers.get_all()
    }

    pub fn create_pool(
        &mut self,
        token_x: ActorId,
        token_y: ActorId,
        fee_tier: FeeTier,
        init_sqrt_price: SqrtPrice,
        init_tick: i32,
    ) {
        panicking!(move || {
            let invariant = InvariantStorage::as_mut();
            let current_timestamp = exec::block_timestamp();

            if !invariant.fee_tiers.contains(&fee_tier) {
                return Err(InvariantError::FeeTierNotFound);
            };

            check_tick(init_tick, fee_tier.tick_spacing)
                .map_err(|_| InvariantError::InvalidInitTick)?;

            let pool_key = PoolKey::new(token_x, token_y, fee_tier)?;

            if invariant.pools.get(&pool_key).is_ok() {
                return Err(InvariantError::PoolAlreadyExist);
            };

            let pool = Pool::create(
                init_sqrt_price,
                init_tick,
                current_timestamp,
                fee_tier.tick_spacing,
                invariant.config.admin,
            )?;
            invariant.pools.add(&pool_key, &pool)?;
            invariant.pool_keys.add(&pool_key)?;

            Ok(())
        })
    }

    pub fn get_pool(
        &self,
        token_x: ActorId,
        token_y: ActorId,
        fee_tier: FeeTier,
    ) -> Result<Pool, InvariantError> {
        let invariant = InvariantStorage::as_ref();

        let pool_key = PoolKey::new(token_x, token_y, fee_tier)?;
        invariant.pools.get(&pool_key)
    }

    pub fn get_pools(&self, size: u8, offset: u16) -> Result<Vec<PoolKey>, InvariantError> {
        InvariantStorage::as_ref().pool_keys.get_all(size, offset)
    }

    pub fn change_fee_receiver(&mut self, pool_key: PoolKey, fee_receiver: ActorId) {
        panicking!(move || {
            let invariant = InvariantStorage::as_mut();

            if !self.is_caller_admin(&invariant) {
                return Err(InvariantError::NotAdmin);
            }

            let mut pool = invariant.pools.get(&pool_key)?;
            pool.fee_receiver = fee_receiver;
            invariant.pools.update(&pool_key, &pool)?;

            Ok(())
        })
    }

    pub fn create_position(
        &mut self,
        pool_key: PoolKey,
        lower_tick: i32,
        upper_tick: i32,
        liquidity_delta: Liquidity,
        slippage_limit_lower: SqrtPrice,
        slippage_limit_upper: SqrtPrice,
    ) -> Position {
        panicking!(move || {
            let invariant = InvariantStorage::as_mut();

            let caller = self.exec_context.actor_id();
            let current_timestamp = exec::block_timestamp();
            let current_block_number = exec::block_height() as u64;

            // liquidity delta = 0 => return
            if liquidity_delta == Liquidity::new(U256::from(0)) {
                return Err(InvariantError::ZeroLiquidity);
            }

            if lower_tick == upper_tick {
                return Err(InvariantError::InvalidTickIndex);
            }

            let mut pool = invariant.pools.get(&pool_key)?;

            let (mut lower_tick, should_add_lower) =
                invariant.get_or_create_tick(pool_key, lower_tick);
            let (mut upper_tick, should_add_upper) =
                invariant.get_or_create_tick(pool_key, upper_tick);

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

            invariant.decrease_token_balance(&pool_key.token_x, &caller, x.into())?;
            invariant.decrease_token_balance(&pool_key.token_y, &caller, y.into())?;

            invariant.pools.update(&pool_key, &pool)?;

            invariant.positions.add(&caller, &position);

            if should_add_lower {
                invariant.add_tick(pool_key, lower_tick)?;
            } else {
                invariant
                    .ticks
                    .update(pool_key, lower_tick.index, lower_tick)?;
            }

            if should_add_upper {
                invariant.add_tick(pool_key, upper_tick)?;
            } else {
                invariant
                    .ticks
                    .update(pool_key, upper_tick.index, upper_tick)?;
            }

            self.notify_on(InvariantEvent::PositionCreatedEvent {
                timestamp: exec::block_timestamp(),
                address: self.exec_context.actor_id(),
                pool_key,
                liquidity_delta,
                lower_tick: lower_tick.index,
                upper_tick: upper_tick.index,
                current_sqrt_price: pool.sqrt_price,
            })
            .expect("Failed to notify on position created event");

            Ok(position)
        })
    }

    pub fn get_position(&self, owner_id: ActorId, index: u32) -> Result<Position, InvariantError> {
        InvariantStorage::as_ref()
            .positions
            .get(&owner_id, index)
            .cloned()
    }

    pub fn get_tick(&self, key: PoolKey, index: i32) -> Result<Tick, InvariantError> {
        InvariantStorage::as_ref().ticks.get(key, index).cloned()
    }

    pub fn is_tick_initialized(&self, key: PoolKey, index: i32) -> bool {
        InvariantStorage::as_ref()
            .tickmap
            .get(index, key.fee_tier.tick_spacing, key)
    }
    pub fn remove_position(&mut self, index: u32) -> (TokenAmount, TokenAmount) {
        panicking!(move || {
            let invariant = InvariantStorage::as_mut();

            let caller = self.exec_context.actor_id();
            let current_timestamp = exec::block_timestamp();

            let mut position = invariant.positions.get(&caller, index).cloned()?;
            let Position {
                pool_key,
                lower_tick_index,
                upper_tick_index,
                liquidity: withdrawn_liquidity,
                ..
            } = position;

            let mut lower_tick = invariant.ticks.get(pool_key, lower_tick_index).cloned()?;

            let mut upper_tick = invariant.ticks.get(pool_key, upper_tick_index).cloned()?;

            let pool = &mut invariant.pools.get(&pool_key)?;

            let (amount_x, amount_y, remove_lower_tick, remove_upper_tick) = position.remove(
                pool,
                current_timestamp,
                &mut lower_tick,
                &mut upper_tick,
                pool_key.fee_tier.tick_spacing,
            );

            invariant.pools.update(&pool_key, pool)?;

            if remove_lower_tick {
                invariant.remove_tick(pool_key, lower_tick)?;
            } else {
                invariant
                    .ticks
                    .update(pool_key, lower_tick_index, lower_tick)?;
            }

            if remove_upper_tick {
                invariant.remove_tick(pool_key, upper_tick)?;
            } else {
                invariant
                    .ticks
                    .update(pool_key, upper_tick_index, upper_tick)?;
            }

            invariant.positions.remove(&caller, index)?;

            let token_x = pool_key.token_x;
            let token_y = pool_key.token_y;

            invariant.increase_token_balance(&token_x, &caller, amount_x)?;
            invariant.increase_token_balance(&token_y, &caller, amount_y)?;

            self.notify_on(InvariantEvent::PositionRemovedEvent {
                timestamp: exec::block_timestamp(),
                address: self.exec_context.actor_id(),
                pool_key,
                liquidity: withdrawn_liquidity,
                lower_tick_index: lower_tick.index,
                upper_tick_index: upper_tick.index,
                sqrt_price: pool.sqrt_price,
            })
            .expect("Failed to notify on position removed event");

            Ok((amount_x, amount_y))
        })
    }

    pub fn transfer_position(&mut self, index: u32, receiver: ActorId) {
        panicking!(move || {
            InvariantStorage::as_mut().positions.transfer(
                &self.exec_context.actor_id(),
                index,
                &receiver,
            )
        })
    }

    pub fn get_all_positions(&self, owner_id: ActorId) -> Vec<Position> {
        InvariantStorage::as_ref().positions.get_all(&owner_id)
    }

    pub fn swap(
        &mut self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> CalculateSwapResult {
        panicking!(move || {
            let invariant = InvariantStorage::as_mut();

            let caller = self.exec_context.actor_id();

            let calculate_swap_result = invariant.calculate_swap(
                pool_key,
                x_to_y,
                amount,
                by_amount_in,
                sqrt_price_limit,
            )?;

            let mut crossed_tick_indexes: Vec<i32> = vec![];

            for tick in calculate_swap_result.ticks.iter() {
                crossed_tick_indexes.push(tick.index);
                invariant.ticks.update(pool_key, tick.index, *tick)?;
            }

            invariant
                .pools
                .update(&pool_key, &calculate_swap_result.pool)?;

            let (swapped_token, returned_token) = if x_to_y {
                (&pool_key.token_x, &pool_key.token_y)
            } else {
                (&pool_key.token_y, &pool_key.token_x)
            };

            invariant.decrease_token_balance(
                &swapped_token,
                &caller,
                calculate_swap_result.amount_in.into(),
            )?;

            invariant.increase_token_balance(
                &returned_token,
                &caller,
                calculate_swap_result.amount_out.into(),
            )?;

            if !crossed_tick_indexes.is_empty() {
                self.notify_on(InvariantEvent::CrossTickEvent {
                    timestamp: exec::block_timestamp(),
                    address: caller,
                    pool_key,
                    indexes: crossed_tick_indexes,
                })
                .expect("Failed to notify on cross tick event");
            }

            self.notify_on(InvariantEvent::SwapEvent {
                timestamp: exec::block_timestamp(),
                address: caller,
                pool_key,
                amount_in: calculate_swap_result.amount_in,
                amount_out: calculate_swap_result.amount_out,
                fee: calculate_swap_result.fee,
                start_sqrt_price: calculate_swap_result.start_sqrt_price,
                target_sqrt_price: calculate_swap_result.target_sqrt_price,
                x_to_y,
            })
            .expect("Failed to notify on swap event");

            Ok(calculate_swap_result)
        })
    }

    pub fn quote(
        &self,
        pool_key: PoolKey,
        x_to_y: bool,
        amount: TokenAmount,
        by_amount_in: bool,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<QuoteResult, InvariantError> {
        let invariant = InvariantStorage::as_ref();

        let calculate_swap_result =
            invariant.calculate_swap(pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit)?;

        Ok(QuoteResult {
            amount_in: calculate_swap_result.amount_in,
            amount_out: calculate_swap_result.amount_out,
            target_sqrt_price: calculate_swap_result.pool.sqrt_price,
            ticks: calculate_swap_result.ticks,
        })
    }

    pub fn quote_route(
        &self,
        amount_in: TokenAmount,
        swaps: Vec<SwapHop>,
    ) -> Result<TokenAmount, InvariantError> {
        Self::route(amount_in, swaps)
    }

    pub fn route(
        amount_in: TokenAmount,
        swaps: Vec<SwapHop>,
    ) -> Result<TokenAmount, InvariantError> {
        let invariant = InvariantStorage::as_ref();

        let mut next_swap_amount = amount_in;

        for swap in swaps.iter() {
            let SwapHop { pool_key, x_to_y } = *swap;

            let sqrt_price_limit = if x_to_y {
                SqrtPrice::new(MIN_SQRT_PRICE.into())
            } else {
                SqrtPrice::new(MAX_SQRT_PRICE.into())
            };

            let result = invariant.calculate_swap(
                pool_key,
                x_to_y,
                next_swap_amount,
                true,
                sqrt_price_limit,
            )?;

            next_swap_amount = result.amount_out;
        }

        Ok(next_swap_amount)
    }

    pub fn claim_fee(&mut self, index: u32) -> (TokenAmount, TokenAmount) {
        panicking!(move || {
            let invariant = InvariantStorage::as_mut();

            let caller = self.exec_context.actor_id();
            let current_timestamp = exec::block_timestamp();

            let mut position = invariant.positions.get(&caller, index).cloned()?;

            let mut lower_tick = invariant
                .ticks
                .get(position.pool_key, position.lower_tick_index)
                .cloned()?;

            let mut upper_tick = invariant
                .ticks
                .get(position.pool_key, position.upper_tick_index)
                .cloned()?;

            let mut pool = invariant.pools.get(&position.pool_key)?;

            let (x, y) = position.claim_fee(
                &mut pool,
                &mut upper_tick,
                &mut lower_tick,
                current_timestamp,
            );

            invariant.positions.update(&caller, index, &position)?;
            invariant.pools.update(&position.pool_key, &pool)?;
            invariant
                .ticks
                .update(position.pool_key, upper_tick.index, upper_tick)?;
            invariant
                .ticks
                .update(position.pool_key, lower_tick.index, lower_tick)?;

            invariant.increase_token_balance(&position.pool_key.token_x, &caller, x)?;
            invariant.increase_token_balance(&position.pool_key.token_y, &caller, y)?;

            Ok((x, y))
        })
    }

    pub fn withdraw_protocol_fee(&mut self, pool_key: PoolKey) {
        panicking!(move || {
            let invariant = InvariantStorage::as_mut();

            let caller = self.exec_context.actor_id();

            let mut pool = invariant.pools.get(&pool_key)?;

            if pool.fee_receiver != caller {
                return Err(InvariantError::NotFeeReceiver);
            }

            let (amount_x, amount_y) = pool.withdraw_protocol_fee(pool_key);
            invariant.pools.update(&pool_key, &pool)?;

            invariant.increase_token_balance(&pool_key.token_x, &caller, amount_x)?;
            invariant.increase_token_balance(&pool_key.token_y, &caller, amount_y)?;

            Ok(())
        })
    }

    pub fn get_user_balances(&self, user: ActorId) -> Vec<(ActorId, TokenAmount)> {
        InvariantStorage::as_ref()
            .balances
            .get(&user)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect()
    }

    pub async fn deposit_single_token(
        &mut self,
        token: ActorId,
        amount: TokenAmount,
    ) -> TokenAmount {
        panicking_async!(|| async move {
            let invariant = InvariantStorage::as_mut();
            let caller = &self.exec_context.actor_id();

            if !invariant.can_increase_token_balance(&token, &caller, amount) {
                return Err(InvariantError::FailedToChangeTokenBalance);
            }

            Self::transfer_single_token(invariant, &token, &caller, amount, TransferType::Deposit)
                .await?;

            Ok(amount)
        })
    }

    pub async fn withdraw_single_token(
        &mut self,
        token: ActorId,
        amount: Option<TokenAmount>,
    ) -> TokenAmount {
        panicking_async!(|| async move {
            let invariant = InvariantStorage::as_mut();
            let caller = &self.exec_context.actor_id();

            let amount = invariant.decrease_token_balance(&token, &caller, amount)?;

            Self::transfer_single_token(
                invariant,
                &token,
                &caller,
                amount,
                TransferType::Withdrawal,
            )
            .await?;

            Ok(amount)
        })
    }

    pub async fn deposit_token_pair(
        &mut self,
        token_x: (ActorId, TokenAmount),
        token_y: (ActorId, TokenAmount),
    ) -> (TokenAmount, TokenAmount) {
        panicking_async!(|| async move {
            let invariant = InvariantStorage::as_mut();
            let caller = &self.exec_context.actor_id();

            if token_x.0.eq(&token_y.0) {
                return Err(InvariantError::TokensAreSame);
            }

            let transfer_type = TransferType::Deposit;

            if !invariant.can_increase_token_balance(&token_x.0, &caller, token_x.1)
                || !invariant.can_increase_token_balance(&token_y.0, &caller, token_y.1)
            {
                return Err(InvariantError::FailedToChangeTokenBalance);
            }

            if !token_x.1.is_zero() && !token_y.1.is_zero() {
                Self::transfer_token_pair(invariant, &caller, &token_x, &token_y, transfer_type)
                    .await?;
            } else if !token_x.1.is_zero() {
                Self::transfer_single_token(
                    invariant,
                    &token_x.0,
                    &caller,
                    token_x.1,
                    transfer_type,
                )
                .await?;
            } else if !token_y.1.is_zero() {
                Self::transfer_single_token(
                    invariant,
                    &token_y.0,
                    &caller,
                    token_y.1,
                    transfer_type,
                )
                .await?;
            }

            Ok((token_x.1, token_y.1))
        })
    }

    pub async fn withdraw_token_pair(
        &mut self,
        token_x: (ActorId, Option<TokenAmount>),
        token_y: (ActorId, Option<TokenAmount>),
    ) -> (TokenAmount, TokenAmount) {
        panicking_async!(|| async move {
            let invariant = InvariantStorage::as_mut();
            let caller = &self.exec_context.actor_id();

            if token_x.0.eq(&token_y.0) {
                return Err(InvariantError::TokensAreSame);
            }

            let transfer_type = TransferType::Withdrawal;
            let amount_x = invariant.decrease_token_balance(&token_x.0, &caller, token_x.1);
            let amount_y = invariant.decrease_token_balance(&token_y.0, &caller, token_y.1);

            let amount_x = if let Err(e) = amount_x {
                if e == InvariantError::NoBalanceForTheToken && token_x.1.is_none() {
                    TokenAmount::new(U256::from(0))
                } else {
                    return Err(e);
                }
            } else {
                amount_x?
            };

            let amount_y = if let Err(e) = amount_y {
                if e == InvariantError::NoBalanceForTheToken && token_y.1.is_none() {
                    TokenAmount::new(U256::from(0))
                } else {
                    return Err(e);
                }
            } else {
                amount_y?
            };

            if !amount_x.is_zero() && !amount_y.is_zero() {
                Self::transfer_token_pair(
                    invariant,
                    &caller,
                    &(token_x.0, amount_x),
                    &(token_y.0, amount_y),
                    transfer_type,
                )
                .await?;
            } else if !amount_x.is_zero() {
                Self::transfer_single_token(
                    invariant,
                    &token_x.0,
                    &caller,
                    amount_x,
                    transfer_type,
                )
                .await?;
            } else if !amount_y.is_zero() {
                Self::transfer_single_token(
                    invariant,
                    &token_y.0,
                    &caller,
                    amount_y,
                    transfer_type,
                )
                .await?;
            }

            Ok((amount_x, amount_y))
        })
    }

    fn is_caller_admin(&self, invariant_storage: &Invariant) -> bool {
        invariant_storage.config.admin == self.exec_context.actor_id()
    }

    async fn transfer_single_token(
        invariant: &mut Invariant,
        token: &ActorId,
        caller: &ActorId,
        amount: TokenAmount,
        transfer_type: TransferType,
    ) -> Result<(), InvariantError> {
        if exec::gas_available() < TRANSFER_COST {
            return Err(InvariantError::NotEnoughGasToExecute);
        }

        let program_id = &program_id();
        let (from, to) = match transfer_type {
            TransferType::Deposit => (caller, program_id),
            TransferType::Withdrawal => (program_id, caller),
        };

        let message =
            Self::send_transfer_token_message(invariant, token, from, to, amount, transfer_type)
                .map_err(|_| InvariantError::TransferError)?;

        let message_id = message.waiting_reply_to.into();

        let message = message.await;

        let transfer_check =
            Self::handle_transfer_result(invariant, message, message_id, *token, transfer_type);

        if transfer_check == Err(InvariantError::ReplyHandlingFailed) {
            reply_with_err_and_leave(InvariantError::ReplyHandlingFailed);
        }

        transfer_check
    }

    async fn transfer_token_pair(
        invariant: &mut Invariant,
        caller: &ActorId,
        token_x: &(ActorId, TokenAmount),
        token_y: &(ActorId, TokenAmount),
        transfer_type: TransferType,
    ) -> Result<(), InvariantError> {
        if exec::gas_available() < 2 * TRANSFER_COST {
            return Err(InvariantError::NotEnoughGasToExecute);
        }

        let program = &program_id();

        let (from, to) = match transfer_type {
            TransferType::Deposit => (caller, program),
            TransferType::Withdrawal => (program, caller),
        };

        let token_x_message = Self::send_transfer_token_message(
            invariant,
            &token_x.0,
            from,
            to,
            token_x.1,
            transfer_type,
        )?;

        let token_y_message = Self::send_transfer_token_message(
            invariant,
            &token_y.0,
            from,
            to,
            token_y.1,
            transfer_type,
        )?;

        let token_x_message_id = token_x_message.waiting_reply_to.into();
        let token_y_message_id = token_y_message.waiting_reply_to.into();

        let (token_x_message, token_y_message) = futures::join!(token_x_message, token_y_message);

        let token_x_check = Self::handle_transfer_result(
            invariant,
            token_x_message,
            token_x_message_id,
            token_x.0,
            transfer_type,
        );
        let token_y_check = Self::handle_transfer_result(
            invariant,
            token_y_message,
            token_y_message_id,
            token_y.0,
            transfer_type,
        );

        if token_x_check == Err(InvariantError::ReplyHandlingFailed)
            || token_y_check == Err(InvariantError::ReplyHandlingFailed)
        {
            reply_with_err_and_leave(InvariantError::ReplyHandlingFailed);
        }

        match transfer_type {
            TransferType::Deposit => match (token_x_check, token_y_check) {
                (Err(_), Ok(_)) | (Ok(_), Err(_)) => Err(InvariantError::RecoverableTransferError),
                (Err(_), Err(_)) => Err(InvariantError::UnrecoverableTransferError),
                _ => Ok(()),
            },
            TransferType::Withdrawal => match (token_x_check, token_y_check) {
                (Ok(_), Ok(_)) => Ok(()),
                _ => Err(InvariantError::RecoverableTransferError),
            },
        }
    }

    pub fn get_tickmap(&self, pool_key: PoolKey) -> Vec<(u16, u64)> {
        let tick_spacing = pool_key.fee_tier.tick_spacing;

        let max_chunk_index = get_max_chunk(tick_spacing);
        let min_chunk_index = get_min_chunk(tick_spacing);

        InvariantStorage::as_ref().tickmap_slice(min_chunk_index..=max_chunk_index, pool_key)
    }

    pub fn get_liquidity_ticks(&self, pool_key: PoolKey) -> Vec<LiquidityTick> {
        let mut ticks = vec![];
        let tick_spacing = pool_key.fee_tier.tick_spacing;

        let max_tick = get_max_tick(tick_spacing);
        let (chunk_limit, bit_limit) = tick_to_position(max_tick, tick_spacing);

        let invariant = InvariantStorage::as_ref();
        for i in 0..=chunk_limit {
            let chunk = invariant.tickmap.bitmap.get(&(i, pool_key)).unwrap_or(&0);

            if chunk != &0 {
                let end = if *chunk as u16 == chunk_limit {
                    bit_limit
                } else {
                    (CHUNK_SIZE - 1) as u8
                };

                for bit in 0..=end {
                    if get_bit_at_position(*chunk, bit) == 1 {
                        let tick_index = position_to_tick(i, bit, tick_spacing);

                        invariant
                            .ticks
                            .get(pool_key, tick_index)
                            .map(|tick| ticks.push(LiquidityTick::from(tick)))
                            .unwrap();
                    }
                }
            }
        }

        ticks
    }

    fn send_transfer_token_message(
        invariant: &mut Invariant,

        token_address: &ActorId,
        from: &ActorId,
        to: &ActorId,
        amount: TokenAmount,
        transfer_type: TransferType,
    ) -> Result<CodecMessageFuture<TokenTransferResponse>, InvariantError> {
        if amount == TokenAmount::new(U256::from(0)) {
            return Err(InvariantError::TransferError);
        }
        if from == to {
            return Err(InvariantError::TransferError);
        }

        let service_name = "Erc20".encode();
        let action = "TransferFrom".encode();

        let request = [
            service_name,
            action,
            from.encode(),
            to.encode(),
            amount.encode(),
        ]
        .concat();

        let message = msg::send_bytes_with_gas_for_reply_as::<_, TokenTransferResponse>(
            (*token_address).into(),
            request,
            TRANSFER_GAS_LIMIT,
            0,
            TRANSFER_REPLY_HANDLING_COST,
        )
        .map_err(|_| InvariantError::TransferError)?;

        let account = match transfer_type {
            TransferType::Deposit => *from,
            TransferType::Withdrawal => *to,
        };

        invariant.awaiting_transfers.insert(
            (message.waiting_reply_to.into(), *token_address),
            AwaitingTransfer {
                transfer_type,
                account,
                amount,
            },
        );

        Ok(message)
    }

    fn handle_transfer_result(
        invariant: &mut Invariant,
        message: Result<TokenTransferResponse, gstd::errors::Error>,
        message_id: MessageId,
        token: ActorId,
        transfer_type: TransferType,
    ) -> Result<(), InvariantError> {
        if invariant
            .awaiting_transfers
            .remove(&(message_id, token))
            .is_some()
        {
            return Err(InvariantError::ReplyHandlingFailed);
        }

        let err: InvariantError = match transfer_type {
            TransferType::Deposit => InvariantError::UnrecoverableTransferError,
            TransferType::Withdrawal => InvariantError::RecoverableTransferError,
        };

        let message = message.map_err(|_| err.clone())?;

        if !message.2 {
            return Err(err);
        };

        Ok(())
    }
}

fn reply_with_err_and_leave(err: InvariantError) {
    reply(err, 0).expect("Failed to send reply");
    exec::leave();
}
