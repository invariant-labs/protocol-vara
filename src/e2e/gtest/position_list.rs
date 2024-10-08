use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{
    fee_growth::FeeGrowth, liquidity::Liquidity, percentage::Percentage,
    sqrt_price::calculate_sqrt_price,
};
use sails_rs::ActorId;

#[test]
fn test_remove_position_from_empty_list() {
    let sys = System::new();
    let invariant = init_invariant(&sys, Percentage::from_scale(6, 3));

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 3).unwrap();
    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let _res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_1);

    remove_position(&invariant, REGULAR_USER_1, 0)
        .assert_panicked_with(InvariantError::PositionNotFound);
}

#[test]
fn test_add_multiple_positions() {
    let sys = System::new();
    let invariant = init_invariant(&sys, Percentage::from_scale(6, 3));

    let initial_amount = U256::from( 10u128.pow(10));
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount, initial_amount));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 3).unwrap();
    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let _res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_1);

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_amount,
        token_y,
        initial_amount,
        None::<&str>,
    )
    .unwrap();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    // Open positions
    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[2],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[1],
        tick_indexes[4],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    //Remove middle position

    let position_index_to_remove = 2;
    let mut position_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());
    let last_position = position_list_before.last().unwrap();

    remove_position(&invariant, REGULAR_USER_1, position_index_to_remove).assert_success();

    let position_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    let tested_position = position_list_after[position_index_to_remove as usize];
    assert_eq!(&tested_position, last_position);
    position_list_before.remove(position_index_to_remove as usize);
    assert_eq!(position_list_before, position_list_after);

    // Add position in place of the removed one
    let position_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[1],
        tick_indexes[2],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    let mut position_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    assert_eq!(position_list_before.len() + 1, position_list_after.len());
    position_list_after.pop();
    assert_eq!(position_list_before, position_list_after);

    // Remove last position
    let mut position_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());
    let position_to_remove = position_list_before.len() - 1;

    remove_position(&invariant, REGULAR_USER_1, position_to_remove as u32).assert_success();

    let position_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    assert_eq!(position_list_before.len() - 1, position_list_after.len());
    position_list_before.pop();
    assert_eq!(position_list_before, position_list_after);

    // Remove all positions
    let list_len_before = get_all_positions(&invariant, REGULAR_USER_1.into()).len();

    for i in (0..list_len_before).rev() {
        remove_position(&invariant, REGULAR_USER_1, i as u32).assert_success();
    }

    let list_len_after = get_all_positions(&invariant, REGULAR_USER_1.into()).len();
    assert_eq!(list_len_after, 0);

    // Add position to cleared list
    let position_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());
    assert_eq!(position_list_before.len(), 0);
    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();
    let position_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    assert_eq!(position_list_after.len(), 1);
}
#[test]
fn test_only_owner_can_modify_position_list() {
    let sys = System::new();
    let invariant = init_invariant(&sys, Percentage::from_scale(6, 3));

    let initial_amount = U256::from( 10u128.pow(10));
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount, initial_amount));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 3).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let _res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_1);

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_amount,
        token_y,
        initial_amount,
        None::<&str>,
    )
    .unwrap();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    // Open positions
    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[2],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[1],
        tick_indexes[4],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    //Remove middle position

    let position_index_to_remove = 2;
    let mut position_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());
    let last_position = position_list_before.last().unwrap();
    remove_position(&invariant, REGULAR_USER_1, position_index_to_remove).assert_success();

    let position_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    let tested_position = position_list_after[position_index_to_remove as usize];
    assert_eq!(&tested_position, last_position);
    position_list_before.remove(position_index_to_remove as usize);
    assert_eq!(position_list_before, position_list_after);

    // Add position in place of the removed one
    let position_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[1],
        tick_indexes[2],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();
    let mut position_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    assert_eq!(position_list_before.len() + 1, position_list_after.len());
    position_list_after.pop();
    assert_eq!(position_list_before, position_list_after);

    // Remove last position
    let position_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());
    let position_to_remove = position_list_before.len() - 1;

    remove_position(&invariant, REGULAR_USER_2, position_to_remove as u32)
        .assert_panicked_with(InvariantError::PositionNotFound);
}

