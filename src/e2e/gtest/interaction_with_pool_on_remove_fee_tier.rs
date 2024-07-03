use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gtest::*;
use math::{percentage::Percentage, sqrt_price::calculate_sqrt_price, token_amount::TokenAmount};
use sails_rtl::ActorId;

#[test]
fn test_interaction_with_pool_on_removed_fee_tier() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let (token_x_program, token_y_program) = init_tokens(&sys);

    init_basic_pool(&invariant, &token_x, &token_y);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    // Remove Fee Tier
    remove_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    assert!(!fee_tier_exists(&invariant, fee_tier));

    // Attempt to create same pool again
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_panicked_with(InvariantError::FeeTierNotFound);

    // Init  position
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);

    // Init swap
    init_basic_swap(&sys, &invariant, &token_x_program, &token_y_program);

    // Claim fee
    let (claimed_x, claimed_y) =
        claim_fee(&invariant, REGULAR_USER_1, 0, None::<InvariantError>).unwrap();

    assert_eq!(claimed_x, TokenAmount::new(U256::from(5)));
    assert_eq!(claimed_y, TokenAmount::new(U256::from(0)));

    // Change fee receiver
    change_fee_receiver(&invariant, ADMIN, pool_key, REGULAR_USER_1.into()).assert_success();

    // Withdraw protocol fee
    withdraw_protocol_fee(&invariant, REGULAR_USER_1, pool_key).assert_success();

    // Close position
    remove_position(&invariant, REGULAR_USER_1, 0).assert_success();

    // Get pool
    get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    // Get Pools
    let pools = get_pools(&invariant, u8::MAX, 0).unwrap();
    assert_eq!(pools.len(), 1);

    // Transfer position
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);
    let transferred_index = 0;
    let owner_list_before = get_all_positions(&invariant, REGULAR_USER_1.into());
    let recipient_list_before = get_all_positions(&invariant, REGULAR_USER_2.into());
    let removed_position =
        get_position(&invariant, REGULAR_USER_1.into(), transferred_index).unwrap();

    transfer_position(
        &invariant,
        REGULAR_USER_1,
        transferred_index,
        REGULAR_USER_2,
    )
    .assert_success();

    let recipient_position =
        get_position(&invariant, REGULAR_USER_2.into(), transferred_index).unwrap();
    let owner_list_after = get_all_positions(&invariant, REGULAR_USER_1.into());
    let recipient_list_after = get_all_positions(&invariant, REGULAR_USER_2.into());

    assert_eq!(owner_list_before.len(), 1);
    assert_eq!(owner_list_after.len(), 0);
    assert_eq!(recipient_list_before.len(), 0);
    assert_eq!(recipient_list_after.len(), 1);

    assert_eq!(recipient_position, removed_position);
    // Compare lists of positions
    assert_eq!(recipient_list_after, owner_list_before);

    // Read fee tier and create same pool
    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    create_pool(
        &invariant,
        REGULAR_USER_1,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_panicked_with(InvariantError::PoolAlreadyExist);
}
