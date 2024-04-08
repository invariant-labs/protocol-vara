use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gstd::*;
use gtest::*;
use io::*;
use math::{percentage::Percentage, sqrt_price::calculate_sqrt_price};

#[test]
fn test_get_pool() {
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
    assert!(!res.main_failed());
    assert!(res.log().last().unwrap().payload().is_empty());

    let init_sqrt_price = calculate_sqrt_price(0).unwrap();
    let init_tick = 0;

    let block_timestamp = sys.block_timestamp();

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
    let pool = get_pool(&invariant, token_0, token_1, fee_tier).unwrap();

    assert_eq!(
        pool,
        Pool {
            start_timestamp: block_timestamp,
            last_timestamp: block_timestamp,
            sqrt_price: init_sqrt_price,
            current_tick_index: init_tick,
            fee_receiver: ADMIN.into(),
            ..Pool::default()
        }
    )
}
