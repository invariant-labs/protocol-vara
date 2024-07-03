use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::gtest::*;

use contracts::*;
use decimal::*;
use gtest::*;
use math::{percentage::Percentage, token_amount::TokenAmount};
use sails_rtl::ActorId;

#[test]
fn test_single_deposit_and_withdraw() {
    let sys = System::new();
    sys.init_logger();
    let token: ActorId = TOKEN_X_ID.into();

    let (token_program, _) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    let amount = U256::from(500);

    mint(&token_program, REGULAR_USER_2, amount).assert_success();
    increase_allowance(&token_program, REGULAR_USER_2, INVARIANT_ID, amount * 3).assert_success();

    // deposit and withdraw with value
    assert_eq!(
        deposit_single_token(&invariant, REGULAR_USER_2, token, amount, None::<&str>).unwrap(),
        TokenAmount(amount)
    );

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_2),
        vec![(token, TokenAmount(amount))]
    );

    assert_eq!(
        withdraw_single_token(
            &invariant,
            REGULAR_USER_2,
            token,
            amount.into(),
            None::<&str>
        )
        .unwrap(),
        TokenAmount(amount)
    );

    assert_eq!(get_user_balances(&invariant, REGULAR_USER_2), vec![]);

    // deposit and withdraw with None
    assert_eq!(
        deposit_single_token(&invariant, REGULAR_USER_2, token, amount, None::<&str>).unwrap(),
        TokenAmount(amount)
    );

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_2),
        vec![(token, TokenAmount(amount))]
    );

    assert_eq!(
        withdraw_single_token(&invariant, REGULAR_USER_2, token, None, None::<&str>).unwrap(),
        TokenAmount(amount)
    );

    assert_eq!(get_user_balances(&invariant, REGULAR_USER_2), vec![]);

    // deposit and withdraw with multiple requests
    assert_eq!(
        deposit_single_token(&invariant, REGULAR_USER_2, token, U256::from(250), None::<&str>).unwrap(),
        TokenAmount::new(U256::from(250))
    );

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_2),
        vec![(token, TokenAmount::new(U256::from(250)))]
    );

    assert_eq!(
        deposit_single_token(&invariant, REGULAR_USER_2, token, U256::from(250), None::<&str>).unwrap(),
        TokenAmount::new(U256::from(250))
    );

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_2),
        vec![(token, TokenAmount::new(U256::from(500)))]
    );

    assert_eq!(
        withdraw_single_token(&invariant, REGULAR_USER_2, token, U256::from(250).into(), None::<&str>).unwrap(),
        TokenAmount::new(U256::from(250))
    );

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_2),
        vec![(token, TokenAmount::new(U256::from(250)))]
    );

    assert_eq!(
        withdraw_single_token(&invariant, REGULAR_USER_2, token, U256::from(250).into(), None::<&str>).unwrap(),
        TokenAmount::new(U256::from(250))
    );
}

#[test]
fn test_single_deposit_failures() {
    let sys = System::new();
    sys.init_logger();
    let token: ActorId = TOKEN_X_ID.into();

    let (token_program, _) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    let amount = U256::one();

    let deposit_and_verify = |err: InvariantError| {
        let invariant_before = balance_of(&token_program, INVARIANT_ID);
        let user_before = balance_of(&token_program, REGULAR_USER_2);

        let balances_before = get_user_balances(&invariant, REGULAR_USER_2);

        assert_eq!(
            deposit_single_token(&invariant, REGULAR_USER_2, token, amount, err.into()),
            None
        );

        assert_eq!(
            get_user_balances(&invariant, REGULAR_USER_2),
            balances_before
        );

        let invariant = balance_of(&token_program, INVARIANT_ID);
        let user = balance_of(&token_program, REGULAR_USER_2);

        assert_eq!(invariant, invariant_before);
        assert_eq!(user, user_before);
    };

    // balance too low
    deposit_and_verify(InvariantError::UnrecoverableTransferError);
    mint(&token_program, REGULAR_USER_2, amount).assert_success();

    // allowance too low
    deposit_and_verify(InvariantError::UnrecoverableTransferError);

    burn(
        &token_program,
        REGULAR_USER_2,
        balance_of(&token_program, REGULAR_USER_2),
    )
    .assert_success();

    let max_deposit = U256::MAX;

    mint(&token_program, REGULAR_USER_2, max_deposit).assert_success();
    set_allowance(&token_program, REGULAR_USER_2, INVARIANT_ID, max_deposit).assert_success();
    deposit_single_token(&invariant, REGULAR_USER_2, token, max_deposit, None::<&str>).unwrap();

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_2),
        vec![(token, TokenAmount(max_deposit)),]
    );

    deposit_and_verify(InvariantError::FailedToChangeTokenBalance);
}

