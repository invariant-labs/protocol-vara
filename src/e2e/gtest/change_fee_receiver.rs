use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gstd::*;
use gtest::*;
use io::*;
use math::{percentage::Percentage, sqrt_price::calculate_sqrt_price};

#[test]
fn test_change_fee_receiver() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(0));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);
    let fee_tier = FeeTier {
        fee: Percentage::new(1),
        tick_spacing: 1,
    };
    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let init_sqrt_price = calculate_sqrt_price(0).unwrap();

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_x: token_0,
            token_y: token_1,
            fee_tier,
            init_sqrt_price,
            init_tick: 0,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    let pool = get_pool(&invariant, token_0, token_1, fee_tier).unwrap();
    assert_eq!(pool.fee_receiver, ADMIN.into());

    let pool_key = PoolKey::new(token_0, token_1, fee_tier).unwrap();
    let res = invariant.send(
        ADMIN,
        InvariantAction::ChangeFeeReceiver(pool_key, REGULAR_USER_1.into()),
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let pool = get_pool(&invariant, token_0, token_1, fee_tier).unwrap();
    assert_eq!(pool.fee_receiver, REGULAR_USER_1.into());
}

#[test]
fn test_change_fee_receiver_not_admin() {
    let sys = System::new();
    sys.init_logger();

    let mut invariant = init_invariant(&sys, Percentage(0));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);
    let fee_tier = FeeTier {
        fee: Percentage::new(1),
        tick_spacing: 1,
    };
    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let init_sqrt_price = calculate_sqrt_price(0).unwrap();

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_x: token_0,
            token_y: token_1,
            fee_tier,
            init_sqrt_price,
            init_tick: 0,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    let pool = get_pool(&invariant, token_0, token_1, fee_tier).unwrap();
    assert_eq!(pool.fee_receiver, ADMIN.into());

    let pool_key = PoolKey::new(token_0, token_1, fee_tier).unwrap();
    let _res = invariant.send_and_assert_panic(
        REGULAR_USER_1,
        InvariantAction::ChangeFeeReceiver(pool_key, REGULAR_USER_1.into()),
        InvariantError::NotAdmin,
    );

    let pool = get_pool(&invariant, token_0, token_1, fee_tier).unwrap();
    assert_eq!(pool.fee_receiver, ADMIN.into());
}
