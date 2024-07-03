use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gtest::*;
use math::{percentage::Percentage, sqrt_price::calculate_sqrt_price};
use sails_rtl::ActorId;

#[test]
fn test_change_fee_receiver() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(0)));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);
    let fee_tier = FeeTier {
        fee: Percentage::new(U128::from(1)),
        tick_spacing: 1,
    };
    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let init_sqrt_price = calculate_sqrt_price(0).unwrap();

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        0,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

    let pool = get_pool(&invariant, token_0, token_1, fee_tier).unwrap();
    assert_eq!(pool.fee_receiver, ADMIN.into());

    let pool_key = PoolKey::new(token_0, token_1, fee_tier).unwrap();
    change_fee_receiver(&invariant, ADMIN, pool_key, REGULAR_USER_1.into())
        .assert_single_event()
        .assert_empty()
        .assert_to(ADMIN);

    let pool = get_pool(&invariant, token_0, token_1, fee_tier).unwrap();
    assert_eq!(pool.fee_receiver, REGULAR_USER_1.into());
}

#[test]
fn test_change_fee_receiver_not_admin() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(U128::from(0)));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);
    let fee_tier = FeeTier {
        fee: Percentage::new(U128::from(1)),
        tick_spacing: 1,
    };
    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let init_sqrt_price = calculate_sqrt_price(0).unwrap();

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        0,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

    let pool = get_pool(&invariant, token_0, token_1, fee_tier).unwrap();
    assert_eq!(pool.fee_receiver, ADMIN.into());

    let pool_key = PoolKey::new(token_0, token_1, fee_tier).unwrap();
    change_fee_receiver(&invariant, REGULAR_USER_1, pool_key, REGULAR_USER_1.into())
        .assert_panicked_with(InvariantError::NotAdmin);

    let pool = get_pool(&invariant, token_0, token_1, fee_tier).unwrap();
    assert_eq!(pool.fee_receiver, ADMIN.into());
}
