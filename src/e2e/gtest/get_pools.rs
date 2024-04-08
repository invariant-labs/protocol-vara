use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gstd::*;
use gtest::*;
use io::*;
use math::{percentage::Percentage, sqrt_price::calculate_sqrt_price};

#[test]
fn test_get_pools() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, 100);

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);
    let fee_tier = FeeTier {
        fee: Percentage::new(1),
        tick_spacing: 1,
    };
    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
    
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let init_sqrt_price = calculate_sqrt_price(0).unwrap();
    let init_tick = 0;

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0,
            token_1,
            fee_tier,
            init_sqrt_price,
            init_tick,
        },
    );

    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)]));

    let pool_keys = get_pools(&invariant, u8::MAX, 0).unwrap();

    assert_eq!(
        pool_keys,
        vec![PoolKey::new(token_0, token_1, fee_tier).unwrap()]
    )
}
