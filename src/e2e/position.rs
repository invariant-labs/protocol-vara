use crate::test_helpers::consts::GEAR_PATH;
use crate::test_helpers::gclient::token::init_tokens;
use crate::test_helpers::gclient::{
    add_fee_tier, approve, balance_of, create_pool, create_position, get_api_user_id,
    get_new_token, get_pool, get_pools, get_position, get_tick, init_invariant, init_token, mint,
    pools_are_identical_no_timestamp, token,
};
use contracts::{pool_key, FeeTier, InvariantError, Pool, PoolKey, Tick};
use decimal::*;
use fungible_token_io::InitConfig;
use gclient::{GearApi, Result};
use gstd::prelude::*;
use io::*;
use math::sqrt_price::get_max_tick;
use math::{
    fee_growth::FeeGrowth,
    liquidity::Liquidity,
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
};

#[tokio::test]
async fn test_create_position() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = admin_api.clone().with("//Bob").unwrap();

    let mut listener = admin_api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&admin_api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&admin_api, &mut listener, init).await;

    let (token_x, token_y) = init_tokens(&user_api, &mut listener, InitConfig::default()).await;

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    add_fee_tier(&admin_api, &mut listener, invariant, fee_tier, None).await;

    create_pool(
        &user_api,
        &mut listener,
        invariant,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
        None,
    )
    .await;
    get_pool(&user_api, invariant, token_x, token_y, fee_tier, None)
        .await
        .unwrap();

    mint(&user_api, &mut listener, token_x, 500).await.unwrap();
    mint(&user_api, &mut listener, token_y, 500).await.unwrap();

    approve(&user_api, &mut listener, token_x, invariant, 500)
        .await
        .unwrap();
    approve(&user_api, &mut listener, token_y, invariant, 500)
        .await
        .unwrap();
    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    create_position(
        &user_api,
        &mut listener,
        invariant,
        pool_key,
        -10,
        10,
        Liquidity::new(10),
        SqrtPrice::new(0),
        SqrtPrice::max_instance(),
        None,
    )
    .await;
    Ok(())
}

#[tokio::test]
async fn test_position_below_current_tick() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_1_api = admin_api.clone().with("//Bob").unwrap();

    let mut listener = admin_api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&admin_api).into(),
            protocol_fee: 0,
        },
    };

    let invariant = init_invariant(&admin_api, &mut listener, init).await;

    let (token_x, token_y) = init_tokens(&user_1_api, &mut listener, InitConfig::default()).await;

    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

    let initial_balance = 10_000_000_000;

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();

    add_fee_tier(&admin_api, &mut listener, invariant, fee_tier, None).await;
    create_pool(
        &user_1_api,
        &mut listener,
        invariant,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
        None,
    )
    .await;
    let pool_state_before = get_pool(&user_1_api, invariant, token_x, token_y, fee_tier, None)
        .await
        .unwrap();

    mint(&user_1_api, &mut listener, token_x, initial_balance)
        .await
        .unwrap();
    mint(&user_1_api, &mut listener, token_y, initial_balance)
        .await
        .unwrap();

    approve(
        &user_1_api,
        &mut listener,
        token_x,
        invariant,
        initial_balance,
    )
    .await
    .unwrap();
    approve(
        &user_1_api,
        &mut listener,
        token_y,
        invariant,
        initial_balance,
    )
    .await
    .unwrap();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();

    let lower_tick_index = -46080;
    let upper_tick_index = -23040;
    let liquidity_delta = Liquidity::from_integer(10_000);

    create_position(
        &user_1_api,
        &mut listener,
        invariant,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool_state_before.sqrt_price,
        SqrtPrice::max_instance(),
        None,
    )
    .await;

    let pool_state = get_pool(&user_1_api, invariant, token_x, token_y, fee_tier, None)
        .await
        .unwrap();

    let position_state = get_position(
        &user_1_api,
        invariant,
        get_api_user_id(&user_1_api),
        0,
        None,
    )
    .await
    .unwrap();

    let lower_tick = get_tick(&user_1_api, invariant, pool_key, lower_tick_index, None)
        .await
        .unwrap();
    let upper_tick = get_tick(&user_1_api, invariant, pool_key, upper_tick_index, None)
        .await
        .unwrap();
    let user_1_x = balance_of(&user_1_api, token_x, get_api_user_id(&user_1_api)).await;
    let user_1_y = balance_of(&user_1_api, token_y, get_api_user_id(&user_1_api)).await;
    let invariant_x = balance_of(&user_1_api, token_x, invariant).await;
    let invariant_y = balance_of(&user_1_api, token_y, invariant).await;

    let zero_fee = FeeGrowth::new(0);
    let expected_x_increase = 0;
    let expected_y_increase = 2162;

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

    Ok(())
}

