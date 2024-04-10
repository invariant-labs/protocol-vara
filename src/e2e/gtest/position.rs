use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use fungible_token_io::FTAction;
use gstd::{prelude::*, ActorId};
use gtest::*;
use io::*;
use math::{
    fee_growth::FeeGrowth, liquidity::Liquidity, percentage::Percentage,
    sqrt_price::calculate_sqrt_price, token_amount::TokenAmount,
};

#[test]
fn test_create_position() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, 100);

    let (token_0_program, token_1_program) = init_tokens_with_mint(&sys, (500, 500));
    let token_0 = ActorId::from(TOKEN_X_ID);
    let token_1 = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 10).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0,
            token_1,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    assert!(!token_0_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: 500
            }
        )
        .main_failed());

    assert!(!token_1_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: 500
            }
        )
        .main_failed());

    let pool_key = PoolKey::new(token_0.into(), token_1.into(), fee_tier).unwrap();
    let pool = get_pool(&invariant, token_0, token_1, fee_tier).unwrap();

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: -10,
            upper_tick: 10,
            liquidity_delta: Liquidity::new(10),
            slippage_limit_lower: pool.sqrt_price,
            slippage_limit_upper: pool.sqrt_price,
        },
    );
    assert!(res.events_eq(vec![
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedEvent {
                address: REGULAR_USER_1.into(),
                pool_key,
                liquidity_delta: Liquidity::new(10),
                block_timestamp: sys.block_timestamp(),
                lower_tick: -10,
                upper_tick: 10,
                current_sqrt_price: init_sqrt_price,
            }
        ),
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedReturn(Position {
                pool_key,
                liquidity: Liquidity::new(10),
                lower_tick_index: -10,
                upper_tick_index: 10,
                fee_growth_inside_x: FeeGrowth::new(0),
                fee_growth_inside_y: FeeGrowth::new(0),
                last_block_number: sys.block_height() as u64,
                tokens_owed_x: TokenAmount(0),
                tokens_owed_y: TokenAmount(0)
            })
        )
    ]));
}

#[test]
fn test_position_below_current_tick() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, 100);

    let initial_balance = 10_000_000_000;
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_balance, initial_balance));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0: token_x,
            token_1: token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    assert!(!token_x_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: initial_balance
            }
        )
        .main_failed());

    assert!(!token_y_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: initial_balance
            }
        )
        .main_failed());

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool_state_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let lower_tick_index = -46080;
    let upper_tick_index = -23040;
    let liquidity_delta = Liquidity::from_integer(10_000);

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            liquidity_delta,
            slippage_limit_lower: pool_state_before.sqrt_price,
            slippage_limit_upper: pool_state_before.sqrt_price,
        },
    );
    let expected_x_increase = 0;
    let expected_y_increase = 2162;

    assert!(res.events_eq(vec![
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedEvent {
                address: REGULAR_USER_1.into(),
                pool_key,
                liquidity_delta,
                block_timestamp: sys.block_timestamp(),
                lower_tick: lower_tick_index,
                upper_tick: upper_tick_index,
                current_sqrt_price: init_sqrt_price,
            }
        ),
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedReturn(Position {
                pool_key,
                liquidity: liquidity_delta,
                lower_tick_index,
                upper_tick_index,
                fee_growth_inside_x: FeeGrowth::new(0),
                fee_growth_inside_y: FeeGrowth::new(0),
                last_block_number: sys.block_height() as u64,
                tokens_owed_x: TokenAmount(0),
                tokens_owed_y: TokenAmount(0)
            })
        )
    ]));

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let position_state = get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap();

    let lower_tick = get_tick(&invariant, pool_key, lower_tick_index).unwrap();
    let upper_tick = get_tick(&invariant, pool_key, upper_tick_index).unwrap();
    let user_1_x = balance_of(&token_x_program, REGULAR_USER_1.into());
    let user_1_y = balance_of(&token_y_program, REGULAR_USER_1.into());
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID.into());
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID.into());

    let zero_fee = FeeGrowth::new(0);

    // Check ticks
    assert_eq!(lower_tick.index, lower_tick_index);
    assert_eq!(upper_tick.index, upper_tick_index);
    assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
    assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
    assert_eq!(lower_tick.liquidity_change, liquidity_delta);
    assert_eq!(upper_tick.liquidity_change, liquidity_delta);
    assert!(lower_tick.sign);
    assert!(!upper_tick.sign);

    // Check position
    assert_eq!(position_state.pool_key, pool_key);
    assert_eq!(position_state.liquidity, liquidity_delta);
    assert_eq!(position_state.lower_tick_index, lower_tick_index);
    assert_eq!(position_state.upper_tick_index, upper_tick_index);
    assert_eq!(position_state.fee_growth_inside_x, zero_fee);
    assert_eq!(position_state.fee_growth_inside_y, zero_fee);

    // Check pool
    assert_eq!(pool_state.liquidity, pool_state_before.liquidity);
    assert_eq!(pool_state.current_tick_index, init_tick);

    // Check balances
    assert_eq!(user_1_x, initial_balance.checked_sub(invariant_x).unwrap());
    assert_eq!(user_1_y, initial_balance.checked_sub(invariant_y).unwrap());

    assert_eq!(
        (invariant_x, invariant_y),
        (expected_x_increase, expected_y_increase)
    );
}