#[test]
fn test_single_withdraw_failures() {
    let sys = System::new();
    sys.init_logger();
    let token: ActorId = TOKEN_X_ID.into();

    let (token_program, _) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    let amount = U256::from(500);

    mint(&token_program, REGULAR_USER_2, amount).assert_success();
    increase_allowance(&token_program, REGULAR_USER_2, INVARIANT_ID, amount).assert_success();

    let withdraw_fails = |amount: Option<U256>, err: InvariantError| {
        let invariant_before = balance_of(&token_program, INVARIANT_ID);
        let user_before = balance_of(&token_program, REGULAR_USER_2);

        let balances_before = get_user_balances(&invariant, REGULAR_USER_2);

        let invariant_after = balance_of(&token_program, INVARIANT_ID);
        let user = balance_of(&token_program, REGULAR_USER_2);
        assert_eq!(invariant_after, invariant_before);
        assert_eq!(user, user_before);

        assert_eq!(
            withdraw_single_token(
                &invariant,
                REGULAR_USER_2,
                token,
                amount,
                err.clone().into()
            ),
            None
        );

        let invariant_after = balance_of(&token_program, INVARIANT_ID);
        let user = balance_of(&token_program, REGULAR_USER_2);
        assert_eq!(invariant_after, invariant_before);
        assert_eq!(user, user_before);

        assert_eq!(
            get_user_balances(&invariant, REGULAR_USER_2),
            balances_before
        );
    };

    // no balance
    withdraw_fails(amount.into(), InvariantError::NoBalanceForTheToken.into());
    withdraw_fails(None, InvariantError::NoBalanceForTheToken.into());

    assert_eq!(
        deposit_single_token(&invariant, REGULAR_USER_2, token, amount, None::<&str>).unwrap(),
        TokenAmount(amount)
    );

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_2),
        vec![(token, TokenAmount(amount))]
    );

    // balance insufficient
    withdraw_fails(
        (amount + 1).into(),
        InvariantError::FailedToChangeTokenBalance.into(),
    );

    // transfer failure
    set_transfer_fail(&token_program, true).assert_success();

    withdraw_fails(
        amount.into(),
        InvariantError::RecoverableTransferError.into(),
    );
}

