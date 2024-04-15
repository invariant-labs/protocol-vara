use crate::test_helpers::gtest::*;
use contracts::*;
use decimal::*;
use gstd::{prelude::*, ActorId};
use gtest::*;
use io::*;
use math::{percentage::Percentage, sqrt_price::calculate_sqrt_price};

pub fn init_basic_pool(invariant: &Program<'_>, token_x: &ActorId, token_y: &ActorId) {
    let fee_tier = FeeTier {
        fee: Percentage::from_scale(6, 3),
        tick_spacing: 10,
    };
    invariant
        .send(ADMIN, InvariantAction::AddFeeTier(fee_tier))
        .assert_success();

    let init_tick = 0;
    let init_sqrt_price = calculate_sqrt_price(init_tick).unwrap();

    assert!(!invariant
        .send(
            REGULAR_USER_1,
            InvariantAction::CreatePool {
                token_0: *token_x,
                token_1: *token_y,
                fee_tier,
                init_sqrt_price,
                init_tick,
            }
        )
        .main_failed());
}
