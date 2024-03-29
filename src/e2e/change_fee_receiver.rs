use crate::test_helpers::consts::GEAR_PATH;
use crate::test_helpers::gclient::{
    add_fee_tier, change_fee_receiver, create_pool, get_api_user_id, get_new_token, get_pool,
    init_invariant,
};
use contracts::{FeeTier, InvariantError, PoolKey};
use decimal::*;
use gclient::{GearApi, Result};
use gstd::prelude::*;
use io::*;
use math::sqrt_price::calculate_sqrt_price;
use math::types::percentage::Percentage;

#[tokio::test]
async fn test_change_fee_receiver() -> Result<()> {
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
    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
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
        get_pool(&api, invariant, token_0, token_1, fee_tier, None)
            .await
            .unwrap()
            .fee_receiver,
        get_api_user_id(&api).into()
    );

    change_fee_receiver(
        &api,
        &mut listener,
        invariant,
        PoolKey::new(token_0.into(), token_1.into(), fee_tier).unwrap(),
        get_api_user_id(&user_api).into(),
        None,
    )
    .await;

    assert_eq!(
        get_pool(&api, invariant, token_0, token_1, fee_tier, None)
            .await
            .unwrap()
            .fee_receiver,
        get_api_user_id(&user_api).into()
    );
    Ok(())
}

#[tokio::test]
async fn test_change_fee_receiver_not_admin() -> Result<()> {
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
    let fee_tier = FeeTier::new(Percentage::from_scale(2, 4), 1).unwrap();
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
        get_pool(&api, invariant, token_0, token_1, fee_tier, None)
            .await
            .unwrap()
            .fee_receiver,
        get_api_user_id(&api).into()
    );

    change_fee_receiver(
        &user_api,
        &mut listener,
        invariant,
        PoolKey::new(token_0.into(), token_1.into(), fee_tier).unwrap(),
        get_api_user_id(&user_api).into(),
        InvariantError::NotAdmin.into(),
    )
    .await;

    assert_eq!(
        get_pool(&api, invariant, token_0, token_1, fee_tier, None)
            .await
            .unwrap()
            .fee_receiver,
        get_api_user_id(&api).into()
    );
    Ok(())
}
