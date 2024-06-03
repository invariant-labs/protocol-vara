use gear_erc20::services::admin::utils::Role;
use gclient::{EventProcessor, GearApi, Result};
use gstd::{ActorId, Encode};

mod utils_gclient;
use sails_rtl::U256;
use utils_gclient::*;
const GEAR_PATH: &str = "../../target/tmp/gear";

#[tokio::test]
async fn test_basic_functionality() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await?;
    // let api = GearApi::dev().await?;
    let mut listener = api.subscribe().await?;
    assert!(listener.blocks_running().await?);

    // init
    let (message_id, program_id) = init(&api).await;
    assert!(listener.message_processed(message_id).await?.succeed());

    // grant role minter
    let user: ActorId = api.get_actor_id();
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Admin", action: "GrantRole", payload: (user, Role::Minter));
    assert!(listener.message_processed(message_id).await?.succeed());

    // mint
    let value: U256 = 1_000.into();
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Admin", action: "Mint", payload: (user, value));
    assert!(listener.message_processed(message_id).await?.succeed());

    // check state
    let state = get_state_balances(&api, program_id, &mut listener, 0, 1).await;
    assert_eq!(state, vec![(user.into(), value)]);

    // grant role burner
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Admin", action: "GrantRole", payload: (user, Role::Burner));
    assert!(listener.message_processed(message_id).await?.succeed());

    // burn
    let value: U256 = 500.into();
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Admin", action: "Burn", payload: (user, value));
    assert!(listener.message_processed(message_id).await?.succeed());

    // check state
    let state = get_state_balances(&api, program_id, &mut listener, 0, 1).await;
    assert_eq!(state, vec![(user.into(), value)]);

    // transfer
    let to = api.get_specific_actor_id(USERS_STR[0]);
    let value: U256 = 250.into();
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Erc20", action: "Transfer", payload: (to, value));
    assert!(listener.message_processed(message_id).await?.succeed());

    // check state
    let state = get_state_balances(&api, program_id, &mut listener, 1, 1).await;
    assert_eq!(state, vec![(to.into(), value)]);

    // approve
    let spender = api.get_specific_actor_id(USERS_STR[0]);
    let value: U256 = 250.into();
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Erc20", action: "Approve", payload: (spender, value));
    assert!(listener.message_processed(message_id).await?.succeed());

    // transfer from
    let from = user;
    let to = api.get_specific_actor_id(USERS_STR[1]);
    let new_api = get_new_client(&api, USERS_STR[0]).await;
    let value: U256 = 250.into();
    let message_id = send_request!(api: &new_api, program_id: program_id, service_name: "Erc20", action: "TransferFrom", payload: (from, to, value));
    assert!(listener.message_processed(message_id).await?.succeed());

    // check state
    let state = get_state_balances(&api, program_id, &mut listener, 0, 3).await;
    let user_1 = api.get_actor_id();
    let user_2 = api.get_specific_actor_id(USERS_STR[0]);
    let user_3 = api.get_specific_actor_id(USERS_STR[1]);
    assert!(!state.contains(&(user_1.into(), 0.into())));
    assert!(state.contains(&(user_2.into(), 250.into())));
    assert!(state.contains(&(user_3.into(), 250.into())));

    Ok(())
}

#[tokio::test]
async fn test_roles() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await?;

    let mut listener = api.subscribe().await?;
    assert!(listener.blocks_running().await?);

    // init
    let (message_id, program_id) = init(&api).await;
    assert!(listener.message_processed(message_id).await?.succeed());

    // check state
    let user: ActorId = api.get_actor_id();
    let role = "FungibleAdmin".to_string();
    let has_role = get_state_has_role(&api, program_id, &mut listener, user, role).await;
    assert!(has_role);

    // grant role minter
    let to = api.get_specific_actor_id(USERS_STR[0]);
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Admin", action: "GrantRole", payload: (to, Role::Minter));
    assert!(listener.message_processed(message_id).await?.succeed());

    // check state
    let role = "FungibleMinter".to_string();
    let has_role = get_state_has_role(&api, program_id, &mut listener, to, role.clone()).await;
    assert!(has_role);

    // remove role minter
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Admin", action: "RemoveRole", payload: (to, Role::Minter));
    assert!(listener.message_processed(message_id).await?.succeed());

    // check state
    let has_role = get_state_has_role(&api, program_id, &mut listener, to, role).await;
    assert!(!has_role);

    // grant role burner
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Admin", action: "GrantRole", payload: (to, Role::Burner));
    assert!(listener.message_processed(message_id).await?.succeed());

    // check state
    let role = "FungibleBurner".to_string();
    let has_role = get_state_has_role(&api, program_id, &mut listener, to, role.clone()).await;
    assert!(has_role);

    // remove role burner
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Admin", action: "RemoveRole", payload: (to, Role::Burner));
    assert!(listener.message_processed(message_id).await?.succeed());

    // check state
    let has_role = get_state_has_role(&api, program_id, &mut listener, to, role).await;
    assert!(!has_role);

    Ok(())
}

#[tokio::test]
async fn test_pausable() -> Result<()> {
    let api = GearApi::dev_from_path(GEAR_PATH).await?;

    let mut listener = api.subscribe().await?;
    assert!(listener.blocks_running().await?);

    // init
    let (message_id, program_id) = init(&api).await;
    assert!(listener.message_processed(message_id).await?.succeed());

    // check state
    let paused = get_state_is_paused(&api, program_id, &mut listener).await;
    assert!(!paused);

    // pause
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Pausable", action: "Pause", payload: ());
    assert!(listener.message_processed(message_id).await?.succeed());

    // check state
    let paused = get_state_is_paused(&api, program_id, &mut listener).await;
    assert!(paused);

    // unpause
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Pausable", action: "Unpause", payload: ());
    assert!(listener.message_processed(message_id).await?.succeed());

    // check state
    let paused = get_state_is_paused(&api, program_id, &mut listener).await;
    assert!(!paused);

    Ok(())
}
