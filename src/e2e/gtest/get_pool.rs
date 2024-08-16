use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gtest::*;
use math::{percentage::Percentage, sqrt_price::calculate_sqrt_price};
use sails_rs::ActorId;

#[test]
fn test_get_pool() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage(100));

    let token_0 = ActorId::from([0x01; 32]);
    let token_1 = ActorId::from([0x02; 32]);
    let fee_tier = FeeTier {
        fee: Percentage::new(1),
        tick_spacing: 1,
    };
    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_sqrt_price = calculate_sqrt_price(0).unwrap();
    let init_tick = 0;

    let block_timestamp = sys.block_timestamp();

    let res = create_pool(
        &invariant,
        REGULAR_USER_1,
        token_0,
        token_1,
        fee_tier,
        init_sqrt_price,
        init_tick,
    );
    res.assert_single_event()
        .assert_empty()
        .assert_to(REGULAR_USER_1);

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
