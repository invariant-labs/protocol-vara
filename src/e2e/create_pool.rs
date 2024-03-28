use crate::test_helpers::consts::GEAR_PATH;
use crate::test_helpers::gclient::{
    add_fee_tier, create_pool, get_api_user_id,
    get_new_token, get_pool, get_pools, init_invariant, pools_are_identical_no_timestamp
};
use contracts::{FeeTier, InvariantError, Pool, PoolKey};
use decimal::*;
use gclient::{GearApi, Result};
use gstd::prelude::*;
use io::*;
use math::{
    percentage::Percentage,
    sqrt_price::{calculate_sqrt_price, SqrtPrice},
};

#[tokio::test]
async fn test_create_pool() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = api.clone().with("//Bob").unwrap();
    let token_0 = get_new_token(get_api_user_id(&api));
    let token_1 = get_new_token(token_0);

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    add_fee_tier(&api, &mut listener, invariant, fee_tier, None).await;
    
    create_pool(
        &user_api,
        &mut listener,
        invariant,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
        None,
    )
    .await;

    assert_eq!(
        get_pools(&api, invariant, u8::MAX, 0, None).await.unwrap(),
        vec![PoolKey{token_x: token_0.into(), token_y: token_1.into(), fee_tier}]
    );

    let pool = get_pool(&api, invariant, token_0, token_1, fee_tier, None).await.expect("Pool doesn't exist");
    let expected_pool = Pool {
        sqrt_price: init_sqrt_price,
        current_tick_index: init_tick,
        fee_receiver: get_api_user_id(&api).into(),
        ..Pool::default()
    };

    pools_are_identical_no_timestamp(&pool, &expected_pool);

    Ok(())
}

#[tokio::test]
async fn test_create_pool_x_to_y_and_y_to_x() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = api.clone().with("//Bob").unwrap();

    let token_0 = get_new_token(get_api_user_id(&api));
    let token_1 = get_new_token(token_0);

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    add_fee_tier(&api, &mut listener, invariant, fee_tier, None).await;

    create_pool(
        &user_api,
        &mut listener,
        invariant,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
        None,
    )
    .await;
    create_pool(
        &user_api,
        &mut listener,
        invariant,
        token_1,
        token_0,
        fee_tier,
        init_sqrt_price,
        init_tick,
        InvariantError::PoolAlreadyExist.into(),
    )
    .await;

    assert_eq!(
        get_pools(&api, invariant, u8::MAX, 0, None).await.unwrap(),
        vec![PoolKey{token_x: token_0.into(), token_y: token_1.into(), fee_tier}]
    );

    Ok(())
}

#[tokio::test]
async fn test_create_pool_with_same_tokens() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = api.clone().with("//Bob").unwrap();

    let token_0 = get_new_token(get_api_user_id(&api));

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    add_fee_tier(&api, &mut listener, invariant, fee_tier, None).await;

    create_pool(
        &user_api,
        &mut listener,
        invariant,
        token_0,
        token_0,
        fee_tier,
        init_sqrt_price,
        init_tick,
        InvariantError::TokensAreSame.into(),
    )
    .await;

    assert_eq!(
        get_pools(&api, invariant, u8::MAX, 0, None).await.unwrap(),
        vec![]
    );
    Ok(())
}

#[tokio::test]
async fn test_create_pool_fee_tier_not_added() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = api.clone().with("//Bob").unwrap();
    
    let token_0 = get_new_token(get_api_user_id(&api));
    let token_1 = get_new_token(token_0);

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 100).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    create_pool(
        &user_api,
        &mut listener,
        invariant,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
        InvariantError::FeeTierNotFound.into(),
    )
    .await;

    assert_eq!(
        get_pools(&api, invariant, u8::MAX, 0, None).await.unwrap(),
        vec![]
    );
    Ok(())
}