#[test]
fn test_position_within_current_tick() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, 100);

    let initial_balance = 100_000_000;
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_balance, initial_balance));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();
    let max_tick_test = 177_450;
    let min_tick_test = -max_tick_test;
    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0: token_x,
            token_1: token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    assert!(!token_x_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: initial_balance
            }
        )
        .main_failed());

    assert!(!token_y_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: initial_balance
            }
        )
        .main_failed());

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool_state_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let lower_tick_index = min_tick_test + 10;
    let upper_tick_index = max_tick_test - 10;
    let liquidity_delta = Liquidity::from_integer(100);

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            liquidity_delta,
            slippage_limit_lower: pool_state_before.sqrt_price,
            slippage_limit_upper: pool_state_before.sqrt_price,
        },
    );
    assert!(!res.main_failed());

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let position_state = get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap();

    let lower_tick = get_tick(&invariant, pool_key, lower_tick_index).unwrap();
    let upper_tick = get_tick(&invariant, pool_key, upper_tick_index).unwrap();
    let user_1_x = balance_of(&token_x_program, REGULAR_USER_1.into());
    let user_1_y = balance_of(&token_y_program, REGULAR_USER_1.into());
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID.into());
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID.into());
    let zero_fee = FeeGrowth::new(0);
    let expected_x_increase = 317;
    let expected_y_increase = 32;

    // Check ticks
    assert_eq!(lower_tick.index, lower_tick_index);
    assert_eq!(upper_tick.index, upper_tick_index);
    assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
    assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
    assert_eq!(lower_tick.liquidity_change, liquidity_delta);
    assert_eq!(upper_tick.liquidity_change, liquidity_delta);
    assert!(lower_tick.sign);
    assert!(!upper_tick.sign);

    // Check pool
    assert_eq!(pool_state.liquidity, liquidity_delta);
    assert_eq!(pool_state.current_tick_index, init_tick);

    // Check position
    assert_eq!(position_state.pool_key, pool_key);
    assert_eq!(position_state.liquidity, liquidity_delta);
    assert_eq!(position_state.lower_tick_index, lower_tick_index);
    assert_eq!(position_state.upper_tick_index, upper_tick_index);
    assert_eq!(position_state.fee_growth_inside_x, zero_fee);
    assert_eq!(position_state.fee_growth_inside_y, zero_fee);

    // Check balances
    assert_eq!(user_1_x, initial_balance.checked_sub(invariant_x).unwrap());
    assert_eq!(user_1_y, initial_balance.checked_sub(invariant_y).unwrap());

    assert_eq!(
        (invariant_x, invariant_y),
        (expected_x_increase, expected_y_increase)
    );
}

#[test]
fn test_position_above_current_tick() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, 100);

    let initial_balance = 100_000_000;
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_balance, initial_balance));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();
    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0: token_x,
            token_1: token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    assert!(!token_x_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: initial_balance
            }
        )
        .main_failed());

    assert!(!token_y_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: initial_balance
            }
        )
        .main_failed());

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool_state_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let lower_tick_index = -22980;
    let upper_tick_index = 0;
    let liquidity_delta = Liquidity::from_integer(10_000);

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            liquidity_delta,
            slippage_limit_lower: pool_state_before.sqrt_price,
            slippage_limit_upper: pool_state_before.sqrt_price,
        },
    );

    assert!(res.events_eq(vec![
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedEvent {
                address: REGULAR_USER_1.into(),
                pool_key,
                liquidity_delta,
                block_timestamp: sys.block_timestamp(),
                lower_tick: lower_tick_index,
                upper_tick: upper_tick_index,
                current_sqrt_price: init_sqrt_price,
            }
        ),
        TestEvent::invariant_response(
            REGULAR_USER_1,
            InvariantEvent::PositionCreatedReturn(Position {
                pool_key,
                liquidity: liquidity_delta,
                lower_tick_index,
                upper_tick_index,
                fee_growth_inside_x: FeeGrowth::new(0),
                fee_growth_inside_y: FeeGrowth::new(0),
                last_block_number: sys.block_height() as u64,
                tokens_owed_x: TokenAmount(0),
                tokens_owed_y: TokenAmount(0)
            })
        )
    ]));

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let position_state = get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap();

    let lower_tick = get_tick(&invariant, pool_key, lower_tick_index).unwrap();
    let upper_tick = get_tick(&invariant, pool_key, upper_tick_index).unwrap();
    let user_1_x = balance_of(&token_x_program, REGULAR_USER_1.into());
    let user_1_y = balance_of(&token_y_program, REGULAR_USER_1.into());
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID.into());
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID.into());

    let zero_fee = FeeGrowth::new(0);
    let expected_x_increase = 21549;
    let expected_y_increase = 0;

    // Check ticks
    assert_eq!(lower_tick.index, lower_tick_index);
    assert_eq!(upper_tick.index, upper_tick_index);
    assert_eq!(lower_tick.liquidity_gross, liquidity_delta);
    assert_eq!(upper_tick.liquidity_gross, liquidity_delta);
    assert_eq!(lower_tick.liquidity_change, liquidity_delta);
    assert_eq!(upper_tick.liquidity_change, liquidity_delta);
    assert!(lower_tick.sign);
    assert!(!upper_tick.sign);

    // Check position
    assert_eq!(position_state.pool_key, pool_key);
    assert_eq!(position_state.liquidity, liquidity_delta);
    assert_eq!(position_state.lower_tick_index, lower_tick_index);
    assert_eq!(position_state.upper_tick_index, upper_tick_index);
    assert_eq!(position_state.fee_growth_inside_x, zero_fee);
    assert_eq!(position_state.fee_growth_inside_y, zero_fee);

    // Check pool
    assert_eq!(pool_state.liquidity, Liquidity::new(0));
    assert_eq!(pool_state.current_tick_index, init_tick);

    // Check balances
    assert_eq!(user_1_x, initial_balance.checked_sub(invariant_x).unwrap());
    assert_eq!(user_1_y, initial_balance.checked_sub(invariant_y).unwrap());

    assert_eq!(invariant_x, expected_x_increase);
    assert_eq!(invariant_y, expected_y_increase);
}

