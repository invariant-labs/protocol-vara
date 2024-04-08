use crate::test_helpers::gclient::{
    add_fee_tier, init_invariant, get_fee_tiers, fee_tier_exists,
    get_api_user_id,
};
use crate::test_helpers::consts::GEAR_PATH;
use gclient::{GearApi, Result};
use gstd::prelude::*;
use io::*;
use contracts::{InvariantError, FeeTier};
use math::types::percentage::Percentage;
use decimal::*;

#[tokio::test]
async fn test_add_multiple_fee_tiers() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let first_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    add_fee_tier(&api, &mut listener, invariant, first_fee_tier, None).await;
    let second_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 2).unwrap();
    add_fee_tier(&api, &mut listener, invariant, second_fee_tier, None).await;
    let third_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 4).unwrap();
    add_fee_tier(&api, &mut listener, invariant, third_fee_tier, None).await;


    assert!(fee_tier_exists(&api, invariant, first_fee_tier).await);
    assert!(fee_tier_exists(&api, invariant, second_fee_tier).await);
    assert!(fee_tier_exists(&api, invariant, third_fee_tier).await);

    let fee_tiers = get_fee_tiers(&api, invariant).await;
    assert_eq!(fee_tiers, vec![first_fee_tier, second_fee_tier, third_fee_tier]);

    Ok(())
}

#[tokio::test]
async fn test_add_existing_fee_tier() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let first_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    add_fee_tier(&api, &mut listener, invariant, first_fee_tier, None).await;
    let second_fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    add_fee_tier(&api, &mut listener, invariant, second_fee_tier, InvariantError::FeeTierAlreadyExist.into()).await;

    Ok(())
}

#[tokio::test]
async fn test_add_fee_tier_not_admin() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let mut user_id = get_api_user_id(&api);
    user_id[1] = user_id[1].wrapping_add(1);
    
    let init = InitInvariant {
        config: InvariantConfig {
            admin: user_id.into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    add_fee_tier(&api, &mut listener, invariant, fee_tier, InvariantError::NotAdmin.into()).await;

    Ok(())
}

#[tokio::test]
async fn test_add_fee_tier_zero_fee() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier::new(Percentage::new(0), 10).unwrap();
    add_fee_tier(&api, &mut listener, invariant, fee_tier, None).await;

    Ok(())
}

#[tokio::test]
async fn test_add_fee_tier_tick_spacing_zero() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(2, 4),
        tick_spacing: 0,
    };
    add_fee_tier(&api, &mut listener, invariant, fee_tier, InvariantError::InvalidTickSpacing.into()).await;

    Ok(())
}


#[tokio::test]
async fn test_add_fee_tier_over_upper_bound_tick_spacing() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier {
        fee: Percentage::from_scale(2, 4),
        tick_spacing: 101,
    };
    add_fee_tier(&api, &mut listener, invariant, fee_tier, InvariantError::InvalidTickSpacing.into()).await;

    Ok(())
}

#[tokio::test]
async fn test_add_fee_tier_fee_above_limit() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&api, &mut listener, init).await;

    let fee_tier = FeeTier {
        fee: Percentage::from_integer(1),
        tick_spacing: 10,
    };

    add_fee_tier(&api, &mut listener, invariant, fee_tier, InvariantError::InvalidFee.into()).await;

    Ok(())
}