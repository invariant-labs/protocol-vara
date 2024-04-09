use crate::test_helpers::gclient::entrypoints::{
    add_fee_tier, create_pool, create_position, get_pool,
};
use crate::test_helpers::gclient::token::{approve, mint};
use crate::test_helpers::gclient::utils::*;
use contracts::{FeeTier, PoolKey};
use decimal::*;
use gclient::{EventListener, GearApi};
use math::{liquidity::Liquidity, percentage::Percentage, sqrt_price::calculate_sqrt_price};
#[allow(dead_code)]
pub async fn init_slippage_pool_with_liquidity(
    admin_api: &GearApi,
    user_api: &GearApi,
    listener: &mut EventListener,
    invariant: ProgramId,
    token_0: ProgramId,
    token_1: ProgramId,
) -> PoolKey {
    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 10,
    };

    add_fee_tier(admin_api, listener, invariant, fee_tier, None).await;

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    create_pool(
        user_api,
        listener,
        invariant,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
        None,
    )
    .await;

    let mint_amount = 10u128.pow(10);

    mint(&user_api, listener, token_0, mint_amount)
        .await
        .unwrap();
    mint(&user_api, listener, token_1, mint_amount)
        .await
        .unwrap();

    approve(&user_api, listener, token_0, invariant, mint_amount)
        .await
        .unwrap();
    approve(&user_api, listener, token_1, invariant, mint_amount)
        .await
        .unwrap();

    let pool_key = PoolKey::new(token_0.into(), token_1.into(), fee_tier).unwrap();
    let lower_tick = -1000;
    let upper_tick = 1000;
    let liquidity = Liquidity::from_integer(10u128.pow(10));

    let pool_before = get_pool(user_api, invariant, token_0, token_1, fee_tier, None)
        .await
        .unwrap();

    let slippage_limit_lower = pool_before.sqrt_price;
    let slippage_limit_upper = pool_before.sqrt_price;

    create_position(
        user_api,
        listener,
        invariant,
        pool_key,
        lower_tick,
        upper_tick,
        liquidity,
        slippage_limit_lower,
        slippage_limit_upper,
        None,
    )
    .await;

    let pool_after = get_pool(user_api, invariant, token_0, token_1, fee_tier, None)
        .await
        .unwrap();

    assert_eq!(pool_after.liquidity, liquidity);

    pool_key
}