#[test]
fn test_create_position_not_enough_token_x() {
    let sys = System::new();
    sys.init_logger();

    let mut invariant = init_invariant(&sys, 100);

    let initial_balance = 100_000_000;
    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (1, initial_balance));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0: token_x,
            token_1: token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    assert!(!token_x_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: initial_balance
            }
        )
        .main_failed());

    assert!(!token_y_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: initial_balance
            }
        )
        .main_failed());

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool_state_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let lower_tick_index = -8;
    let upper_tick_index = 8;
    let liquidity_delta = Liquidity::from_integer(10_000);

    let _res = invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            liquidity_delta,
            slippage_limit_lower: pool_state_before.sqrt_price,
            slippage_limit_upper: pool_state_before.sqrt_price,
        },
        InvariantError::TransferError,
    );

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    get_tick(&invariant, pool_key, lower_tick_index).unwrap_err();
    get_tick(&invariant, pool_key, upper_tick_index).unwrap_err();
    get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap_err();

    let user_1_x = balance_of(&token_x_program, REGULAR_USER_1.into());
    let user_1_y = balance_of(&token_y_program, REGULAR_USER_1.into());
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID.into());
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID.into());

    assert_eq!(invariant_x, 0);
    assert_eq!(invariant_y, 0);
    assert_eq!(user_1_x, 1);
    assert_eq!(user_1_y, initial_balance);
    assert_eq!(&pool_state, &pool_state_before);
}

#[test]
fn test_create_position_not_enough_token_y() {
    let sys = System::new();
    sys.init_logger();

    let mut invariant = init_invariant(&sys, 100);

    let initial_balance = 100_000_000;
    let (token_x_program, token_y_program) = init_tokens_with_mint(&sys, (initial_balance, 1));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0: token_x,
            token_1: token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    assert!(!token_x_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: initial_balance
            }
        )
        .main_failed());

    assert!(!token_y_program
        .send(
            REGULAR_USER_1,
            FTAction::Approve {
                tx_id: None,
                to: INVARIANT_ID.into(),
                amount: initial_balance
            }
        )
        .main_failed());

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool_state_before = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    let lower_tick_index = -8;
    let upper_tick_index = 8;
    let liquidity_delta = Liquidity::from_integer(10_000);

    let _res = invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::CreatePosition {
            pool_key,
            lower_tick: lower_tick_index,
            upper_tick: upper_tick_index,
            liquidity_delta,
            slippage_limit_lower: pool_state_before.sqrt_price,
            slippage_limit_upper: pool_state_before.sqrt_price,
        },
        InvariantError::TransferError,
    );

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    get_tick(&invariant, pool_key, lower_tick_index).unwrap_err();
    get_tick(&invariant, pool_key, upper_tick_index).unwrap_err();
    get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap_err();

    let user_1_x = balance_of(&token_x_program, REGULAR_USER_1.into());
    let user_1_y = balance_of(&token_y_program, REGULAR_USER_1.into());
    let invariant_x = balance_of(&token_x_program, INVARIANT_ID.into());
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID.into());

    assert_eq!(invariant_x, 0);
    assert_eq!(invariant_y, 0);
    assert_eq!(user_1_x, initial_balance);
    assert_eq!(user_1_y, 1);
    assert_eq!(&pool_state, &pool_state_before);
}