#[test]
fn test_transfer_position_ownership() {
    let sys = System::new();
    let invariant = init_invariant(&sys, Percentage::from_scale(6, 3));

    let initial_amount = U256::from( 10u128.pow(10));
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount, initial_amount));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 3).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let _res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_1);

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_amount,
        token_y,
        initial_amount,
        None::<&str>,
    )
    .unwrap();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    // Open positions
    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[2],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[1],
        tick_indexes[4],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    // Transfer first position
    let transferred_index = 0;
    let owner_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());
    let mut recipient_list_before = get_all_positions(&invariant, REGULAR_USER_2.into());
    let transferred_position =
        get_position(&invariant, REGULAR_USER_1.into(), transferred_index).unwrap();
    let last_position_before = owner_list_before[owner_list_before.len() - 1];

    transfer_position(
        &invariant,
        REGULAR_USER_1,
        transferred_index,
        REGULAR_USER_2,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_1);

    let recipient_list_after = get_all_positions(&invariant, REGULAR_USER_2.into());
    let mut owner_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    let recipient_position = get_position(&invariant, REGULAR_USER_2.into(), 0).unwrap();
    let owner_first_position_after = get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap();

    assert_eq!(recipient_list_after.len(), 1);
    assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);

    assert_eq!(owner_list_after.len(), owner_list_before.len() - 1);
    assert_eq!(owner_list_after.len(), 3);

    assert_eq!(recipient_position, transferred_position);
    assert_eq!(owner_first_position_after, last_position_before);

    owner_list_after.push(transferred_position);
    let owner_list_after_len = owner_list_after.len();
    owner_list_after.swap(transferred_index as usize, owner_list_after_len - 1);
    recipient_list_before.push(transferred_position);
    assert_eq!(owner_list_after, owner_list_before);
    assert_eq!(recipient_list_before, recipient_list_after);

    // Transfer middle position
    let transferred_index = 1;
    let owner_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());
    let mut recipient_list_before = get_all_positions(&invariant, REGULAR_USER_2.into());
    let transferred_position =
        get_position(&invariant, REGULAR_USER_1.into(), transferred_index).unwrap();
    let last_position_before = owner_list_before[owner_list_before.len() - 1];

    transfer_position(
        &invariant,
        REGULAR_USER_1,
        transferred_index,
        REGULAR_USER_2,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_1);

    let recipient_list_after = get_all_positions(&invariant, REGULAR_USER_2.into());
    let mut owner_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    let recipient_position = get_position(
        &invariant,
        REGULAR_USER_2.into(),
        recipient_list_after.len() as u32 - 1,
    )
    .unwrap();
    let owner_middle_position_after =
        get_position(&invariant, REGULAR_USER_1.into(), transferred_index).unwrap();

    assert_eq!(recipient_list_after.len(), 2);
    assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);

    assert_eq!(owner_list_after.len(), owner_list_before.len() - 1);
    assert_eq!(owner_list_after.len(), 2);

    assert_eq!(recipient_position, transferred_position);
    assert_eq!(owner_middle_position_after, last_position_before);

    owner_list_after.push(transferred_position);
    let owner_list_after_len = owner_list_after.len();
    owner_list_after.swap(transferred_index as usize, owner_list_after_len - 1);
    recipient_list_before.push(transferred_position);
    assert_eq!(owner_list_after, owner_list_before);
    assert_eq!(recipient_list_before, recipient_list_after);

    // Transfer last position
    let owner_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());
    let transferred_index = owner_list_before.len() - 1;
    let mut recipient_list_before = get_all_positions(&invariant, REGULAR_USER_2.into());
    let transferred_position =
        get_position(&invariant, REGULAR_USER_1.into(), transferred_index as u32).unwrap();
    let first_position_before = owner_list_before[0];

    transfer_position(
        &invariant,
        REGULAR_USER_1,
        transferred_index as u32,
        REGULAR_USER_2,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_1);

    let recipient_list_after = get_all_positions(&invariant, REGULAR_USER_2.into());
    let mut owner_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    let recipient_position = get_position(
        &invariant,
        REGULAR_USER_2.into(),
        recipient_list_after.len() as u32 - 1,
    )
    .unwrap();
    let owner_first_position_after = get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap();

    assert_eq!(recipient_list_after.len(), 3);
    assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);

    assert_eq!(owner_list_after.len(), owner_list_before.len() - 1);
    assert_eq!(owner_list_after.len(), 1);

    assert_eq!(recipient_position, transferred_position);
    assert_eq!(owner_first_position_after, first_position_before);

    owner_list_after.push(transferred_position);
    recipient_list_before.push(transferred_position);
    assert_eq!(owner_list_after, owner_list_before);
    assert_eq!(recipient_list_before, recipient_list_after);

    // Clear position
    let owner_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());
    let transferred_index = 0;
    let mut recipient_list_before = get_all_positions(&invariant, REGULAR_USER_2.into());
    let transferred_position =
        get_position(&invariant, REGULAR_USER_1.into(), transferred_index as u32).unwrap();

    transfer_position(
        &invariant,
        REGULAR_USER_1,
        transferred_index,
        REGULAR_USER_2,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_1);

    let recipient_list_after = get_all_positions(&invariant, REGULAR_USER_2.into());
    let mut owner_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    let recipient_position = get_position(
        &invariant,
        REGULAR_USER_2.into(),
        recipient_list_after.len() as u32 - 1,
    )
    .unwrap();
    get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap_err();

    assert_eq!(recipient_list_after.len(), 4);
    assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);

    assert_eq!(owner_list_after.len(), owner_list_before.len() - 1);
    assert_eq!(owner_list_after.len(), 0);

    assert_eq!(recipient_position, transferred_position);

    owner_list_after.push(transferred_position);
    recipient_list_before.push(transferred_position);
    assert_eq!(owner_list_after, owner_list_before);
    assert_eq!(recipient_list_before, recipient_list_after);

    //Get back position
    let owner_list_before = get_all_positions(&invariant, REGULAR_USER_2.into());
    let transferred_index = 0;
    let mut recipient_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());
    let transferred_position =
        get_position(&invariant, REGULAR_USER_2.into(), transferred_index as u32).unwrap();
    let last_position_before = owner_list_before.last().unwrap();

    transfer_position(
        &invariant,
        REGULAR_USER_2,
        transferred_index,
        REGULAR_USER_1,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_2);

    let recipient_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    let mut owner_list_after = get_all_positions(&invariant, REGULAR_USER_2.into());
    let recipient_position = get_position(
        &invariant,
        REGULAR_USER_1.into(),
        recipient_list_after.len() as u32 - 1,
    )
    .unwrap();
    let owner_first_position_after = get_position(&invariant, REGULAR_USER_2.into(), 0).unwrap();

    assert_eq!(recipient_list_after.len(), 1);
    assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);

    assert_eq!(owner_list_after.len(), owner_list_before.len() - 1);
    assert_eq!(owner_list_after.len(), 3);

    assert_eq!(recipient_position, transferred_position);
    assert_eq!(owner_first_position_after, *last_position_before);

    owner_list_after.push(transferred_position);
    let owner_list_after_len = owner_list_after.len();
    owner_list_after.swap(transferred_index as usize, owner_list_after_len - 1);
    recipient_list_before.push(transferred_position);
    assert_eq!(owner_list_after, owner_list_before);
    assert_eq!(recipient_list_before, recipient_list_after);
}

