use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gstd::{prelude::*, ActorId};
use gtest::*;
use io::*;
use math::{percentage::Percentage, sqrt_price::calculate_sqrt_price, token_amount::TokenAmount};

#[test]
fn test_interaction_with_pool_on_removed_fee_tier() {
    let sys = System::new();
    sys.init_logger();

    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let mut invariant = init_invariant(&sys, Percentage::from_scale(1, 2));
    let (token_x_program, token_y_program) = init_tokens(&sys);

    init_basic_pool(&invariant, &token_x, &token_y);

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 10).unwrap();
    let pool_key = PoolKey::new(token_x, token_y, fee_tier).unwrap();
    // Remove Fee Tier
    invariant
        .send(ADMIN, InvariantAction::RemoveFeeTier(fee_tier))
        .assert_success();
    assert!(!fee_tier_exists(&invariant, fee_tier));
    
    // Attempt to create same pool again
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_x,
            token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
        InvariantError::FeeTierNotFound,
    );
    
    // Init  position
    init_basic_position(&sys, &invariant, &token_x_program, &token_y_program);
    
    // Init swap
    init_basic_swap(&sys, &invariant, &token_x_program, &token_y_program);
    
    // Claim fee
    let (claimed_x, claimed_y) = claim_fee(&invariant, REGULAR_USER_1, 0, None::<InvariantError>).unwrap();
    
    assert_eq!(claimed_x, TokenAmount(5));
    assert_eq!(claimed_y, TokenAmount(0));
    
    // Change fee receiver
    invariant.send(
        ADMIN,
        InvariantAction::ChangeFeeReceiver(pool_key, REGULAR_USER_1.into()),
    ).assert_success();
    
    // Withdraw protocol fee
    invariant.send(REGULAR_USER_1, InvariantAction::WithdrawProtocolFee(pool_key)).assert_success();    
    
    // Close position
    invariant.send(REGULAR_USER_1, InvariantAction::RemovePosition { position_id: 0 }).assert_success();    

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

    invariant.send(REGULAR_USER_1, InvariantAction::TransferPosition {
        index: transferred_index,
        receiver: REGULAR_USER_2.into(),
    }).assert_success();

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
    invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier)).assert_success();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_x,
            token_y,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
        InvariantError::PoolAlreadyExist
    );
}
