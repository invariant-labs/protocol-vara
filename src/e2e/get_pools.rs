use crate::test_helpers::consts::GEAR_PATH;
use crate::test_helpers::gclient::{
    add_fee_tier, create_pool, get_api_user_id, get_new_token, get_pool, get_pools, init_invariant,
};
use contracts::{FeeTier, PoolKey};
use decimal::*;
use gclient::{GearApi, Result};
use gstd::prelude::*;
use io::*;
use math::{percentage::Percentage, sqrt_price::calculate_sqrt_price};

#[tokio::test]
async fn test_get_pools() -> Result<()> {
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

    get_pool(&api, invariant, token_0, token_1, fee_tier, None).await.unwrap();

    assert_eq!(
        get_pools(&api, invariant, u8::MAX, 0, None).await.unwrap(),
        vec![PoolKey::new(token_0.into(),token_1.into(), fee_tier).unwrap()]
    );
    Ok(())
}