#[test]
fn test_only_owner_can_transfer_position() {
    let sys = System::new();
    let invariant = init_invariant(&sys, Percentage::from_scale(6, 3));

    let initial_amount = U256::from( 10u128.pow(10));
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount, initial_amount));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 3).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let _res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_1);

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_amount,
        token_y,
        initial_amount,
        None::<&str>,
    )
    .unwrap();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    // Open positions
    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[0],
        tick_indexes[2],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        tick_indexes[1],
        tick_indexes[4],
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    // Transfer first position
    let transferred_index = 0;
    let owner_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());
    let recipient_list_before = get_all_positions(&invariant, REGULAR_USER_2.into());

    transfer_position(
        &invariant,
        REGULAR_USER_2,
        transferred_index,
        REGULAR_USER_1,
    )
    .assert_panicked_with(InvariantError::PositionNotFound);

    let recipient_list_after = get_all_positions(&invariant, REGULAR_USER_2.into());
    let owner_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    get_position(&invariant, REGULAR_USER_2.into(), 0).unwrap_err();

    assert_eq!(recipient_list_after.len(), 0);
    assert_eq!(owner_list_after.len(), 4);

    assert_eq!(recipient_list_after, recipient_list_before);
    assert_eq!(owner_list_after, owner_list_before);
}

