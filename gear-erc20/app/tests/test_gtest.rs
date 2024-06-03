use gear_erc20::services::admin::utils::Role;
use gstd::{ActorId, Encode};
use gtest::System;
use sails_rtl::U256;

mod utils_gtest;
use utils_gtest::*;

#[test]
fn gtest_test_roles() {
    let sys = System::new();
    sys.init_logger();

    let ft = init(&sys);

    let admin_user: ActorId = USERS[0].into();
    let user: ActorId = USERS[1].into();

    // failed mint
    let value: U256 = 1_000.into();
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Mint", payload: (admin_user, value));
    assert!(res.main_failed());

    // failed grant role
    let res = send_request!(ft: ft, user: USERS[1], service_name: "Admin", action: "GrantRole", payload: (user, Role::Minter));
    assert!(res.main_failed());

    // success grant role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "GrantRole", payload: (admin_user, Role::Minter));
    assert!(!res.main_failed());

    // success mint
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Mint", payload: (admin_user, value));
    assert!(!res.main_failed());

    // remove role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "RemoveRole", payload: (admin_user, Role::Minter));
    assert!(!res.main_failed());

    // failed mint
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Burn", payload: (admin_user, value));
    assert!(res.main_failed());

    // failed burn
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Burn", payload: (admin_user, value));
    assert!(res.main_failed());

    // grant role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "GrantRole", payload: (admin_user, Role::Burner));
    assert!(!res.main_failed());

    // success burn
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Burn", payload: (admin_user, value / 2));
    assert!(!res.main_failed());

    // remove role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "RemoveRole", payload: (admin_user, Role::Burner));
    assert!(!res.main_failed());

    // failed burn
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Burn", payload: (admin_user, value));
    assert!(res.main_failed());

    // failed pause
    let res = send_request!(ft: ft, user: USERS[1], service_name: "Pausable", action: "Pause", payload: ());
    assert!(res.main_failed());

    // delegate admin
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Pausable", action: "DelegateAdmin", payload: (user));
    assert!(!res.main_failed());

    // success pause
    let res = send_request!(ft: ft, user: USERS[1], service_name: "Pausable", action: "Pause", payload: ());
    assert!(!res.main_failed());

    // failed kill
    let res = send_request!(ft: ft, user: USERS[1], service_name: "Admin", action: "Kill", payload: (user));
    assert!(res.main_failed());

    // success kill
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Kill", payload: (user));
    assert!(!res.main_failed());
}

#[test]
fn gtest_test_pausable() {
    let sys = System::new();
    sys.init_logger();

    let ft = init(&sys);
    let admin_user: ActorId = USERS[0].into();

    // success grant role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "GrantRole", payload: (admin_user, Role::Minter));
    assert!(!res.main_failed());

    // success grant role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "GrantRole", payload: (admin_user, Role::Burner));
    assert!(!res.main_failed());

    // success pause
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Pausable", action: "Pause", payload: ());
    assert!(!res.main_failed());

    // failed grant role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "GrantRole", payload: (admin_user, Role::Minter));
    assert!(res.main_failed());

    // failed mint
    let value: U256 = 1_000.into();
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Mint", payload: (admin_user, value));
    assert!(res.main_failed());

    // failed burn
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Burn", payload: (admin_user, value));
    assert!(res.main_failed());
}

//#[cfg(feature = "test")]
#[test]
fn gtest_test_transfer_fail() {
    let sys = System::new();
    sys.init_logger();

    let ft = init(&sys);
    let admin_user: ActorId = USERS[0].into();
    let user: ActorId = USERS[1].into();

    let value = 100;
    // success grant role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "GrantRole", payload: (admin_user, Role::Minter));
    assert!(!res.main_failed());

    // success mint
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Mint", payload: (admin_user, U256::from(value)));
    assert!(!res.main_failed());

    let res = send_request!(ft: ft, user: USERS[0], service_name: "Erc20", action: "SetFailTransfer", payload: (true));
    assert!(!res.main_failed());

    let res = send_request!(ft: ft, user: USERS[0], service_name: "Erc20", action: "Transfer", payload: (user, U256::from(value)));
    res.assert_panicked_with("Manually forced panic");
    
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Erc20", action: "SetFailTransfer", payload: (false));
    assert!(!res.main_failed());

    let res = send_request!(ft: ft, user: USERS[0], service_name: "Erc20", action: "Transfer", payload: (user, U256::from(value)));
    assert!(!res.main_failed());
}