#[test]
fn test_token_pair_deposit_and_withdraw() {
    let sys = System::new();
    sys.init_logger();
    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (token_x_program, token_y_program) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    let amount_x = U256::from(500);
    let amount_y = U256::from(500);

    mint(&token_x_program, REGULAR_USER_2, amount_x).assert_success();
    increase_allowance(&token_x_program, REGULAR_USER_2, INVARIANT_ID, amount_x * 3)
        .assert_success();

    mint(&token_y_program, REGULAR_USER_2, amount_y).assert_success();
    increase_allowance(&token_y_program, REGULAR_USER_2, INVARIANT_ID, amount_y * 3)
        .assert_success();

    let check_balances_after_withdraw = || {
        assert_eq!(get_user_balances(&invariant, REGULAR_USER_2), vec![]);

        let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y = balance_of(&token_y_program, INVARIANT_ID);
        let user_x = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y = balance_of(&token_y_program, REGULAR_USER_2);

        assert_eq!(invariant_x, U256::zero());
        assert_eq!(invariant_y, U256::zero());
        assert_eq!(user_x, amount_x);
        assert_eq!(user_y, amount_y);
    };

    let check_balances_after_deposit = || {
        assert_eq!(
            get_user_balances(&invariant, REGULAR_USER_2),
            vec![
                (token_x, TokenAmount(amount_x)),
                (token_y, TokenAmount(amount_y)),
            ]
        );

        let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y = balance_of(&token_y_program, INVARIANT_ID);
        let user_x = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y = balance_of(&token_y_program, REGULAR_USER_2);

        assert_eq!(invariant_x, amount_x);
        assert_eq!(invariant_y, amount_y);
        assert_eq!(user_x, U256::zero());
        assert_eq!(user_y, U256::zero());
    };

    // deposit and withdraw with value
    assert_eq!(
        deposit_token_pair(
            &invariant,
            REGULAR_USER_2,
            token_x,
            amount_x,
            token_y,
            amount_y,
            None::<&str>
        )
        .unwrap(),
        (TokenAmount(amount_x), TokenAmount(amount_y))
    );

    check_balances_after_deposit();

    assert_eq!(
        withdraw_token_pair(
            &invariant,
            REGULAR_USER_2,
            token_x,
            amount_x.into(),
            token_y,
            amount_y.into(),
            None::<&str>
        )
        .unwrap(),
        (TokenAmount(amount_x), TokenAmount(amount_y))
    );
    check_balances_after_withdraw();

    // deposit and withdraw with None
    assert_eq!(
        deposit_token_pair(
            &invariant,
            REGULAR_USER_2,
            token_x,
            amount_x,
            token_y,
            amount_y,
            None::<&str>
        )
        .unwrap(),
        (TokenAmount(amount_x), TokenAmount(amount_y))
    );

    check_balances_after_deposit();

    assert_eq!(
        withdraw_token_pair(
            &invariant,
            REGULAR_USER_2,
            token_x,
            None,
            token_y,
            None,
            None::<&str>
        )
        .unwrap(),
        (TokenAmount(amount_x), TokenAmount(amount_y))
    );

    check_balances_after_withdraw();

    deposit_single_token(&invariant, REGULAR_USER_2, token_x, amount_x, None::<&str>).unwrap();

    assert_eq!(
        withdraw_token_pair(
            &invariant,
            REGULAR_USER_2,
            token_x,
            None,
            token_y,
            None,
            None::<&str>
        )
        .unwrap(),
        (TokenAmount(amount_x), TokenAmount::new(U256::from(0)))
    );

    deposit_single_token(&invariant, REGULAR_USER_2, token_y, amount_y, None::<&str>).unwrap();

    assert_eq!(
        withdraw_token_pair(
            &invariant,
            REGULAR_USER_2,
            token_x,
            None,
            token_y,
            None,
            None::<&str>
        )
        .unwrap(),
        (TokenAmount::new(U256::from(0)), TokenAmount(amount_y))
    );
}