#[test]
fn test_multiple_positions_on_same_tick() {
    let sys = System::new();
    let invariant = init_invariant(&sys, Percentage::from_scale(6, 3));

    let initial_amount = U256::from( 10u128.pow(8));
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (initial_amount, initial_amount));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 10).unwrap();

    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let _res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_1);

    increase_allowance(
        &token_x_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();
    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        initial_amount,
    )
    .assert_success();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        initial_amount,
        token_y,
        initial_amount,
        None::<&str>,
    )
    .unwrap();

    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    let lower_tick_index = -10;
    let upper_tick_index = 10;

    let liquidity_delta = Liquidity::new(U256::from(100));

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    // Open positions
    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    let first_position = get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    let second_position = get_position(&invariant, REGULAR_USER_1.into(), 1).unwrap();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    let third_position = get_position(&invariant, REGULAR_USER_1.into(), 2).unwrap();

    assert_eq!(
        first_position.lower_tick_index,
        second_position.lower_tick_index
    );
    assert_eq!(
        first_position.upper_tick_index,
        second_position.upper_tick_index
    );
    assert_eq!(
        first_position.lower_tick_index,
        third_position.lower_tick_index
    );
    assert_eq!(
        first_position.upper_tick_index,
        third_position.upper_tick_index
    );

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    let lower_tick = get_tick(&invariant, pool_key, lower_tick_index).unwrap();
    let upper_tick = get_tick(&invariant, pool_key, upper_tick_index).unwrap();

    let expected_liquidity = Liquidity::new(liquidity_delta.get() * 3);
    let zero_fee = FeeGrowth::new(0);

    // Check ticks
    assert_eq!(lower_tick.index, lower_tick_index);
    assert_eq!(upper_tick.index, upper_tick_index);
    assert_eq!(lower_tick.liquidity_gross, expected_liquidity);
    assert_eq!(upper_tick.liquidity_gross, expected_liquidity);
    assert_eq!(lower_tick.liquidity_change, expected_liquidity);
    assert_eq!(upper_tick.liquidity_change, expected_liquidity);
    assert!(lower_tick.sign);
    assert!(!upper_tick.sign);

    // Check pool
    assert_eq!(pool_state.liquidity, expected_liquidity);
    assert_eq!(pool_state.current_tick_index, init_tick);

    // Check first position
    assert_eq!(first_position.pool_key, pool_key);
    assert_eq!(first_position.liquidity, liquidity_delta);
    assert_eq!(first_position.lower_tick_index, lower_tick_index);
    assert_eq!(first_position.upper_tick_index, upper_tick_index);
    assert_eq!(first_position.fee_growth_inside_x, zero_fee);
    assert_eq!(first_position.fee_growth_inside_y, zero_fee);

    // Check second position
    assert_eq!(second_position.pool_key, pool_key);
    assert_eq!(second_position.liquidity, liquidity_delta);
    assert_eq!(second_position.lower_tick_index, lower_tick_index);
    assert_eq!(second_position.upper_tick_index, upper_tick_index);
    assert_eq!(second_position.fee_growth_inside_x, zero_fee);
    assert_eq!(second_position.fee_growth_inside_y, zero_fee);

    // Check third position
    assert_eq!(third_position.pool_key, pool_key);
    assert_eq!(third_position.liquidity, liquidity_delta);
    assert_eq!(third_position.lower_tick_index, lower_tick_index);
    assert_eq!(third_position.upper_tick_index, upper_tick_index);
    assert_eq!(third_position.fee_growth_inside_x, zero_fee);
    assert_eq!(third_position.fee_growth_inside_y, zero_fee);

    let lower_tick_index = -10;
    let upper_tick_index = 10;
    let zero_fee = FeeGrowth::new(0);
    let liquidity_delta = Liquidity::new(U256::from(100));

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    // Open positions
    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    let first_position = get_position(&invariant, REGULAR_USER_1.into(), 0).unwrap();

    // Check first position
    assert_eq!(first_position.pool_key, pool_key);
    assert_eq!(first_position.liquidity, liquidity_delta);
    assert_eq!(first_position.lower_tick_index, lower_tick_index);
    assert_eq!(first_position.upper_tick_index, upper_tick_index);
    assert_eq!(first_position.fee_growth_inside_x, zero_fee);
    assert_eq!(first_position.fee_growth_inside_y, zero_fee);

    let lower_tick_index = -20;
    let upper_tick_index = -10;

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    let second_position = get_position(&invariant, REGULAR_USER_1.into(), 4).unwrap();

    // Check second position
    assert_eq!(second_position.pool_key, pool_key);
    assert_eq!(second_position.liquidity, liquidity_delta);
    assert_eq!(second_position.lower_tick_index, lower_tick_index);
    assert_eq!(second_position.upper_tick_index, upper_tick_index);
    assert_eq!(second_position.fee_growth_inside_x, zero_fee);
    assert_eq!(second_position.fee_growth_inside_y, zero_fee);

    let lower_tick_index = 10;
    let upper_tick_index = 20;

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool_state.sqrt_price,
        pool_state.sqrt_price,
    )
    .assert_success();

    let third_position = get_position(&invariant, REGULAR_USER_1.into(), 5).unwrap();

    // Check third position
    assert_eq!(third_position.pool_key, pool_key);
    assert_eq!(third_position.liquidity, liquidity_delta);
    assert_eq!(third_position.lower_tick_index, lower_tick_index);
    assert_eq!(third_position.upper_tick_index, upper_tick_index);
    assert_eq!(third_position.fee_growth_inside_x, zero_fee);
    assert_eq!(third_position.fee_growth_inside_y, zero_fee);

    let pool_state = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();
    let tick_n20 = get_tick(&invariant, pool_key, -20).unwrap();
    let tick_n10 = get_tick(&invariant, pool_key, -10).unwrap();
    let tick_10 = get_tick(&invariant, pool_key, 10).unwrap();
    let tick_20 = get_tick(&invariant, pool_key, 20).unwrap();
    let tick_n20_bit = is_tick_initialized(&invariant, pool_key, -20);
    let tick_n10_bit = is_tick_initialized(&invariant, pool_key, -10);
    let tick_10_bit = is_tick_initialized(&invariant, pool_key, 10);
    let tick_20_bit = is_tick_initialized(&invariant, pool_key, 20);

    let expected_active_liquidity = Liquidity::new(U256::from(400));

    // Check tick -20
    assert_eq!(tick_n20.index, -20);
    assert_eq!(tick_n20.liquidity_gross, Liquidity::new(U256::from(100)));
    assert_eq!(tick_n20.liquidity_change, Liquidity::new(U256::from(100)));
    assert!(tick_n20.sign);
    assert!(tick_n20_bit);

    // Check tick -10
    assert_eq!(tick_n10.index, -10);
    assert_eq!(tick_n10.liquidity_gross, Liquidity::new(U256::from(500)));
    assert_eq!(tick_n10.liquidity_change, Liquidity::new(U256::from(300)));
    assert!(tick_n10.sign);
    assert!(tick_n10_bit);

    // Check tick 10
    assert_eq!(tick_10.index, 10);
    assert_eq!(tick_10.liquidity_gross, Liquidity::new(U256::from(500)));
    assert_eq!(tick_10.liquidity_change, Liquidity::new(U256::from(300)));
    assert!(!tick_10.sign);
    assert!(tick_10_bit);

    // Check tick 20
    assert_eq!(tick_20.index, 20);
    assert_eq!(tick_20.liquidity_gross, Liquidity::new(U256::from(100)));
    assert_eq!(tick_20.liquidity_change, Liquidity::new(U256::from(100)));
    assert!(!tick_20.sign);
    assert!(tick_20_bit);

    // Check pool
    assert_eq!(pool_state.liquidity, expected_active_liquidity);
    assert_eq!(pool_state.current_tick_index, init_tick);
}
