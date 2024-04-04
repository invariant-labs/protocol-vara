use crate::test_helpers::consts::GEAR_PATH;
use crate::test_helpers::gclient::{
    add_fee_tier, approve, create_pool, create_position, get_all_positions, get_api_user_id,
    get_pool, get_position, get_tick, init_invariant, init_tokens, is_tick_initialized, mint,
    positions_are_identical_no_timestamp, remove_position, transfer_position,
};
use fungible_token_io::InitConfig;
use gclient::{GearApi, Result};
use math::sqrt_price::calculate_sqrt_price;

use contracts::*;
use decimal::*;
use io::*;
use math::{fee_growth::FeeGrowth, liquidity::Liquidity, percentage::Percentage};

#[tokio::test]
async fn test_remove_position_from_empty_list() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = admin_api.clone().with("//Bob").unwrap();

    let mut listener = admin_api.subscribe().await?;
    let invariant = init_invariant(
        &admin_api,
        &mut listener,
        InitInvariant {
            config: InvariantConfig {
                protocol_fee: Percentage::from_scale(6, 3).get() as u128,
                admin: get_api_user_id(&admin_api).into(),
            },
        },
    )
    .await;

    let (token_x, token_y) = init_tokens(&admin_api, &mut listener, InitConfig::default()).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 3).unwrap();

    add_fee_tier(&admin_api, &mut listener, invariant, fee_tier, None).await;

    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

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

    let res = remove_position(&user_api, &mut listener, invariant, 0).await;

    assert_eq!(res, Err(InvariantError::PositionNotFound));

    Ok(())
}

