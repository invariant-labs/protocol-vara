use gclient::GearApi;
use gstd::prelude::*;
use contracts::{Pool, Position};

pub type MessageId = [u8; 32];
pub type UserId = [u8; 32];
pub type ProgramId = [u8; 32];
pub type TokenId = [u8; 32];

pub async fn send_message(
    api: &GearApi,
    program: impl Into<[u8; 32]> + gstd::Copy,
    message: impl Encode,
) -> MessageId {
    let gas_info = api
        .calculate_handle_gas(
            None,
            program.into().into(),
            message.encode().clone(),
            0,
            true,
        )
        .await
        .expect("Failed to send message");
        
    let (message_id, _hash) = api
        .send_message(program.into().into(), message, gas_info.burned * 2, 0)
        .await
        .expect("Failed to send message");
    <[u8; 32]>::from(message_id)
}

pub fn get_api_user_id (api: &GearApi) -> UserId {
    let user_id = api.account_id();
    <[u8; 32]>::from(user_id.clone())
}

pub fn get_new_token(mut last_token_id: TokenId) -> TokenId {
    let new_start = u128::from_le_bytes(last_token_id[0..16].try_into().unwrap()).wrapping_add(1);
    for (i, val) in new_start.to_le_bytes().into_iter().enumerate() {
        last_token_id[i] = val;
    }
    last_token_id
}

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

pub fn positions_are_identical_no_timestamp(position: &Position, other_position: &Position) {
    let Position { 
        pool_key,
        liquidity,
        lower_tick_index,
        upper_tick_index,
        fee_growth_inside_x,
        fee_growth_inside_y,
        last_block_number: _,
        tokens_owed_x,
        tokens_owed_y 
    } = *position;
    assert_eq!(pool_key, other_position.pool_key);
    assert_eq!(liquidity, other_position.liquidity);
    assert_eq!(lower_tick_index, other_position.lower_tick_index);
    assert_eq!(upper_tick_index, other_position.upper_tick_index);
    assert_eq!(fee_growth_inside_x, other_position.fee_growth_inside_x);
    assert_eq!(fee_growth_inside_y, other_position.fee_growth_inside_y);
    assert_eq!(tokens_owed_x, other_position.tokens_owed_x);
    assert_eq!(tokens_owed_y, other_position.tokens_owed_y);
}