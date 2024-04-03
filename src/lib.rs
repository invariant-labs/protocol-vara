#![no_std]
extern crate alloc;
#[cfg(test)]
mod e2e;
#[cfg(test)]
mod test_helpers;

use contracts::{errors::InvariantError, FeeTier, FeeTiers, Pool, PoolKey, PoolKeys, Pools, Position, Positions, Tick, Tickmap, Ticks};
use decimal::*;
use gstd::{
    async_main, exec, msg::{self, reply}, prelude::*, ActorId
};
use io::*;
use math::{check_tick, percentage::Percentage, sqrt_price::SqrtPrice, liquidity::Liquidity };
use fungible_token_io::{FTAction, FTError, FTEvent};


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
}

impl Invariant {
    pub fn change_protocol_fee(&mut self, protocol_fee: u128) -> Result<u128, InvariantError> {
        if !self.is_caller_admin() {
            return Err(InvariantError::NotAdmin);
        }

        self.config.protocol_fee = protocol_fee;
        Ok(self.config.protocol_fee)
    }

    pub fn get_protocol_fee(&self) -> u128 {
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
    )->Result<(),InvariantError> {
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
        fee_tier: FeeTier
    )->Result<Pool, InvariantError> {
        let pool_key = PoolKey::new(token_0, token_1, fee_tier)?;
        self.pools.get(&pool_key)
    }

    pub fn get_pools(&self, size: u8, offset: u16) ->  Result<Vec<PoolKey>, InvariantError> {
        self.pool_keys.get_all(size, offset)
    }

