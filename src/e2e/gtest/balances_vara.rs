use crate::invariant_service::VARA_ADDRESS;
use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;
use contracts::InvariantError;
use decimal::*;
use math::percentage::Percentage;
use math::token_amount::TokenAmount;

use gtest::*;
#[test]
fn test_vara_deposit() {
    let sys = System::new();
    let vara_mint = 1000000000000;
    sys.init_logger();
    sys.mint_to(REGULAR_USER_1, vara_mint);

    let invariant = init_invariant(&sys, Percentage::new(0));
    assert_eq!(
        deposit_vara(&invariant, REGULAR_USER_1, vara_mint, None::<&str>).unwrap(),
        TokenAmount::new(vara_mint.into())
    );

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_1),
        vec![(VARA_ADDRESS, TokenAmount::new(vara_mint.into()))]
    )
}

#[test]
#[should_panic(
    expected = "Insufficient value: user (0x0200000000000000000000000000000000000000000000000000000000000000) tries to send (1000000000001) value, while his balance (1000000000000)"
)]
fn test_vara_deposit_not_enough_value() {
    let sys = System::new();
    let vara_mint = 1000000000000;
    sys.init_logger();
    sys.mint_to(REGULAR_USER_1, vara_mint);

    assert_eq!(sys.balance_of(INVARIANT_ID), 0);
    assert_eq!(sys.balance_of(REGULAR_USER_1), vara_mint);

    let invariant = init_invariant(&sys, Percentage::new(0));
    deposit_vara(&invariant, REGULAR_USER_1, vara_mint + 1, None::<&str>);
}

#[test]
fn test_vara_withdraw() {
    let sys = System::new();
    let vara_mint = 1000000000000;
    sys.init_logger();
    sys.mint_to(REGULAR_USER_1, vara_mint);

    let invariant = init_invariant(&sys, Percentage::new(0));
    deposit_vara(&invariant, REGULAR_USER_1, vara_mint, None::<&str>);

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_1),
        vec![(VARA_ADDRESS, TokenAmount::new(vara_mint.into()))]
    );
    assert_eq!(sys.balance_of(INVARIANT_ID), vara_mint);
    assert_eq!(sys.balance_of(REGULAR_USER_1), 0);

    let value = withdraw_vara(&invariant, REGULAR_USER_1, vara_mint.into(), None::<&str>);
    assert_eq!(value.unwrap(), TokenAmount::new(vara_mint.into()));

    assert_eq!(get_user_balances(&invariant, REGULAR_USER_1), vec![]);
    sys.claim_value_from_mailbox(REGULAR_USER_1);
    assert_eq!(sys.balance_of(INVARIANT_ID), 0);
    assert_eq!(sys.balance_of(REGULAR_USER_1), vara_mint);

    let value = withdraw_vara(&invariant, REGULAR_USER_1, None, None::<&str>).unwrap();
    assert_eq!(value, TokenAmount::new(0.into()));
    sys.claim_value_from_mailbox(REGULAR_USER_1);
    assert_eq!(sys.balance_of(INVARIANT_ID), 0);
    assert_eq!(sys.balance_of(REGULAR_USER_1), vara_mint);
}

#[test]
fn test_vara_withdraw_failure() {
    let sys = System::new();
    let vara_mint = 1000000000000;
    sys.init_logger();

    let invariant = init_invariant(&sys, Percentage::new(0));
    withdraw_vara(
        &invariant,
        REGULAR_USER_1,
        1.into(),
        Some(InvariantError::NoBalanceForTheToken),
    );

    sys.claim_value_from_mailbox(REGULAR_USER_1);

    assert_eq!(sys.balance_of(INVARIANT_ID), 0);
    assert_eq!(sys.balance_of(REGULAR_USER_1), 0);
    assert_eq!(get_user_balances(&invariant, REGULAR_USER_1), vec![]);

    sys.mint_to(REGULAR_USER_1, vara_mint);

    deposit_vara(&invariant, REGULAR_USER_1, vara_mint, None::<&str>);

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_1),
        vec![(VARA_ADDRESS, TokenAmount::new(vara_mint.into()))]
    );
    assert_eq!(sys.balance_of(INVARIANT_ID), vara_mint);
    assert_eq!(sys.balance_of(REGULAR_USER_1), 0);

    withdraw_vara(
        &invariant,
        REGULAR_USER_1,
        (vara_mint + 1).into(),
        Some(InvariantError::FailedToChangeTokenBalance),
    );

    sys.claim_value_from_mailbox(REGULAR_USER_1);
    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_1),
        vec![(VARA_ADDRESS, TokenAmount::new(vara_mint.into()))]
    );
    assert_eq!(sys.balance_of(INVARIANT_ID), vara_mint);
    assert_eq!(sys.balance_of(REGULAR_USER_1), 0);
}