#[tokio::test]
async fn test_create_pool_init_tick_not_divisible_by_tick_spacing() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = api.clone().with("//Bob").unwrap();

    let token_0 = get_new_token(get_api_user_id(&api));
    let token_1 = get_new_token(token_0);

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 3).unwrap();

    let init_tick = 2;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    add_fee_tier(&api, &mut listener, invariant, fee_tier, None).await;
    create_pool(
        &user_api,
        &mut listener,
        invariant,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
        InvariantError::InvalidInitTick.into(),
    )
    .await;

    assert_eq!(
        get_pools(&api, invariant, u8::MAX, 0, None).await.unwrap(),
        vec![]
    );
    Ok(())
}

#[tokio::test]
async fn test_create_pool_init_sqrt_price_minimal_difference_from_tick() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = api.clone().with("//Bob").unwrap();

    let token_0 = get_new_token(get_api_user_id(&api));
    let token_1 = get_new_token(token_0);

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 3).unwrap();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap() + SqrtPrice::new(1);

    add_fee_tier(&api, &mut listener, invariant, fee_tier, None).await;
    create_pool(
        &user_api,
        &mut listener,
        invariant,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
        None,
    )
    .await;

    assert_eq!(
        get_pools(&api, invariant, u8::MAX, 0, None).await.unwrap(),
        vec![PoolKey{token_x: token_0.into(), token_y: token_1.into(), fee_tier}]
    );

    assert_eq!(
        get_pool(&api, invariant, token_0, token_1, fee_tier, None)
            .await
            .unwrap()
            .current_tick_index,
        init_tick
    );

    Ok(())
}

#[tokio::test]
async fn test_create_pool_init_sqrt_price_has_closer_init_tick() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = api.clone().with("//Bob").unwrap();

    let token_0 = get_new_token(get_api_user_id(&api));
    let token_1 = get_new_token(token_0);

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 1).unwrap();

    let init_tick = 2;
    let init_sqrt_price = SqrtPrice::new(1000175003749000000000000);

    add_fee_tier(&api, &mut listener, invariant, fee_tier, None).await;

    create_pool(
        &user_api,
        &mut listener,
        invariant,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
        InvariantError::InvalidInitSqrtPrice.into(),
    )
    .await;

    let correct_init_tick = 3;

    create_pool(
        &user_api,
        &mut listener,
        invariant,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        correct_init_tick,
        None,
    )
    .await;

    assert_eq!(
        get_pools(&api, invariant, u8::MAX, 0, None).await.unwrap(),
        vec![PoolKey{token_x: token_0.into(), token_y: token_1.into(), fee_tier}]
    );

    assert_eq!(
        get_pool(&api, invariant, token_0, token_1, fee_tier, None)
            .await
            .unwrap()
            .current_tick_index,
        correct_init_tick
    );

    Ok(())
}
#[tokio::test]
async fn test_create_pool_init_sqrt_price_has_closer_init_tick_spacing_over_one() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = api.clone().with("//Bob").unwrap();

    let token_0 = get_new_token(get_api_user_id(&api));
    let token_1 = get_new_token(token_0);

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(5, 1), 3).unwrap();

    let init_tick = 0;
    let init_sqrt_price = SqrtPrice::new(1000225003749000000000000);

    add_fee_tier(&api, &mut listener, invariant, fee_tier, None).await;

    create_pool(
        &user_api,
        &mut listener,
        invariant,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
        InvariantError::InvalidInitSqrtPrice.into(),
    )
    .await;

    let correct_init_tick = 3;

    create_pool(
        &user_api,
        &mut listener,
        invariant,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        correct_init_tick,
        None,
    )
    .await;

    assert_eq!(
        get_pools(&api, invariant, u8::MAX, 0, None).await.unwrap(),
        vec![PoolKey{token_x: token_0.into(), token_y: token_1.into(), fee_tier}]
    );

    assert_eq!(
        get_pool(&api, invariant, token_0, token_1, fee_tier, None)
            .await
            .unwrap()
            .current_tick_index,
        correct_init_tick
    );

    Ok(())
}