    pub fn change_fee_receiver(&mut self, pool_key: PoolKey, fee_receiver: ActorId) -> Result<(), InvariantError> {
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
        let caller = msg::source();
        let program = exec::program_id();
        let current_timestamp = exec::block_timestamp();
        let current_block_number = exec::block_height() as u64;

        // liquidity delta = 0 => return
        if liquidity_delta == Liquidity::new(0) {
            return Err(InvariantError::ZeroLiquidity);
        }

        let mut pool = self.pools.get(&pool_key)?;

        let mut lower_tick = self
            .ticks
            .get(pool_key, lower_tick)
            .cloned()
            .unwrap_or_else(|_| Self::create_tick(self, pool_key, lower_tick).unwrap());

        let mut upper_tick = self
            .ticks
            .get(pool_key, upper_tick)
            .cloned()
            .unwrap_or_else(|_| Self::create_tick(self, pool_key, upper_tick).unwrap());

        let undo_ticks_creation = |invariant: &mut Self, remove_lower_tick: bool, lower_tick: Tick, remove_upper_tick: bool, upper_tick: Tick| {
            if remove_lower_tick {
                invariant.remove_tick(pool_key, lower_tick)
                    .expect("Attempted to remove incorrect tick");
            } 
            if remove_upper_tick {
                invariant.remove_tick(pool_key, upper_tick)
                    .expect("Attempted to remove incorrect tick");
            }
        };

        let (mut position, x, y) = match Position::create(
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
        ) {
            Ok(position) => position,
            Err(e) => {
                undo_ticks_creation(
                    self,
                    lower_tick.liquidity_gross.is_zero(),
                    lower_tick,
                    upper_tick.liquidity_gross.is_zero(),
                    upper_tick
                );
                return Err(e);
            }
        };

        let undo_creating_position = |invariant: &mut Self,
                                      position: &mut Position,
                                      pool: &mut Pool,
                                      mut lower_tick: Tick,
                                      mut upper_tick: Tick,
                                      pool_key: PoolKey| {
            let (_x, _y, remove_lower_tick, remove_upper_tick) = position.remove(
                pool,
                current_timestamp,
                &mut lower_tick,
                &mut upper_tick,
                pool_key.fee_tier.tick_spacing,
            );

            undo_ticks_creation(invariant, remove_lower_tick, lower_tick, remove_upper_tick, upper_tick);
        };

        let first_transaction = self
            .transfer_tokens(&pool_key.token_x, None, &caller, &program, x.get())
            .await;

        if let Err(e) = first_transaction {
            undo_creating_position(
                self,
                &mut position,
                &mut pool,
                lower_tick,
                upper_tick,
                pool_key,
            );

            return Err(e);
        }
        let second_transaction = self
            .transfer_tokens(&pool_key.token_y, None, &caller, &program, y.get())
            .await;

        if let Err(e) = second_transaction {
            undo_creating_position(
                self,
                &mut position,
                &mut pool,
                lower_tick,
                upper_tick,
                pool_key,
            );
            self.transfer_tokens(&pool_key.token_x, None, &program, &caller, x.get())
                .await
                .unwrap();

            return Err(e);
        }

        self.pools.update(&pool_key, &pool)?;

        self.positions.add(&caller, &position);

        self.ticks.update(pool_key, lower_tick.index, lower_tick)?;
        self.ticks.update(pool_key, upper_tick.index, upper_tick)?;
     
        self.emit_event(InvariantEvent::PositionCreatedEvent {
            block_timestamp: exec::block_timestamp(),
            address: msg::source(),
            pool_key,
            liquidity_delta,
            lower_tick: lower_tick.index,
            upper_tick: upper_tick.index,
            current_sqrt_price: pool.sqrt_price 
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

    async fn transfer_tokens(
        &mut self,
        token_address: &ActorId,
        tx_id: Option<u64>,
        from: &ActorId,
        to: &ActorId,
        amount_tokens: u128,
    ) -> Result<(), InvariantError> {
        let tx_id = tx_id.or(self.generate_transaction_id().into());
        let reply = msg::send_for_reply_as::<_, Result<FTEvent, FTError>>(
            *token_address,
            FTAction::Transfer {
                tx_id,
                from: *from,
                to: *to,
                amount: amount_tokens,
            },
            0,
            0,
        )
        .map_err(|_| InvariantError::TransferError)?
        .await
        .map_err(|_| InvariantError::TransferError)?;

        match reply {
            Ok(ft_event) => {
                match ft_event {
                FTEvent::Transfer { from: _, to: _, amount: _ } => {
                        return Ok(())
                    },
                _ => return Err(InvariantError::TransferError)
                }
            }
            Err(_ft_error) => {
                return Err(InvariantError::TransferError)
            }
    
        }
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
        self.ticks.add(pool_key, index, tick)?;

        self.tickmap
            .flip(true, index, pool_key.fee_tier.tick_spacing, pool_key);

        Ok(tick)
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

#[no_mangle]
extern "C" fn init() {
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
                    reply(InvariantEvent::ActionFailed(e), 0).expect("Unable to reply");
                }
            };
        }
        InvariantAction::AddFeeTier(fee_tier) => {
            match invariant.add_fee_tier(fee_tier) {
                Ok(_fee_tier) => {}
                Err(e) => {
                    reply(InvariantEvent::ActionFailed(e), 0).expect("Unable to reply");
                }
            };
        }
        InvariantAction::RemoveFeeTier(fee_tier) => {
            match invariant.remove_fee_tier(fee_tier) {
                Ok(_fee_tier) => {}
                Err(e) => {
                    reply(InvariantEvent::ActionFailed(e), 0).expect("Unable to reply");
                }
            };
        }
        InvariantAction::CreatePool {
            token_0,
            token_1,
            fee_tier,
            init_sqrt_price,
            init_tick,
        } => {
            match invariant.create_pool(token_0, token_1, fee_tier, init_sqrt_price, init_tick) {
            Ok(_fee_tier) => {}
            Err(e) => {
                reply(InvariantEvent::ActionFailed(e), 0).expect("Unable to reply");
            }
        }
        }
        InvariantAction::ChangeFeeReceiver(pool_key, fee_receiver) => {
            match invariant.change_fee_receiver(pool_key, fee_receiver) {
                Ok(_) => {}
                Err(e) => {
                    reply(InvariantEvent::ActionFailed(e), 0).expect("Unable to reply");
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
                    reply(InvariantEvent::PositionCreatedReturn(position), 0).expect("Unable to reply");
                }
                Err(e) => {
                                        reply(InvariantEvent::ActionFailed(e), 0).expect("Unable to reply");
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
        InvariantStateQuery::GetPools(size, offset) => {
            match invariant.get_pools(size, offset) {
                Ok(pool_keys) => {
                    reply(InvariantStateReply::Pools(pool_keys), 0).expect("Unable to reply");
                }
                Err(e) => {
                    reply(InvariantStateReply::QueryFailed(e), 0).expect("Unable to reply");
                }
            }
        }
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
            reply(InvariantStateReply::IsTickInitialized(invariant.is_tick_initialized(pool_key, index)), 0)
            .expect("Unable to reply");
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use gtest::{Program, System};
    use test_helpers::consts::INVARIANT_PATH;
    use math::sqrt_price::calculate_sqrt_price;
    const ADMIN: u64 = 1;
    const USERS: [u64; 3] = [2, 3, 4];

    const REGULAR_USER_1: u64 = USERS[0];
    const REGULAR_USER_2: u64 = USERS[1];

    const PROGRAM_OWNER: u64 = USERS[2];
    const PROGRAM_ID: u64 = 105;

    pub fn init_invariant(sys: &System, protocol_fee: u128) -> Program<'_> {
        let program = Program::from_file_with_id(sys, PROGRAM_ID, INVARIANT_PATH);

        assert!(!program
            .send(
                PROGRAM_OWNER,
                InitInvariant {
                    config: InvariantConfig {
                        admin: ADMIN.into(),
                        protocol_fee,
                    },
                },
            )
            .main_failed());
        program
    }

    #[test]
    fn test_init() {
        let sys = System::new();
        sys.init_logger();

        let _invariant = init_invariant(&sys, 100);
    }
    #[test]
    fn test_fee_tiers() {
        let sys = System::new();
        sys.init_logger();

        let invariant = init_invariant(&sys, 100);
        let fee_tier = FeeTier::new(Percentage::new(1), 10u16).unwrap();
        let fee_tier_value = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 10u16,
        };

        let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
        assert!(!res.main_failed());

        let state: InvariantStateReply = invariant
            .read_state(InvariantStateQuery::GetFeeTiers)
            .expect("Failed to read state");
        assert_eq!(state, InvariantStateReply::QueriedFeeTiers(vec![fee_tier_value]));

        let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
        assert!(!res.main_failed());
        assert!(res.contains(&(
            ADMIN,
            InvariantEvent::ActionFailed(InvariantError::FeeTierAlreadyExist).encode()
        )));

        let state: InvariantStateReply = invariant
            .read_state(InvariantStateQuery::GetFeeTiers)
            .expect("Failed to read state");
        assert_eq!(state, InvariantStateReply::QueriedFeeTiers(vec![fee_tier_value]));

        let res = invariant.send(ADMIN, InvariantAction::RemoveFeeTier(fee_tier));
        assert!(!res.main_failed());

        let state: InvariantStateReply = invariant
            .read_state(InvariantStateQuery::GetFeeTiers)
            .expect("Failed to read state");
        assert_eq!(state, InvariantStateReply::QueriedFeeTiers(vec![]));
    }

    #[test]
    fn test_add_pool() {
        let sys = System::new();
        sys.init_logger();

        let invariant = init_invariant(&sys, 0);

        let token_0 = ActorId::from([0x01; 32]);
        let token_1 = ActorId::from([0x02; 32]);
        let fee_tier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
        assert!(!res.main_failed());
        assert!(res.log().last().unwrap().payload().is_empty());

        let init_sqrt_price = calculate_sqrt_price(0).unwrap();

        let res = invariant.send(REGULAR_USER_1, InvariantAction::CreatePool{
                token_0,
                token_1,
                fee_tier,
                init_sqrt_price,
                init_tick: 0,
            });
        assert!(!res.main_failed());
        assert!(res.log().last().unwrap().payload().is_empty());

        let res = invariant.send(REGULAR_USER_2, InvariantAction::CreatePool{
                token_0,
                token_1,
                fee_tier,
                init_sqrt_price,
                init_tick: 0,
            });

        assert!(!res.main_failed());
        assert!(res.contains(&(
            USERS[1],
            InvariantEvent::ActionFailed(InvariantError::PoolAlreadyExist).encode()
        )));
    }

    #[test]
    fn test_get_pool() {
        let sys = System::new();
        sys.init_logger();

        let invariant = init_invariant(&sys, 100);

        let token_0 = ActorId::from([0x01; 32]);
        let token_1 = ActorId::from([0x02; 32]);
        let fee_tier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
        assert!(!res.main_failed());
        assert!(res.log().last().unwrap().payload().is_empty());

        let init_sqrt_price = calculate_sqrt_price(0).unwrap();
        let init_tick = 0;

        let block_timestamp = sys.block_timestamp();

        let res = invariant.send(REGULAR_USER_1, InvariantAction::CreatePool{
                token_0,
                token_1,
                fee_tier,
                init_sqrt_price,
                init_tick,
            });
        assert!(!res.main_failed());

        let state: InvariantStateReply = invariant
            .read_state(InvariantStateQuery::GetPool(token_0, token_1, fee_tier))
            .expect("Failed to read state");
        assert_eq!(state, InvariantStateReply::Pool(Pool{
                start_timestamp: block_timestamp,
                last_timestamp: block_timestamp,
                sqrt_price: init_sqrt_price,
                current_tick_index: init_tick,
                fee_receiver: ADMIN.into(),
                ..Pool::default()
            }))
    }

    #[test]
    fn test_get_pools() {
        let sys = System::new();
        sys.init_logger();

        let invariant = init_invariant(&sys, 100);

        let token_0 = ActorId::from([0x01; 32]);
        let token_1 = ActorId::from([0x02; 32]);
        let fee_tier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
        assert!(!res.main_failed());
        assert!(res.log().last().unwrap().payload().is_empty());

        let init_sqrt_price = calculate_sqrt_price(0).unwrap();
        let init_tick = 0;

        let res = invariant.send(REGULAR_USER_1, InvariantAction::CreatePool{
                token_0,
                token_1,
                fee_tier,
                init_sqrt_price,
                init_tick,
            });
        assert!(!res.main_failed());

        let state: InvariantStateReply = invariant
            .read_state(InvariantStateQuery::GetPools(u8::MAX, 0))
            .expect("Failed to read state");
        assert_eq!(state, InvariantStateReply::Pools(vec![PoolKey::new(token_0, token_1, fee_tier).unwrap()]))
    }

    #[test]
    fn test_change_fee_receiver() {
        let sys = System::new();
        sys.init_logger();

        let invariant = init_invariant(&sys, 0);

        let token_0 = ActorId::from([0x01; 32]);
        let token_1 = ActorId::from([0x02; 32]);
        let fee_tier = FeeTier {
            fee: Percentage::new(1),
            tick_spacing: 1,
        };
        let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
        assert!(!res.main_failed());
        assert!(res.log().last().unwrap().payload().is_empty());

        let init_sqrt_price = calculate_sqrt_price(0).unwrap();

        let res = invariant.send(
            REGULAR_USER_1,
            InvariantAction::CreatePool {
                token_0,
                token_1,
                fee_tier,
                init_sqrt_price,
                init_tick: 0,
            });
        assert!(!res.main_failed());
        assert!(res.log().last().unwrap().payload().is_empty());
        let pool_key = PoolKey::new(token_0, token_1, fee_tier).unwrap();
        let res = invariant.send(ADMIN, InvariantAction::ChangeFeeReceiver(pool_key, REGULAR_USER_1.into()));

        assert!(!res.main_failed());
        assert!(res.log().last().unwrap().payload().is_empty());

        let state: InvariantStateReply = invariant
            .read_state(InvariantStateQuery::GetPool(token_0, token_1, fee_tier))
            .expect("Failed to read state");

        if let InvariantStateReply::Pool(pool) = state {
            assert_eq!(pool.fee_receiver, REGULAR_USER_1.into());
        }
    }
}