#[test]
fn test_token_pair_deposit_failures() {
    let sys = System::new();
    sys.init_logger();
    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (token_x_program, token_y_program) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    let amount_x = U256::from(500);
    let amount_y = U256::from(500);

    let clear = |fail_x: bool, fail_y: bool| {
        set_transfer_fail(&token_x_program, false).assert_success();
        set_transfer_fail(&token_y_program, false).assert_success();

        withdraw_token_pair(
            &invariant,
            REGULAR_USER_2,
            token_x,
            None,
            token_y,
            None,
            None::<&str>,
        )
        .unwrap();

        assert_eq!(get_user_balances(&invariant, REGULAR_USER_2), vec![]);

        set_transfer_fail(&token_x_program, fail_x).assert_success();
        set_transfer_fail(&token_y_program, fail_y).assert_success();
    };
    let deposit_both_fails = |deposited_amount_x, deposited_amount_y, err: InvariantError| {
        let balances_before = get_user_balances(&invariant, REGULAR_USER_2);

        let invariant_x_before = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y_before = balance_of(&token_y_program, INVARIANT_ID);
        let user_x_before = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y_before = balance_of(&token_y_program, REGULAR_USER_2);

        assert_eq!(
            deposit_token_pair(
                &invariant,
                REGULAR_USER_2,
                token_x,
                deposited_amount_x,
                token_y,
                deposited_amount_y,
                err.into()
            ),
            None
        );

        assert_eq!(
            balances_before,
            get_user_balances(&invariant, REGULAR_USER_2)
        );

        let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y = balance_of(&token_y_program, INVARIANT_ID);
        let user_x = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y = balance_of(&token_y_program, REGULAR_USER_2);

        assert_eq!(invariant_x, invariant_x_before);
        assert_eq!(invariant_y, invariant_y_before);
        assert_eq!(user_x, user_x_before);
        assert_eq!(user_y, user_y_before);
    };

    let deposit_x_fails = |fail_x: bool, fail_y: bool| {
        clear(fail_x, fail_y);

        let invariant_x_before = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y_before = balance_of(&token_y_program, INVARIANT_ID);
        let user_x_before = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y_before = balance_of(&token_y_program, REGULAR_USER_2);

        assert_eq!(
            deposit_token_pair(
                &invariant,
                REGULAR_USER_2,
                token_x,
                amount_x,
                token_y,
                amount_y,
                InvariantError::RecoverableTransferError.into()
            ),
            None
        );

        assert_eq!(
            get_user_balances(&invariant, REGULAR_USER_2),
            vec![(token_y, TokenAmount(amount_y)),]
        );

        let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y = balance_of(&token_y_program, INVARIANT_ID);
        let user_x = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y = balance_of(&token_y_program, REGULAR_USER_2);

        assert_eq!(invariant_x, invariant_x_before);
        assert_eq!(invariant_y, invariant_y_before + amount_y);
        assert_eq!(user_x, user_x_before);
        assert_eq!(user_y, user_y_before - amount_y);
    };

    let deposit_y_fails = |fail_x: bool, fail_y: bool| {
        clear(fail_x, fail_y);

        let invariant_x_before = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y_before = balance_of(&token_y_program, INVARIANT_ID);
        let user_x_before = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y_before = balance_of(&token_y_program, REGULAR_USER_2);

        assert_eq!(
            deposit_token_pair(
                &invariant,
                REGULAR_USER_2,
                token_x,
                amount_x,
                token_y,
                amount_y,
                InvariantError::RecoverableTransferError.into()
            ),
            None
        );

        assert_eq!(
            get_user_balances(&invariant, REGULAR_USER_2),
            vec![(token_x, TokenAmount(amount_x)),]
        );

        let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y = balance_of(&token_y_program, INVARIANT_ID);
        let user_x = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y = balance_of(&token_y_program, REGULAR_USER_2);

        assert_eq!(invariant_x, invariant_x_before + amount_x);
        assert_eq!(invariant_y, invariant_y_before);
        assert_eq!(user_x, user_x_before - amount_x);
        assert_eq!(user_y, user_y_before);
    };

    let max_deposit = U256::MAX;

    let set_deposit_max = |token: ActorId| {
        let (token, token_program) = if token == token_x {
            (token_x, &token_x_program)
        } else if token == token_y {
            (token_y, &token_y_program)
        } else {
            panic!("unexpected token");
        };

        let mut balances_before = get_user_balances(&invariant, REGULAR_USER_2);
        if balances_before.iter().find(|(t, _)| t == &token).is_some() {
            withdraw_single_token(&invariant, REGULAR_USER_2, token, None, None::<&str>).unwrap();
        }

        burn(
            &token_program,
            REGULAR_USER_2,
            balance_of(&token_program, REGULAR_USER_2),
        )
        .assert_success();
    
        mint(&token_program, REGULAR_USER_2, max_deposit).assert_success();
        set_allowance(&token_program, REGULAR_USER_2, INVARIANT_ID, max_deposit).assert_success();

        deposit_single_token(&invariant, REGULAR_USER_2, token, max_deposit, None::<&str>).unwrap();

        balances_before.push((token, TokenAmount(max_deposit)));

        assert_eq!(
            get_user_balances(&invariant, REGULAR_USER_2),
            balances_before
        );
    };

    // balance too low
    deposit_both_fails(
        amount_x,
        amount_y,
        InvariantError::UnrecoverableTransferError,
    );

    mint(&token_x_program, REGULAR_USER_2, amount_x).assert_success();

    // allowance too low
    deposit_both_fails(
        amount_x,
        amount_y,
        InvariantError::UnrecoverableTransferError,
    );

    increase_allowance(
        &token_x_program,
        REGULAR_USER_2,
        INVARIANT_ID,
        amount_x * 30,
    )
    .assert_success();
    // y balance too low
    deposit_y_fails(false, false);

    mint(&token_y_program, REGULAR_USER_2, amount_y).assert_success();

    // y allowance too low
    deposit_y_fails(false, false);

    increase_allowance(
        &token_y_program,
        REGULAR_USER_2,
        INVARIANT_ID,
        amount_y  * 30,
    )
    .assert_success();

    // x fails set
    deposit_x_fails(true, false);
    // y fails set
    deposit_y_fails(false, true);
    // both fail set
    clear(true, true);
    deposit_both_fails(
        amount_x,
        amount_y,
        InvariantError::UnrecoverableTransferError,
    );
    // fail overflow
    clear(false, false);

    set_deposit_max(token_x);

    // fail overflow x
    deposit_both_fails(U256::one(), U256::zero(), InvariantError::FailedToChangeTokenBalance);

    set_deposit_max(token_y);
    // fail overflow both
    deposit_both_fails(U256::one(), U256::zero(), InvariantError::FailedToChangeTokenBalance);
    deposit_both_fails(U256::zero(), U256::one(), InvariantError::FailedToChangeTokenBalance);
    deposit_both_fails(U256::one(), U256::one(), InvariantError::FailedToChangeTokenBalance);

    // fail overflow y
    deposit_both_fails(U256::zero(), U256::one(), InvariantError::FailedToChangeTokenBalance);

    let invariant_x_before = balance_of(&token_x_program, INVARIANT_ID);
    let invariant_y_before = balance_of(&token_y_program, INVARIANT_ID);
    let user_x_before = balance_of(&token_x_program, REGULAR_USER_2);
    let user_y_before = balance_of(&token_y_program, REGULAR_USER_2);

    let balances_before = get_user_balances(&invariant, REGULAR_USER_2);

    assert_eq!(
        deposit_token_pair(
            &invariant,
            REGULAR_USER_2,
            token_x,
            U256::one(),
            token_x,
            U256::one(),
            InvariantError::TokensAreSame.into()
        ),
        None
    );

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_2),
        balances_before
    );

    let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID);
    let user_x = balance_of(&token_x_program, REGULAR_USER_2);
    let user_y = balance_of(&token_y_program, REGULAR_USER_2);

    assert_eq!(invariant_x, invariant_x_before);
    assert_eq!(invariant_y, invariant_y_before);
    assert_eq!(user_x, user_x_before);
    assert_eq!(user_y, user_y_before);
}

