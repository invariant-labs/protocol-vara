use crate::test_helpers::consts::GEAR_PATH;
use crate::test_helpers::gclient::{
    add_fee_tier, fee_tier_exists, get_api_user_id, init_invariant, remove_fee_tier,
};
use contracts::{FeeTier, InvariantError};
use decimal::*;
use gclient::{GearApi, Result};
use gstd::prelude::*;
use io::*;
use math::types::percentage::Percentage;

#[tokio::test]
async fn test_remove_fee_tier() -> Result<()> {
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

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    add_fee_tier(&api, &mut listener, invariant, fee_tier, None).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 2).unwrap();
    add_fee_tier(&api, &mut listener, invariant, fee_tier, None).await;

    remove_fee_tier(&api, &mut listener, invariant, fee_tier, None).await;

    assert!(!fee_tier_exists(&api, invariant, fee_tier).await);
    Ok(())
}

#[tokio::test]
async fn test_remove_not_existing_fee_tier() -> Result<()> {
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

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    add_fee_tier(&api, &mut listener, invariant, fee_tier, None).await;

    remove_fee_tier(
        &api,
        &mut listener,
        invariant,
        FeeTier::new(Percentage::from_scale(2, 4), 2).unwrap(),
        InvariantError::FeeTierNotFound.into(),
    )
    .await;

    Ok(())
}

#[tokio::test]
async fn test_remove_fee_tier_not_admin() -> Result<()> {
    let admin_api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();
    
    let mut listener = admin_api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(&admin_api).into(),
            protocol_fee: 100,
        },
    };

    let invariant = init_invariant(&admin_api, &mut listener, init).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
    add_fee_tier(&admin_api, &mut listener, invariant, fee_tier, None).await;

    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 2).unwrap();
    add_fee_tier(&admin_api, &mut listener, invariant, fee_tier, None).await;

    let user_api = admin_api.with("//Bob").unwrap();

    remove_fee_tier(
        &user_api,
        &mut listener,
        invariant,
        fee_tier,
        InvariantError::NotAdmin.into(),
    )
    .await;

    Ok(())
}