#[tokio::test]
async fn test_position_within_current_tick() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_1_api = admin_api.clone().with("//Bob").unwrap();

    let mut listener = admin_api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&admin_api).into(),
            protocol_fee: 0,
        },
    };

    let invariant = init_invariant(&admin_api, &mut listener, init).await;

    let (token_x, token_y) = init_tokens(&user_1_api, &mut listener, InitConfig::default()).await;

    let max_tick_test = 177_450;
    let min_tick_test = -max_tick_test;
    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();

    let initial_balance = 100_000_000;

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();

    add_fee_tier(&admin_api, &mut listener, invariant, fee_tier, None).await;

    create_pool(
        &user_1_api,
        &mut listener,
        invariant,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
        None,
    )
    .await;
    let pool_state = get_pool(&user_1_api, invariant, token_x, token_y, fee_tier, None)
        .await
        .unwrap();

    mint(&user_1_api, &mut listener, token_x, initial_balance)
        .await
        .unwrap();
    mint(&user_1_api, &mut listener, token_y, initial_balance)
        .await
        .unwrap();

    approve(
        &user_1_api,
        &mut listener,
        token_x,
        invariant,
        initial_balance,
    )
    .await
    .unwrap();
    approve(
        &user_1_api,
        &mut listener,
        token_y,
        invariant,
        initial_balance,
    )
    .await
    .unwrap();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let lower_tick_index = min_tick_test + 10;
    let upper_tick_index = max_tick_test - 10;
    let liquidity_delta = Liquidity::from_integer(100);

    create_position(
        &user_1_api,
        &mut listener,
        invariant,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool_state.sqrt_price,
        SqrtPrice::max_instance(),
        None,
    )
    .await;

    let pool_state = get_pool(&user_1_api, invariant, token_x, token_y, fee_tier, None)
        .await
        .unwrap();

    let position_state = get_position(
        &user_1_api,
        invariant,
        get_api_user_id(&user_1_api),
        0,
        None,
    )
    .await
    .unwrap();

    let lower_tick = get_tick(&user_1_api, invariant, pool_key, lower_tick_index, None)
        .await
        .unwrap();
    let upper_tick = get_tick(&user_1_api, invariant, pool_key, upper_tick_index, None)
        .await
        .unwrap();
    let user_1_x = balance_of(&user_1_api, token_x, get_api_user_id(&user_1_api)).await;
    let user_1_y = balance_of(&user_1_api, token_y, get_api_user_id(&user_1_api)).await;
    let invariant_x = balance_of(&user_1_api, token_x, invariant).await;
    let invariant_y = balance_of(&user_1_api, token_y, invariant).await;

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

    Ok(())
}

#[tokio::test]
async fn test_position_above_current_tick() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_1_api = admin_api.clone().with("//Bob").unwrap();

    let mut listener = admin_api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&admin_api).into(),
            protocol_fee: 0,
        },
    };

    let invariant = init_invariant(&admin_api, &mut listener, init).await;

    let (token_x, token_y) = init_tokens(&user_1_api, &mut listener, InitConfig::default()).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();
    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();
    let remove_position_index = 0;
    let initial_balance = 10_000_000_000;

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();

    add_fee_tier(&admin_api, &mut listener, invariant, fee_tier, None).await;

    create_pool(
        &user_1_api,
        &mut listener,
        invariant,
        token_x,
        token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
        None,
    )
    .await;
    let pool_state_before = get_pool(&user_1_api, invariant, token_x, token_y, fee_tier, None)
        .await
        .unwrap();

    mint(&user_1_api, &mut listener, token_x, initial_balance)
        .await
        .unwrap();
    mint(&user_1_api, &mut listener, token_y, initial_balance)
        .await
        .unwrap();

    approve(
        &user_1_api,
        &mut listener,
        token_x,
        invariant,
        initial_balance,
    )
    .await
    .unwrap();
    approve(
        &user_1_api,
        &mut listener,
        token_y,
        invariant,
        initial_balance,
    )
    .await
    .unwrap();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();

    let lower_tick_index = -22980;
    let upper_tick_index = 0;
    let liquidity_delta = Liquidity::from_integer(10_000);

    create_position(
        &user_1_api,
        &mut listener,
        invariant,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        SqrtPrice::new(0),
        SqrtPrice::max_instance(),
        None,
    )
    .await;

    let pool_state = get_pool(&user_1_api, invariant, token_x, token_y, fee_tier, None)
        .await
        .unwrap();
    let position_state = get_position(
        &user_1_api,
        invariant,
        get_api_user_id(&user_1_api),
        remove_position_index,
        None,
    )
    .await
    .unwrap();

    let lower_tick = get_tick(&user_1_api, invariant, pool_key, lower_tick_index, None)
        .await
        .unwrap();
    let upper_tick = get_tick(&user_1_api, invariant, pool_key, upper_tick_index, None)
        .await
        .unwrap();
    let user_1_x = balance_of(&user_1_api, token_x, get_api_user_id(&user_1_api)).await;
    let user_1_y = balance_of(&user_1_api, token_y, get_api_user_id(&user_1_api)).await;
    let invariant_x = balance_of(&user_1_api, token_x, invariant).await;
    let invariant_y = balance_of(&user_1_api, token_y, invariant).await;

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

    Ok(())
}
