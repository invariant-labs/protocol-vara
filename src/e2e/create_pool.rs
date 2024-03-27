use crate::test_helpers::consts::GEAR_PATH;
use crate::test_helpers::gclient::{
    add_fee_tier, create_pool, get_api_user_id,
    get_new_token, get_pool, init_invariant
};
use contracts::{FeeTier, InvariantError, Pool};
use decimal::num_traits::Inv;
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
        &api,
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
    let block_timestamp = api.last_block_timestamp().await.unwrap();

    assert_eq!(
        get_pool(&api, invariant, token_0, token_1, fee_tier, None)
            .await
            .unwrap(),
        Pool {
            fee_receiver: get_api_user_id(&api).into(),
            sqrt_price: init_sqrt_price,
            current_tick_index: init_tick,
            last_timestamp: block_timestamp,
            start_timestamp: block_timestamp,
            ..Pool::default()
        }
    );
    Ok(())
}

#[tokio::test]
async fn test_create_pool_x_to_y_and_y_to_x() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let mut token_0 = get_api_user_id(&api);
    token_0[0] = token_0[0].wrapping_add(1);
    let mut token_1 = token_0;
    token_1[0] = token_1[0].wrapping_add(1);

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
        &api,
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
        &api,
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

    Ok(())
}

#[tokio::test]
async fn test_create_pool_with_same_tokens() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let mut token_0 = get_api_user_id(&api);
    token_0[0] = token_0[0].wrapping_add(1);

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
        &api,
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

    Ok(())
}

#[tokio::test]
async fn test_create_pool_fee_tier_not_added() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let mut token_0 = get_api_user_id(&api);
    token_0[0] = token_0[0].wrapping_add(1);
    let mut token_1 = token_0;
    token_1[0] = token_1[0].wrapping_add(1);

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
        &api,
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

    Ok(())
}

#[tokio::test]
async fn test_create_pool_init_tick_not_divisible_by_tick_spacing() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let mut token_0 = get_api_user_id(&api);
    token_0[0] = token_0[0].wrapping_add(1);
    let mut token_1 = token_0;
    token_1[0] = token_1[0].wrapping_add(1);

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
        &api,
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

    Ok(())
}

#[tokio::test]
async fn test_create_pool_init_sqrt_price_minimal_difference_from_tick() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let mut token_0 = get_api_user_id(&api);
    token_0[0] = token_0[0].wrapping_add(1);
    let mut token_1 = token_0;
    token_1[0] = token_1[0].wrapping_add(1);

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
        &api,
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
    let mut token_0 = get_api_user_id(&api);
    token_0[0] = token_0[0].wrapping_add(1);
    let mut token_1 = token_0;
    token_1[0] = token_1[0].wrapping_add(1);

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
        &api,
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
        &api,
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
    let mut token_0 = get_api_user_id(&api);
    token_0[0] = token_0[0].wrapping_add(1);
    let mut token_1 = token_0;
    token_1[0] = token_1[0].wrapping_add(1);

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
        &api,
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
        &api,
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
        get_pool(&api, invariant, token_0, token_1, fee_tier, None)
            .await
            .unwrap()
            .current_tick_index,
        correct_init_tick
    );

    Ok(())
}
#[tokio::test]
async fn test_create_many_pools_success() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let mut token_0 = get_new_token(get_api_user_id(&api));
    let mut token_1 = get_new_token(token_0);


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
    for i in 0..1000 {
        create_pool(
            &api,
            &mut listener,
            invariant,
            token_0,
            token_1,
            fee_tier,
            init_sqrt_price,
            init_tick,
            None,
        );
        get_pool(&api, invariant, token_0, token_1, fee_tier, None).await.unwrap();
        token_1 = get_new_token(token_1);
    };

    Ok(())
}