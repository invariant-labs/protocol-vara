use contracts::*;

use crate::test_helpers::gtest::PositionCreatedEvent;

use super::gtest::SwapEvent;

#[track_caller]
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

#[track_caller]
pub fn swap_events_are_identical_no_timestamp(
    swap_event: &SwapEvent,
    other_swap_event: &SwapEvent,
) {
    let SwapEvent {
        timestamp: _timestamp,
        address,
        pool_key,
        amount_in,
        amount_out,
        fee,
        start_sqrt_price,
        target_sqrt_price,
        x_to_y,
    } = swap_event;
    assert_eq!(*address, other_swap_event.address);
    assert_eq!(*pool_key, other_swap_event.pool_key);
    assert_eq!(*amount_in, other_swap_event.amount_in);
    assert_eq!(*amount_out, other_swap_event.amount_out);
    assert_eq!(*fee, other_swap_event.fee);
    assert_eq!(*start_sqrt_price, other_swap_event.start_sqrt_price);
    assert_eq!(*target_sqrt_price, other_swap_event.target_sqrt_price);
    assert_eq!(*x_to_y, other_swap_event.x_to_y);
}

#[track_caller]
pub fn positions_are_identical_no_timestamp(position: &Position, other_position: &Position) {
    let Position {
        last_block_number: _last_block_number,
        pool_key,
        fee_growth_inside_x,
        fee_growth_inside_y,
        liquidity,
        lower_tick_index,
        upper_tick_index,
        tokens_owed_x,
        tokens_owed_y,
    } = position;

    assert_eq!(*pool_key, other_position.pool_key);
    assert_eq!(*fee_growth_inside_x, other_position.fee_growth_inside_x);
    assert_eq!(*fee_growth_inside_y, other_position.fee_growth_inside_y);
    assert_eq!(*liquidity, other_position.liquidity);
    assert_eq!(*lower_tick_index, other_position.lower_tick_index);
    assert_eq!(*upper_tick_index, other_position.upper_tick_index);
    assert_eq!(*tokens_owed_x, other_position.tokens_owed_x);
    assert_eq!(*tokens_owed_y, other_position.tokens_owed_y);
}

#[track_caller]
pub fn position_created_events_are_identical_no_timestamp(
    position_created_event: &PositionCreatedEvent,
    other_position_created_event: &PositionCreatedEvent,
) {
    let PositionCreatedEvent {
        timestamp: _timestamp,
        address,
        pool_key,
        liquidity_delta,
        lower_tick,
        upper_tick,
        current_sqrt_price,
    } = position_created_event;

    assert_eq!(*pool_key, other_position_created_event.pool_key);
    assert_eq!(*address, other_position_created_event.address);
    assert_eq!(
        *liquidity_delta,
        other_position_created_event.liquidity_delta
    );
    assert_eq!(*lower_tick, other_position_created_event.lower_tick);
    assert_eq!(*upper_tick, other_position_created_event.upper_tick);
    assert_eq!(
        *current_sqrt_price,
        other_position_created_event.current_sqrt_price
    );
}