#[tokio::test]
async fn test_add_multiple_positions() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = admin_api.clone().with("//Bob").unwrap();

    let mut listener = admin_api.subscribe().await?;
    let invariant = init_invariant(
        &admin_api,
        &mut listener,
        InitInvariant {
            config: InvariantConfig {
                protocol_fee: Percentage::from_scale(6, 3).get() as u128,
                admin: get_api_user_id(&admin_api).into(),
            },
        },
    )
    .await;

    let (token_x, token_y) = init_tokens(&admin_api, &mut listener, InitConfig::default()).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(6, 3), 3).unwrap();

    add_fee_tier(&admin_api, &mut listener, invariant, fee_tier, None).await;

    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

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

    mint(&user_api, &mut listener, token_x, u128::MAX)
        .await
        .unwrap();
    mint(&user_api, &mut listener, token_y, u128::MAX)
        .await
        .unwrap();

    approve(&user_api, &mut listener, token_x, invariant, u128::MAX)
        .await
        .unwrap();
    approve(&user_api, &mut listener, token_y, invariant, u128::MAX)
        .await
        .unwrap();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let pool = get_pool(&user_api, invariant, token_x, token_y, fee_tier, None)
        .await
        .unwrap();
    //Create positions
    create_position(
        &user_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    create_position(
        &user_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    create_position(
        &user_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[2],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    create_position(
        &user_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[1],
        tick_indexes[4],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    //Remove middle position
    let position_index_to_remove = 1;
    let position_list_before =
        get_all_positions(&user_api, invariant, get_api_user_id(&user_api)).await;

    let last_position = position_list_before.last().cloned().unwrap();

    let _res = remove_position(
        &user_api,
        &mut listener,
        invariant,
        position_index_to_remove,
    )
    .await
    .unwrap();

    let position_list_after =
        get_all_positions(&user_api, invariant, get_api_user_id(&user_api)).await;

    let tested_position = position_list_after[position_index_to_remove as usize];

    assert_eq!(last_position, tested_position);

    // Add position in place of the removed one
    let position_list_before = position_list_after;

    create_position(
        &user_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[1],
        tick_indexes[2],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    let position_list_after =
        get_all_positions(&user_api, invariant, get_api_user_id(&user_api)).await;
    assert_eq!(position_list_before.len() + 1, position_list_after.len());

    // Remove last position
    let position_list_before = position_list_after;
    let position_index_to_remove = position_list_before.len() - 1;
    remove_position(
        &user_api,
        &mut listener,
        invariant,
        position_index_to_remove as u32,
    )
    .await
    .unwrap();

    let position_list_after =
        get_all_positions(&user_api, invariant, get_api_user_id(&user_api)).await;

    assert_eq!(position_list_before.len() - 1, position_list_after.len());

    // Remove all positions
    let position_list_before = position_list_after;

    for i in (0..position_list_before.len()).rev() {
        remove_position(&user_api, &mut listener, invariant, i as u32)
            .await
            .unwrap();
    }

    let position_list_after =
        get_all_positions(&user_api, invariant, get_api_user_id(&user_api)).await;

    assert_eq!(position_list_after.len(), 0);

    //Add position to cleared list
    create_position(
        &user_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    let position_list_after =
        get_all_positions(&user_api, invariant, get_api_user_id(&user_api)).await;

    assert_eq!(position_list_after.len(), 1);

    Ok(())
}
#[tokio::test]
async fn test_only_owner_can_modify_position_list() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let position_owner_api = admin_api.clone().with("//Bob").unwrap();
    // let unauthorized_user_api = .unwrap();

    let mut listener = admin_api.subscribe().await?;

    assert!(listener.blocks_running().await.unwrap());

    let invariant = init_invariant(
        &admin_api,
        &mut listener,
        InitInvariant {
            config: InvariantConfig {
                protocol_fee: Percentage::from_scale(6, 3).get() as u128,
                admin: get_api_user_id(&admin_api).into(),
            },
        },
    )
    .await;
    let (token_x, token_y) = init_tokens(&admin_api, &mut listener, InitConfig::default()).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 3).unwrap();

    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    add_fee_tier(&admin_api, &mut listener, invariant, fee_tier, None).await;

    create_pool(
        &position_owner_api,
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

    mint(&position_owner_api, &mut listener, token_x, u128::MAX)
        .await
        .unwrap();
    mint(&position_owner_api, &mut listener, token_y, u128::MAX)
        .await
        .unwrap();

    approve(
        &position_owner_api,
        &mut listener,
        token_x,
        invariant,
        u128::MAX,
    )
    .await
    .unwrap();
    approve(
        &position_owner_api,
        &mut listener,
        token_y,
        invariant,
        u128::MAX,
    )
    .await
    .unwrap();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);

    let pool = get_pool(
        &position_owner_api,
        invariant,
        token_x,
        token_y,
        fee_tier,
        None,
    )
    .await
    .unwrap();

    //Create positions
    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[2],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[1],
        tick_indexes[4],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    //Remove middle position
    let position_index_to_remove = 1;
    let position_list_before = get_all_positions(
        &position_owner_api,
        invariant,
        get_api_user_id(&position_owner_api),
    )
    .await;

    let last_position = position_list_before.last().cloned().unwrap();

    let _res = remove_position(
        &position_owner_api,
        &mut listener,
        invariant,
        position_index_to_remove,
    )
    .await
    .unwrap();

    let position_list_after = get_all_positions(
        &position_owner_api,
        invariant,
        get_api_user_id(&position_owner_api),
    )
    .await;

    let tested_position = position_list_after[position_index_to_remove as usize];

    assert_eq!(last_position, tested_position);

    // Add position in place of the removed one
    let position_list_before = position_list_after;

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[1],
        tick_indexes[2],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    let position_list_after = get_all_positions(
        &position_owner_api,
        invariant,
        get_api_user_id(&position_owner_api),
    )
    .await;
    assert_eq!(position_list_before.len() + 1, position_list_after.len());

    //Attempt to remove position as another user
    let res = remove_position(
        &admin_api,
        &mut listener,
        invariant,
        position_index_to_remove,
    )
    .await;

    assert_eq!(res, Err(InvariantError::PositionNotFound));
    Ok(())
}
#[tokio::test]
async fn test_transfer_position_ownership() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let position_owner_api = admin_api.clone().with("//Bob").unwrap();
    let recipient_api = admin_api.clone().with("//Carol").unwrap();

    let mut listener = admin_api.subscribe().await?;

    assert!(listener.blocks_running().await.unwrap());

    let invariant = init_invariant(
        &admin_api,
        &mut listener,
        InitInvariant {
            config: InvariantConfig {
                protocol_fee: Percentage::from_scale(6, 3).get() as u128,
                admin: get_api_user_id(&admin_api).into(),
            },
        },
    )
    .await;

    let (token_x, token_y) = init_tokens(&admin_api, &mut listener, InitConfig::default()).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 3).unwrap();

    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    add_fee_tier(&admin_api, &mut listener, invariant, fee_tier, None).await;

    create_pool(
        &position_owner_api,
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

    mint(&position_owner_api, &mut listener, token_x, u128::MAX)
        .await
        .unwrap();
    mint(&position_owner_api, &mut listener, token_y, u128::MAX)
        .await
        .unwrap();

    approve(
        &position_owner_api,
        &mut listener,
        token_x,
        invariant,
        u128::MAX,
    )
    .await
    .unwrap();
    approve(
        &position_owner_api,
        &mut listener,
        token_y,
        invariant,
        u128::MAX,
    )
    .await
    .unwrap();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);

    let pool = get_pool(
        &position_owner_api,
        invariant,
        token_x,
        token_y,
        fee_tier,
        None,
    )
    .await
    .unwrap();

    //Create positions
    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[2],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[1],
        tick_indexes[4],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    let transferred_index = 0;
    let owner_list_before = get_all_positions(
        &position_owner_api,
        invariant,
        get_api_user_id(&position_owner_api),
    )
    .await;
    let recipient_list_before = get_all_positions(
        &position_owner_api,
        invariant,
        get_api_user_id(&recipient_api),
    )
    .await;

    assert_eq!(owner_list_before.len(), 4);
    assert_eq!(recipient_list_before.len(), 0);

    let removed_position = owner_list_before[transferred_index];
    let last_position_before = owner_list_before.last().cloned().unwrap();

    transfer_position(
        &position_owner_api,
        &mut listener,
        invariant,
        transferred_index as u32,
        get_api_user_id(&recipient_api),
        None,
    )
    .await;

    let recipient_position = get_position(
        &position_owner_api,
        invariant,
        get_api_user_id(&recipient_api),
        0,
        None,
    )
    .await
    .unwrap();

    let owner_first_position_after = get_position(
        &position_owner_api,
        invariant,
        get_api_user_id(&position_owner_api),
        0,
        None,
    )
    .await
    .unwrap();

    let owner_list_after = get_all_positions(
        &position_owner_api,
        invariant,
        get_api_user_id(&position_owner_api),
    )
    .await;

    let recipient_list_after = get_all_positions(
        &position_owner_api,
        invariant,
        get_api_user_id(&recipient_api),
    )
    .await;

    assert_eq!(recipient_list_after.len(), recipient_list_before.len() + 1);
    assert_eq!(owner_list_before.len() - 1, owner_list_after.len());

    assert_eq!(last_position_before, owner_first_position_after);

    positions_are_identical_no_timestamp(&recipient_position, &removed_position);
    Ok(())
}

#[tokio::test]
async fn test_only_owner_can_transfer_position() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let position_owner_api = admin_api.clone().with("//Bob").unwrap();
    let recipient_api = position_owner_api.clone().with("//Carol").unwrap();

    let mut listener = admin_api.subscribe().await?;

    assert!(listener.blocks_running().await.unwrap());

    let invariant = init_invariant(
        &admin_api,
        &mut listener,
        InitInvariant {
            config: InvariantConfig {
                protocol_fee: Percentage::from_scale(6, 3).get() as u128,
                admin: get_api_user_id(&admin_api).into(),
            },
        },
    )
    .await;

    let (token_x, token_y) = init_tokens(&admin_api, &mut listener, InitConfig::default()).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 3).unwrap();

    let init_tick = -23028;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    add_fee_tier(&admin_api, &mut listener, invariant, fee_tier, None).await;

    create_pool(
        &position_owner_api,
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

    mint(&position_owner_api, &mut listener, token_x, u128::MAX)
        .await
        .unwrap();
    mint(&position_owner_api, &mut listener, token_y, u128::MAX)
        .await
        .unwrap();

    approve(
        &position_owner_api,
        &mut listener,
        token_x,
        invariant,
        u128::MAX,
    )
    .await
    .unwrap();
    approve(
        &position_owner_api,
        &mut listener,
        token_y,
        invariant,
        u128::MAX,
    )
    .await
    .unwrap();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let tick_indexes = [-9780, -42, 0, 9, 276, 32343, -50001];
    let liquidity_delta = Liquidity::from_integer(1_000_000);

    let pool = get_pool(
        &position_owner_api,
        invariant,
        token_x,
        token_y,
        fee_tier,
        None,
    )
    .await
    .unwrap();

    //Create positions
    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[1],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[0],
        tick_indexes[2],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        tick_indexes[1],
        tick_indexes[4],
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    let transferred_index = 0;
    let owner_list_before = get_all_positions(
        &position_owner_api,
        invariant,
        get_api_user_id(&position_owner_api),
    )
    .await;
    let recipient_list_before = get_all_positions(
        &position_owner_api,
        invariant,
        get_api_user_id(&recipient_api),
    )
    .await;

    assert_eq!(owner_list_before.len(), 4);
    assert_eq!(recipient_list_before.len(), 0);

    transfer_position(
        &admin_api,
        &mut listener,
        invariant,
        transferred_index as u32,
        get_api_user_id(&recipient_api),
        InvariantError::PositionNotFound.into(),
    )
    .await;

    let owner_list_after = get_all_positions(
        &position_owner_api,
        invariant,
        get_api_user_id(&position_owner_api),
    )
    .await;

    let recipient_list_after = get_all_positions(
        &position_owner_api,
        invariant,
        get_api_user_id(&recipient_api),
    )
    .await;

    assert_eq!(recipient_list_after.len(), recipient_list_before.len());

    assert_eq!(owner_list_before.len(), owner_list_after.len());

    Ok(())
}

