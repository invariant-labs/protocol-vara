extern crate alloc;
use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gstd::*;
use gtest::*;
use io::*;
use math::{percentage::Percentage, sqrt_price::calculate_sqrt_price};

#[test]
fn test_add_pool() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, 0);

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);
    let fee_tier = FeeTier {
        fee: Percentage::new(1),
        tick_spacing: 1,
    };
    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let init_sqrt_price = calculate_sqrt_price(0).unwrap();

    let res = invariant.send(
        REGULAR_USER_1,
        InvariantAction::CreatePool {
            token_0,
            token_1,
            fee_tier,
            init_sqrt_price,
            init_tick: 0,
        },
    );
    assert!(
        res.events_eq(vec![TestEvent::empty_invariant_response(REGULAR_USER_1)])        
    );

    let res = invariant.send(
        REGULAR_USER_2,
        InvariantAction::CreatePool {
            token_0,
            token_1,
            fee_tier,
            init_sqrt_price,
            init_tick: 0,
        },
    );

    res.assert_panicked_with(InvariantError::PoolAlreadyExist);
}
