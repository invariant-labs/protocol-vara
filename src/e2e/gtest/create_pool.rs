use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
};
use sails_rs::{prelude::*, ActorId};

#[test]
fn test_create_pool() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

    assert_eq!(
        get_pool_keys(&invariant, u16::MAX, 0),
        (vec![PoolKey::new(token_0, token_1, fee_tier).unwrap()], 1)
    );

    let pool = get_pool(&invariant, token_0, token_1, fee_tier).expect("Pool doesn't exist");
    let expected_pool = Pool {
        sqrt_price: init_sqrt_price,
        current_tick_index: init_tick,
        fee_receiver: ActorId::from(ADMIN),
        ..Pool::default()
    };

    pools_are_identical_no_timestamp(&pool, &expected_pool);
}

#[test]
fn test_create_pool_x_to_y_and_y_to_x() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_panicked_with(InvariantError::PoolAlreadyExist);

    assert_eq!(
        get_pool_keys(&invariant, u16::MAX, 0),
        (vec![PoolKey::new(token_0, token_1, fee_tier).unwrap()], 1)
    );
}
#[test]
fn test_create_pool_with_same_tokens() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let token_0 = ActorId::from([0x01; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_0,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_panicked_with(InvariantError::TokensAreSame);

    assert_eq!(get_pool_keys(&invariant, u16::MAX, 0), (vec![], 0));
}

#[test]
fn test_create_pool_fee_tier_not_added() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_panicked_with(InvariantError::FeeTierNotFound);

    assert_eq!(get_pool_keys(&invariant, u16::MAX, 0), (vec![],0));
}

#[test]
fn test_create_pool_init_tick_not_divisible_by_tick_spacing() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 3).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let init_tick = 2;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_panicked_with(InvariantError::InvalidInitTick);

    assert_eq!(get_pool_keys(&invariant, u16::MAX, 0), (vec![], 0));
}

#[test]
fn test_create_pool_init_sqrt_price_minimal_difference_from_tick() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 3).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap() + SqrtPrice::new(1);

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

    assert_eq!(
        get_pool(&invariant, token_0, token_1, fee_tier)
            .unwrap()
            .current_tick_index,
        init_tick
    );
}

#[test]
fn test_create_pool_init_sqrt_price_has_closer_init_tick() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let init_tick = 2;
    let init_sqrt_price = SqrtPrice::new(1000175003749000000000000u128);

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_panicked_with(InvariantError::InvalidInitSqrtPrice);

    let correct_tick_index = 3;
    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        correct_tick_index,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

    assert_eq!(
        get_pool(&invariant, token_0, token_1, fee_tier)
            .unwrap()
            .current_tick_index,
        correct_tick_index
    );
}

#[test]
fn test_create_pool_init_sqrt_price_has_closer_init_tick_spacing_over_one() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 3).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);

    let init_tick = 0;
    let init_sqrt_price = SqrtPrice::new(1000225003749000000000000u128);

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_panicked_with(InvariantError::InvalidInitSqrtPrice);

    let correct_tick_index = 3;
    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        correct_tick_index,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

    assert_eq!(
        get_pool(&invariant, token_0, token_1, fee_tier)
            .unwrap()
            .current_tick_index,
        correct_tick_index
    );
}