#[test]
fn test_vara_deposit_and_withdraw_with_normal_entrypoint_failures() {
    let sys = System::new();
    let vara_mint = 1000000000000;
    sys.init_logger();
    sys.mint_to(REGULAR_USER_1, vara_mint);

    let invariant = init_invariant(&sys, Percentage::new(0));

    assert_eq!(sys.balance_of(REGULAR_USER_1), vara_mint);
    assert_eq!(sys.balance_of(INVARIANT_ID), 0);

    deposit_single_token(
        &invariant,
        REGULAR_USER_1,
        VARA_ADDRESS,
        vara_mint.into(),
        Some(InvariantError::InvalidVaraDepositAttempt),
    );

    sys.claim_value_from_mailbox(REGULAR_USER_1);
    assert_eq!(sys.balance_of(REGULAR_USER_1), vara_mint);
    assert_eq!(sys.balance_of(INVARIANT_ID), 0);
    assert_eq!(get_user_balances(&invariant, REGULAR_USER_1), vec![]);

    deposit_token_pair(
        &invariant,
        REGULAR_USER_1,
        VARA_ADDRESS,
        vara_mint.into(),
        TOKEN_X_ID,
        vara_mint.into(),
        Some(InvariantError::InvalidVaraDepositAttempt),
    );

    sys.claim_value_from_mailbox(REGULAR_USER_1);
    assert_eq!(sys.balance_of(REGULAR_USER_1), vara_mint);
    assert_eq!(sys.balance_of(INVARIANT_ID), 0);
    assert_eq!(get_user_balances(&invariant, REGULAR_USER_1), vec![]);

    deposit_vara(&invariant, REGULAR_USER_1, vara_mint, None::<&str>);

    sys.claim_value_from_mailbox(REGULAR_USER_1);
    assert_eq!(sys.balance_of(REGULAR_USER_1), 0);
    assert_eq!(sys.balance_of(INVARIANT_ID), vara_mint);

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_1),
        vec![(VARA_ADDRESS, TokenAmount::new(vara_mint.into()))]
    );

    withdraw_single_token(
        &invariant,
        REGULAR_USER_1,
        VARA_ADDRESS,
        U256::from(vara_mint).into(),
        Some(InvariantError::InvalidVaraWithdrawAttempt),
    );

    sys.claim_value_from_mailbox(REGULAR_USER_1);
    assert_eq!(sys.balance_of(REGULAR_USER_1), 0);
    assert_eq!(sys.balance_of(INVARIANT_ID), vara_mint);

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_1),
        vec![(VARA_ADDRESS, TokenAmount::new(vara_mint.into()))]
    );

    withdraw_token_pair(
        &invariant,
        REGULAR_USER_1,
        VARA_ADDRESS,
        U256::from(vara_mint).into(),
        TOKEN_X_ID,
        None,
        Some(InvariantError::InvalidVaraWithdrawAttempt),
    );

    sys.claim_value_from_mailbox(REGULAR_USER_1);
    assert_eq!(sys.balance_of(REGULAR_USER_1), 0);
    assert_eq!(sys.balance_of(INVARIANT_ID), vara_mint);

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_1),
        vec![(VARA_ADDRESS, TokenAmount::new(vara_mint.into()))]
    );

    withdraw_vara(
        &invariant,
        REGULAR_USER_1,
        vara_mint.into(),
        None::<&str>,
    );
    sys.claim_value_from_mailbox(REGULAR_USER_1);
    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_1),
        vec![]
    );
    assert_eq!(sys.balance_of(INVARIANT_ID), 0);
    assert_eq!(sys.balance_of(REGULAR_USER_1), vara_mint);
}