#[test]
fn test_token_pair_withdraw_failures() {
    let sys = System::new();
    sys.init_logger();
    let token_x: ActorId = TOKEN_X_ID.into();
    let token_y: ActorId = TOKEN_Y_ID.into();

    let (token_x_program, token_y_program) = init_tokens(&sys);
    let invariant = init_invariant(&sys, Percentage::from_scale(1, 2));

    let amount_x = U256::from(500);
    let amount_y = U256::from(500);

    let clear = |fail_x: bool, fail_y: bool| {
        set_transfer_fail(&token_x_program, false).assert_success();
        set_transfer_fail(&token_y_program, false).assert_success();

        withdraw_token_pair(
            &invariant,
            REGULAR_USER_2,
            token_x,
            None,
            token_y,
            None,
            None::<&str>,
        )
        .unwrap();

        deposit_token_pair(
            &invariant,
            REGULAR_USER_2,
            token_x,
            amount_x,
            token_y,
            amount_y,
            None::<&str>,
        )
        .unwrap();

        assert_eq!(
            get_user_balances(&invariant, REGULAR_USER_2),
            vec![
                (token_x, TokenAmount(amount_x)),
                (token_y, TokenAmount(amount_y))
            ]
        );

        assert_eq!(
            (
                balance_of(&token_x_program, INVARIANT_ID),
                balance_of(&token_y_program, INVARIANT_ID)
            ),
            (amount_x, amount_y)
        );
        set_transfer_fail(&token_x_program, fail_x).assert_success();
        set_transfer_fail(&token_y_program, fail_y).assert_success();
    };

    let withdraw_both_fails = |withdrawn_amount_x: Option<U256>,
                               withdrawn_amount_y: Option<U256>,
                               err: InvariantError| {
        let invariant_x_before = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y_before = balance_of(&token_y_program, INVARIANT_ID);
        let user_x_before = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y_before = balance_of(&token_y_program, REGULAR_USER_2);

        let balances_before = get_user_balances(&invariant, REGULAR_USER_2);

        assert_eq!(
            withdraw_token_pair(
                &invariant,
                REGULAR_USER_2,
                token_x,
                withdrawn_amount_x,
                token_y,
                withdrawn_amount_y,
                err.into()
            ),
            None
        );

        assert_eq!(
            get_user_balances(&invariant, REGULAR_USER_2),
            balances_before
        );

        let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y = balance_of(&token_y_program, INVARIANT_ID);
        let user_x = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y = balance_of(&token_y_program, REGULAR_USER_2);

        assert_eq!(invariant_x, invariant_x_before);
        assert_eq!(invariant_y, invariant_y_before);
        assert_eq!(user_x, user_x_before);
        assert_eq!(user_y, user_y_before);
    };

    let withdraw_x_fails = |withdrawn_amount_x: Option<U256>,
                            withdrawn_amount_y: Option<U256>,
                            err: InvariantError| {
        let invariant_x_before = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y_before = balance_of(&token_y_program, INVARIANT_ID);
        let user_x_before = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y_before = balance_of(&token_y_program, REGULAR_USER_2);

        let mut balances_before = get_user_balances(&invariant, REGULAR_USER_2);

        assert_eq!(
            withdraw_token_pair(
                &invariant,
                REGULAR_USER_2,
                token_x,
                withdrawn_amount_x,
                token_y,
                withdrawn_amount_y,
                err.into()
            ),
            None
        );

        let amount_y = balances_before
            .iter()
            .enumerate()
            .find(|(_, (t, _))| t == &token_y)
            .and_then(|(i, (_, v))| Some((i, v.get())));

        let withdrawn_amount_y =
            withdrawn_amount_y.unwrap_or(amount_y.and_then(|(_, v)| Some(v)).unwrap_or(U256::from(0)));

        if let Some((i, _)) = amount_y {
            balances_before.remove(i);
        };

        assert_eq!(
            balances_before,
            get_user_balances(&invariant, REGULAR_USER_2)
        );

        let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y = balance_of(&token_y_program, INVARIANT_ID);
        let user_x = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y = balance_of(&token_y_program, REGULAR_USER_2);

        assert_eq!(invariant_x, invariant_x_before);
        assert_eq!(invariant_y, invariant_y_before - withdrawn_amount_y);
        assert_eq!(user_x, user_x_before);
        assert_eq!(user_y, user_y_before + withdrawn_amount_y);
    };

    let withdraw_y_fails = |withdrawn_amount_x: Option<U256>,
                            withdrawn_amount_y: Option<U256>,
                            err: InvariantError| {
        let invariant_x_before = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y_before = balance_of(&token_y_program, INVARIANT_ID);
        let user_x_before = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y_before = balance_of(&token_y_program, REGULAR_USER_2);

        let mut balances_before = get_user_balances(&invariant, REGULAR_USER_2);

        assert_eq!(
            withdraw_token_pair(
                &invariant,
                REGULAR_USER_2,
                token_x,
                withdrawn_amount_x,
                token_y,
                withdrawn_amount_y,
                err.into()
            ),
            None
        );

        let amount_x = balances_before
            .iter()
            .enumerate()
            .find(|(_, (t, _))| t == &token_x)
            .and_then(|(i, (_, v))| Some((i, v.get())));

        let withdrawn_amount_x =
            withdrawn_amount_x.unwrap_or(amount_x.and_then(|(_, v)| Some(v)).unwrap_or(U256::from(0)));

        if let Some((i, _)) = amount_x {
            balances_before.remove(i);
        };

        assert_eq!(
            balances_before,
            get_user_balances(&invariant, REGULAR_USER_2)
        );

        let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
        let invariant_y = balance_of(&token_y_program, INVARIANT_ID);
        let user_x = balance_of(&token_x_program, REGULAR_USER_2);
        let user_y = balance_of(&token_y_program, REGULAR_USER_2);

        assert_eq!(invariant_x, invariant_x_before - withdrawn_amount_x);
        assert_eq!(invariant_y, invariant_y_before);
        assert_eq!(user_x, user_x_before + withdrawn_amount_x);
        assert_eq!(user_y, user_y_before);
    };

    // fail both empty
    withdraw_both_fails(U256::one().into(), None, InvariantError::NoBalanceForTheToken);
    withdraw_both_fails(None, U256::one().into(), InvariantError::NoBalanceForTheToken);
    withdraw_both_fails(U256::one().into(), U256::one().into(), InvariantError::NoBalanceForTheToken);

    mint(&token_x_program, REGULAR_USER_2, amount_x).assert_success();
    increase_allowance(
        &token_x_program,
        REGULAR_USER_2,
        INVARIANT_ID,
        amount_x * 30,
    )
    .assert_success();
    mint(&token_y_program, REGULAR_USER_2, amount_y).assert_success();
    increase_allowance(
        &token_y_program,
        REGULAR_USER_2,
        INVARIANT_ID,
        amount_y * 30,
    )
    .assert_success();

    // fail set y
    clear(false, true);
    withdraw_y_fails(None, None, InvariantError::RecoverableTransferError);
    clear(false, true);
    withdraw_y_fails(
        amount_x.into(),
        None,
        InvariantError::RecoverableTransferError,
    );
    clear(false, true);
    withdraw_y_fails(None, U256::one().into(), InvariantError::RecoverableTransferError);
    clear(false, true);
    withdraw_y_fails(
        amount_x.into(),
        U256::one().into(),
        InvariantError::RecoverableTransferError,
    );

    // fail set x
    clear(true, false);
    withdraw_x_fails(None, None, InvariantError::RecoverableTransferError);
    clear(true, false);
    withdraw_x_fails(U256::one().into(), None, InvariantError::RecoverableTransferError);
    clear(true, false);
    withdraw_x_fails(
        None,
        amount_y.into(),
        InvariantError::RecoverableTransferError,
    );
    clear(true, false);
    withdraw_x_fails(
        U256::one().into(),
        amount_y.into(),
        InvariantError::RecoverableTransferError,
    );

    // both fail set
    clear(true, true);
    withdraw_both_fails(None, None, InvariantError::RecoverableTransferError);
    clear(true, true);
    withdraw_both_fails(U256::one().into(), None, InvariantError::RecoverableTransferError);
    clear(true, true);
    withdraw_both_fails(None, U256::one().into(), InvariantError::RecoverableTransferError);
    clear(true, true);
    withdraw_both_fails(U256::one().into(), U256::one().into(), InvariantError::RecoverableTransferError);

    // underflow
    clear(false, false);
    withdraw_both_fails(
        (amount_x + U256::one()).into(),
        None,
        InvariantError::FailedToChangeTokenBalance,
    );
    withdraw_both_fails(
        None,
        (amount_y + U256::one()).into(),
        InvariantError::FailedToChangeTokenBalance,
    );
    withdraw_both_fails(
        (amount_x + U256::one()).into(),
        (amount_y + U256::one()).into(),
        InvariantError::FailedToChangeTokenBalance,
    );

    // same tokens
    let invariant_x_before = balance_of(&token_x_program, INVARIANT_ID);
    let invariant_y_before = balance_of(&token_y_program, INVARIANT_ID);
    let user_x_before = balance_of(&token_x_program, REGULAR_USER_2);
    let user_y_before = balance_of(&token_y_program, REGULAR_USER_2);

    let balances_before = get_user_balances(&invariant, REGULAR_USER_2);

    assert_eq!(
        withdraw_token_pair(
            &invariant,
            REGULAR_USER_2,
            token_x,
            U256::one().into(),
            token_x,
            U256::one().into(),
            InvariantError::TokensAreSame.into()
        ),
        None
    );

    assert_eq!(
        get_user_balances(&invariant, REGULAR_USER_2),
        balances_before
    );

    let invariant_x = balance_of(&token_x_program, INVARIANT_ID);
    let invariant_y = balance_of(&token_y_program, INVARIANT_ID);
    let user_x = balance_of(&token_x_program, REGULAR_USER_2);
    let user_y = balance_of(&token_y_program, REGULAR_USER_2);

    assert_eq!(invariant_x, invariant_x_before);
    assert_eq!(invariant_y, invariant_y_before);
    assert_eq!(user_x, user_x_before);
    assert_eq!(user_y, user_y_before);
}
