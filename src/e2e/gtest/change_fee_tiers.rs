use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gtest::*;
use gstd::*;
use io::*;
use math::percentage::Percentage;

#[test]
fn test_fee_tiers() {
    let sys = System::new();
    sys.init_logger();

    let invariant = init_invariant(&sys, 100);
    let fee_tier = FeeTier::new(Percentage::new(1), 10u16).unwrap();
    let fee_tier_value = FeeTier {
        fee: Percentage::new(1),
        tick_spacing: 10u16,
    };

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));

    let fee_tiers = get_fee_tiers(&invariant);
    assert_eq!(fee_tiers, vec![fee_tier_value]);

    let res = invariant.send(ADMIN, InvariantAction::AddFeeTier(fee_tier));
    assert!(res.events_eq(vec![TestEvent::invariant_response(
        ADMIN,
        InvariantEvent::ActionFailed(InvariantError::FeeTierAlreadyExist)
    )]));

    let fee_tiers = get_fee_tiers(&invariant);
    assert_eq!(fee_tiers, vec![fee_tier_value]);

    let res = invariant.send(ADMIN, InvariantAction::RemoveFeeTier(fee_tier));
    
    assert!(res.events_eq(vec![TestEvent::empty_invariant_response(ADMIN)]));
    
    let fee_tiers = get_fee_tiers(&invariant);
    assert_eq!(fee_tiers, vec![]);
}