use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gtest::*;
use math::{percentage::Percentage, sqrt_price::calculate_sqrt_price};
use sails_rtl::ActorId;

pub fn init_basic_pool(invariant: &Program<'_>, token_x: &ActorId, token_y: &ActorId) {
    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 10,
    };
    add_fee_tier(&invariant, ADMIN, fee_tier).assert_success();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    let _res = create_pool(
        &invariant,
        REGULAR_USER_1,
        *token_x,
        *token_y,
        fee_tier,
        init_sqrt_price,
        init_tick,
    )
    .assert_single_event()
    .assert_empty()
    .assert_to(REGULAR_USER_1);
}
