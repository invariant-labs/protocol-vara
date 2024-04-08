use crate::test_helpers::consts::GEAR_PATH;
use crate::test_helpers::gclient::entrypoints::{create_position, get_pool};
use crate::test_helpers::gclient::{get_tick, snippets::*};

use contracts::InvariantError;
use decimal::*;
use gclient::{GearApi, Result};
use gstd::prelude::*;
use math::{liquidity::Liquidity, sqrt_price::SqrtPrice};

#[tokio::test]
async fn test_position_slippage_zero_slippage_and_inside_range() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = admin_api.clone().with("//Bob").unwrap();

    let mut listener = admin_api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let (invariant, token_x, token_y) =
        init_slippage_invariant_and_tokens(&admin_api, &mut listener).await;
    let pool_key = init_slippage_pool_with_liquidity(
        &admin_api,
        &user_api,
        &mut listener,
        invariant,
        token_x,
        token_y,
    )
    .await;

    let pool = get_pool(
        &user_api,
        invariant,
        token_x,
        token_y,
        pool_key.fee_tier,
        None,
    )
    .await
    .unwrap();

    // zero slippage
    {
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let known_price = pool.sqrt_price;
        let tick = pool_key.fee_tier.tick_spacing as i32;

        create_position(
            &user_api,
            &mut listener,
            invariant,
            pool_key,
            -tick,
            tick,
            liquidity_delta,
            known_price,
            known_price,
            None,
        )
        .await;
    };

    // inside range
    {
        let liquidity_delta = Liquidity::from_integer(1_000_000);
        let limit_lower = SqrtPrice::new(994734637981406576896367);
        let limit_upper = SqrtPrice::new(1025038048074314166333500);

        let tick = pool_key.fee_tier.tick_spacing as i32;

        create_position(
            &user_api,
            &mut listener,
            invariant,
            pool_key,
            -tick,
            tick,
            liquidity_delta,
            limit_lower,
            limit_upper,
            None,
        )
        .await
    }

    Ok(())
}

#[tokio::test]
async fn test_position_slippage_below_range() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = admin_api.clone().with("//Bob").unwrap();

    let mut listener = admin_api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let (invariant, token_x, token_y) =
        init_slippage_invariant_and_tokens(&admin_api, &mut listener).await;
    let pool_key = init_slippage_pool_with_liquidity(
        &admin_api,
        &user_api,
        &mut listener,
        invariant,
        token_x,
        token_y,
    )
    .await;

    get_pool(
        &user_api,
        invariant,
        token_x,
        token_y,
        pool_key.fee_tier,
        None,
    )
    .await
    .unwrap();

    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let limit_lower = SqrtPrice::new(1014432353584998786339859);
    let limit_upper = SqrtPrice::new(1045335831204498605270797);
    let tick = pool_key.fee_tier.tick_spacing as i32;

    create_position(
        &user_api,
        &mut listener,
        invariant,
        pool_key,
        -tick,
        tick,
        liquidity_delta,
        limit_lower,
        limit_upper,
        InvariantError::PriceLimitReached.into(),
    )
    .await;

    let _lower_tick = get_tick(&user_api, invariant, pool_key, -tick, InvariantError::TickNotFound.into()).await;
    let _upper_tick = get_tick(&user_api, invariant, pool_key, tick, InvariantError::TickNotFound.into()).await;

    Ok(())
}

#[tokio::test]
async fn test_position_slippage_above_range() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    let user_api = admin_api.clone().with("//Bob").unwrap();

    let mut listener = admin_api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let (invariant, token_x, token_y) =
        init_slippage_invariant_and_tokens(&admin_api, &mut listener).await;
    let pool_key = init_slippage_pool_with_liquidity(
        &admin_api,
        &user_api,
        &mut listener,
        invariant,
        token_x,
        token_y,
    )
    .await;

    get_pool(
        &user_api,
        invariant,
        token_x,
        token_y,
        pool_key.fee_tier,
        None,
    )
    .await
    .unwrap();

    let liquidity_delta = Liquidity::from_integer(1_000_000);
    let limit_lower = SqrtPrice::new(955339206774222158009382);
    let limit_upper = SqrtPrice::new(984442481813945288458906);
    let tick = pool_key.fee_tier.tick_spacing as i32;

    create_position(
        &user_api,
        &mut listener,
        invariant,
        pool_key,
        -tick,
        tick,
        liquidity_delta,
        limit_lower,
        limit_upper,
        InvariantError::PriceLimitReached.into(),
    )
    .await;

    let _lower_tick = get_tick(&user_api, invariant, pool_key, -tick, InvariantError::TickNotFound.into()).await;
    let _upper_tick = get_tick(&user_api, invariant, pool_key, tick, InvariantError::TickNotFound.into()).await;

    Ok(())
}