#[tokio::test]
async fn test_multiple_positions_on_same_tick() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let position_owner_api = admin_api.clone().with("//Bob").unwrap();

    let mut listener = admin_api.subscribe().await?;

    assert!(listener.blocks_running().await.unwrap());

    let invariant = init_invariant(
        &admin_api,
        &mut listener,
        InitInvariant {
            config: InvariantConfig {
                protocol_fee: Percentage::from_integer(0).get() as u128,
                admin: get_api_user_id(&admin_api).into(),
            },
        },
    )
    .await;

    let (token_x, token_y) = init_tokens(&admin_api, &mut listener, InitConfig::default()).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 10).unwrap();
    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    add_fee_tier(&admin_api, &mut listener, invariant, fee_tier, None).await;

    create_pool(
        &position_owner_api,
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

    mint(&position_owner_api, &mut listener, token_x, u128::MAX)
        .await
        .unwrap();
    mint(&position_owner_api, &mut listener, token_y, u128::MAX)
        .await
        .unwrap();

    approve(
        &position_owner_api,
        &mut listener,
        token_x,
        invariant,
        u128::MAX,
    )
    .await
    .unwrap();
    approve(
        &position_owner_api,
        &mut listener,
        token_y,
        invariant,
        u128::MAX,
    )
    .await
    .unwrap();

    let pool_key = PoolKey::new(token_x.into(), token_y.into(), fee_tier).unwrap();
    let lower_tick_index = -10;
    let upper_tick_index = 10;
    let liquidity_delta = Liquidity::new(100);

    let pool = get_pool(
        &position_owner_api,
        invariant,
        token_x,
        token_y,
        fee_tier,
        None,
    )
    .await
    .unwrap();

    // Three position on same lower and upper tick
    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;
    let owner_id = get_api_user_id(&position_owner_api);

    let first_position = get_position(&position_owner_api, invariant, owner_id, 0, None)
        .await
        .unwrap();

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    let second_position = get_position(&position_owner_api, invariant, owner_id, 1, None)
        .await
        .unwrap();

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    let third_position = get_position(&position_owner_api, invariant, owner_id, 2, None)
        .await
        .unwrap();

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

    let pool = get_pool(
        &position_owner_api,
        invariant,
        token_x,
        token_y,
        fee_tier,
        None,
    )
    .await
    .unwrap();
    let lower_tick = get_tick(
        &position_owner_api,
        invariant,
        pool_key,
        lower_tick_index,
        None,
    )
    .await
    .unwrap();
    let upper_tick = get_tick(
        &position_owner_api,
        invariant,
        pool_key,
        upper_tick_index,
        None,
    )
    .await
    .unwrap();

    let expected_liquidity = Liquidity::new(liquidity_delta.get() * 3);
    let zero_fee = FeeGrowth::new(0);

    assert_eq!(lower_tick.index, lower_tick_index);
    assert_eq!(upper_tick.index, upper_tick_index);
    assert_eq!(lower_tick.liquidity_gross, expected_liquidity);
    assert_eq!(upper_tick.liquidity_gross, expected_liquidity);
    assert_eq!(lower_tick.liquidity_change, expected_liquidity);
    assert_eq!(upper_tick.liquidity_change, expected_liquidity);
    assert!(lower_tick.sign);
    assert!(!upper_tick.sign);

    // Check pool
    assert_eq!(pool.liquidity, expected_liquidity);
    assert_eq!(pool.current_tick_index, init_tick);

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

    // Three positions on different ticks
    let lower_tick_index = -10;
    let upper_tick_index = 10;
    let zero_fee = FeeGrowth::new(0);

    let liquidity_delta = Liquidity::new(100);

    create_position(
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    let first_position = get_position(&position_owner_api, invariant, owner_id, 3, None)
        .await
        .unwrap();

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
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    let second_position = get_position(&position_owner_api, invariant, owner_id, 4, None)
        .await
        .unwrap();

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
        &position_owner_api,
        &mut listener,
        invariant,
        pool_key,
        lower_tick_index,
        upper_tick_index,
        liquidity_delta,
        pool.sqrt_price,
        pool.sqrt_price,
        None,
    )
    .await;

    let third_position = get_position(&position_owner_api, invariant, owner_id, 5, None)
        .await
        .unwrap();

    // Check third position
    assert_eq!(third_position.pool_key, pool_key);
    assert_eq!(third_position.liquidity, liquidity_delta);
    assert_eq!(third_position.lower_tick_index, lower_tick_index);
    assert_eq!(third_position.upper_tick_index, upper_tick_index);
    assert_eq!(third_position.fee_growth_inside_x, zero_fee);
    assert_eq!(third_position.fee_growth_inside_y, zero_fee);

    let pool = get_pool(
        &position_owner_api,
        invariant,
        token_x,
        token_y,
        fee_tier,
        None,
    )
    .await
    .unwrap();

    let tick_n20 = get_tick(&position_owner_api, invariant, pool_key, -20, None)
        .await
        .unwrap();
    let tick_n10 = get_tick(&position_owner_api, invariant, pool_key, -10, None)
        .await
        .unwrap();
    let tick_10 = get_tick(&position_owner_api, invariant, pool_key, 10, None)
        .await
        .unwrap();

    let tick_20 = get_tick(&position_owner_api, invariant, pool_key, 20, None)
        .await
        .unwrap();

    let tick_n20_bit = is_tick_initialized(&position_owner_api, invariant, pool_key, -20).await;
    let tick_n10_bit = is_tick_initialized(&position_owner_api, invariant, pool_key, -10).await;
    let tick_10_bit = is_tick_initialized(&position_owner_api, invariant, pool_key, 10).await;
    let tick_20_bit = is_tick_initialized(&position_owner_api, invariant, pool_key, 20).await;

    let expected_active_liquidity = Liquidity::new(400);

    // Check tick -20
    assert_eq!(tick_n20.index, -20);
    assert_eq!(tick_n20.liquidity_gross, Liquidity::new(100));
    assert_eq!(tick_n20.liquidity_change, Liquidity::new(100));
    assert!(tick_n20.sign);
    assert!(tick_n20_bit);

    // Check tick -10
    assert_eq!(tick_n10.index, -10);
    assert_eq!(tick_n10.liquidity_gross, Liquidity::new(500));
    assert_eq!(tick_n10.liquidity_change, Liquidity::new(300));
    assert!(tick_n10.sign);
    assert!(tick_n10_bit);

    // Check tick 10
    assert_eq!(tick_10.index, 10);
    assert_eq!(tick_10.liquidity_gross, Liquidity::new(500));
    assert_eq!(tick_10.liquidity_change, Liquidity::new(300));
    assert!(!tick_10.sign);
    assert!(tick_10_bit);

    // Check tick 20
    assert_eq!(tick_20.index, 20);
    assert_eq!(tick_20.liquidity_gross, Liquidity::new(100));
    assert_eq!(tick_20.liquidity_change, Liquidity::new(100));
    assert!(!tick_20.sign);
    assert!(tick_20_bit);

    // Check pool
    assert_eq!(pool.liquidity, expected_active_liquidity);
    assert_eq!(pool.current_tick_index, init_tick);

    Ok(())
}
