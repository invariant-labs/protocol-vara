use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{
    liquidity::Liquidity,
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, get_max_tick, get_min_tick, SqrtPrice},
};
use sails_rtl::{prelude::*, ActorId};

#[test]
fn test_get_tickmap() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (U256::from(500), U256::from(500)));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);
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
        U256::from(500),
    )
    .assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        U256::from(500),
    )
    .assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        U256::from(500),
        token_y,
        U256::from(500),
        None::<&str>,
    )
    .unwrap();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -58,
        5,
        Liquidity::new(U256::from(10)),
        pool.sqrt_price,
        pool.sqrt_price,
    ).assert_success();
    let tickmap = get_tickmap(&invariant, pool_key);

    assert_eq!(
        tickmap[0],
        (
            3465,
            0b1000000000000000000000000000000000000000000000000000000000000001
        )
    );
    assert_eq!(tickmap.len(), 1);
}

#[test]
fn test_get_tickmap_tick_spacing_over_one() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (U256::from(500), U256::from(500)));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 10).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);
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
        U256::from(500),
    )
    .assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        U256::from(500),
    )
    .assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        U256::from(500),
        token_y,
        U256::from(500),
        None::<&str>,
    )
    .unwrap();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        10,
        20,
        Liquidity::new(U256::from(10)),
        pool.sqrt_price,
        pool.sqrt_price,
    ).assert_success();

    create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        get_min_tick(pool_key.fee_tier.tick_spacing),
        get_max_tick(pool_key.fee_tier.tick_spacing),
        Liquidity::new(U256::from(10)),
        pool.sqrt_price,
        pool.sqrt_price,
    ).assert_success();

    let tickmap = get_tickmap(&invariant, pool_key);

    assert_eq!(tickmap[0], (0, 0b1));
    assert_eq!(
        tickmap[1],
        (346, 0b1100000000000000000000000000000000000000)
    );
    assert_eq!(
        tickmap[2],
        (get_max_chunk(fee_tier.tick_spacing), 0b10000000000)
    );
    assert_eq!(tickmap.len(), 3);
}

#[test]
fn test_get_tickmap_edge_ticks_intialized() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (U256::from(500), U256::from(500)));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);
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
        U256::from(500),
    )
    .assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        U256::from(500),
    )
    .assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        U256::from(500),
        token_y,
        U256::from(500),
        None::<&str>,
    )
    .unwrap();

    let _res = create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        -221818,
        -221817,
        Liquidity::new(U256::from(10)),
        pool.sqrt_price,
        pool.sqrt_price,
    ).assert_success();

    let _res = create_position(
        &invariant,
        REGULAR_USER_1,
        pool_key,
        221817,
        221818,
        Liquidity::new(U256::from(10)),
        pool.sqrt_price,
        pool.sqrt_price,
    ).assert_success();

    let tickmap = get_tickmap(&invariant, pool_key);

    assert_eq!(tickmap[0], (0, 0b11));
    assert_eq!(
        tickmap[1],
        (
            get_max_chunk(fee_tier.tick_spacing),
            0b11000000000000000000000000000000000000000000000000000
        )
    );
    assert_eq!(tickmap.len(), 2);
}

#[test]
fn test_get_tickmap_more_chunks_above() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let initial_mint = U256::from(u128::MAX);
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (U256::from(initial_mint), U256::from(initial_mint)));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);
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
        U256::from(initial_mint),
    )
    .assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        U256::from(initial_mint),
    )
    .assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        U256::from(initial_mint),
        token_y,
        U256::from(initial_mint),
        None::<&str>,
    )
    .unwrap();
    for i in (6..52500).step_by(CHUNK_SIZE as usize) {
        create_position(
            &invariant,
            REGULAR_USER_1,
            pool_key,
            i,
            i + 1,
            Liquidity::new(U256::from(10)),
            pool.sqrt_price,
            SqrtPrice::max_instance(),
        ).assert_success();
    }
    let tickmap = get_tickmap(&invariant, pool_key);

    for (i, val) in tickmap.iter().enumerate() {
        let current = 3466 + i as u16;
        assert_eq!(val, &(current, 0b11u64));
    }
}

#[test]
fn test_get_tickmap_more_chunks_below() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let initial_mint = U256::from(u128::MAX);
    let (token_x_program, token_y_program) =
        init_tokens_with_mint(&sys, (U256::from(initial_mint), U256::from(initial_mint)));
    let token_x = ActorId::from(TOKEN_X_ID);
    let token_y = ActorId::from(TOKEN_Y_ID);

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let res = add_fee_tier(&invariant, ADMIN, fee_tier);
    res.assert_single_event().assert_empty().assert_to(ADMIN);
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
        U256::from(initial_mint),
    )
    .assert_success();

    increase_allowance(
        &token_y_program,
        REGULAR_USER_1,
        INVARIANT_ID,
        U256::from(initial_mint),
    )
    .assert_success();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let pool = get_pool(&invariant, token_x, token_y, fee_tier).unwrap();

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        token_x,
        U256::from(initial_mint),
        token_y,
        U256::from(initial_mint),
        None::<&str>,
    )
    .unwrap();
    for i in (-52544..6).step_by(CHUNK_SIZE as usize) {
        create_position(
            &invariant,
            REGULAR_USER_1,
            pool_key,
            i,
            i + 1,
            Liquidity::new(U256::from(10)),
            pool.sqrt_price,
            SqrtPrice::max_instance(),
        ).assert_success();
    }
    let tickmap = get_tickmap(&invariant, pool_key);

    for (i, val) in tickmap.iter().enumerate() {
        let current = 2644 + i as u16;
        assert_eq!(
            val,
            &(
                current,
                0b110000000000000000000000000000000000000000000000000000000000
            )
        );
    }
}

