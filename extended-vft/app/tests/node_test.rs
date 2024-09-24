use gclient::{EventProcessor, GearApi, Result};
use sails_rs::{ActorId, Decode, Encode, U256};
mod utils_gclient;
use utils_gclient::*;

#[tokio::test]
#[ignore]
async fn test_basic_function() -> Result<()> {
    let api = GearApi::dev_from_path("../target/tmp/gear").await?;
    let john_api = get_new_client(&api, USERS_STR[0]).await;

    let mut listener = api.subscribe().await?;
    assert!(listener.blocks_running().await?);

    // Init
    let (message_id, program_id) = init(&api).await;
    assert!(listener.message_processed(message_id).await?.succeed());
    // Mint
    let actor = api.get_actor_id();
    let value: U256 = 1_000.into();
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Vft", action: "Mint", payload: (actor, value));
    assert!(listener.message_processed(message_id).await?.succeed());
    // Check Balance
    let balance_value = get_state!(api: &api, listener: listener, program_id: program_id, service_name: "Vft", action: "BalanceOf", return_type: U256, payload: (actor));
    assert_eq!(balance_value, value);

    // Burn
    let burn_value: U256 = 100.into();
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Vft", action: "Burn", payload: (actor, burn_value));
    assert!(listener.message_processed(message_id).await?.succeed());
    // Check Balance
    let balance_value = get_state!(api: &api, listener: listener, program_id: program_id, service_name: "Vft", action: "BalanceOf", return_type: U256, payload: (actor));
    assert_eq!(balance_value, value - burn_value);

    // Transfer
    let transfer_value: U256 = 100.into();
    let john_actor_id = john_api.get_actor_id();
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Vft", action: "Transfer", payload: (john_actor_id, burn_value));
    assert!(listener.message_processed(message_id).await?.succeed());
    // Check Balance
    let balance_value = get_state!(api: &api, listener: listener, program_id: program_id, service_name: "Vft", action: "BalanceOf", return_type: U256, payload: (actor));

    assert_eq!(balance_value, value - burn_value - transfer_value);
    let balance_value = get_state!(api: &api, listener: listener, program_id: program_id, service_name: "Vft", action: "BalanceOf", return_type: U256, payload: (john_actor_id));

    assert_eq!(balance_value, transfer_value);

    // Approve
    let approve_value: U256 = 100.into();
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Vft", action: "Approve", payload: (john_actor_id, approve_value));
    assert!(listener.message_processed(message_id).await?.succeed());
    // TransferFrom
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Vft", action: "TransferFrom", payload: (actor, john_actor_id, approve_value));
    assert!(listener.message_processed(message_id).await?.succeed());
    // Check Balance
    let balance_value = get_state!(api: &api, listener: listener, program_id: program_id, service_name: "Vft", action: "BalanceOf", return_type: U256, payload: (john_actor_id));
    assert_eq!(balance_value, transfer_value + approve_value);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_grant_role() -> Result<()> {
    let api = GearApi::dev_from_path("../target/tmp/gear").await?;
    let john_api = get_new_client(&api, USERS_STR[0]).await;

    let mut listener = api.subscribe().await?;
    assert!(listener.blocks_running().await?);

    // Init
    let (message_id, program_id) = init(&api).await;
    assert!(listener.message_processed(message_id).await?.succeed());

    // Grant Admin Role
    let john_actor_id = john_api.get_actor_id();
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Vft", action: "GrantAdminRole", payload: (john_actor_id));
    assert!(listener.message_processed(message_id).await?.succeed());
    // Check State
    let admins: Vec<ActorId> = get_state!(api: &api, listener: listener, program_id: program_id, service_name: "Vft", action: "Admins", return_type: Vec<ActorId>, payload: ());
    assert_eq!(admins, vec![api.get_actor_id(), john_actor_id]);

    // Grant Minter Role
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Vft", action: "GrantMinterRole", payload: (john_actor_id));
    assert!(listener.message_processed(message_id).await?.succeed());
    // Check State
    let minters: Vec<ActorId> = get_state!(api: &api, listener: listener, program_id: program_id, service_name: "Vft", action: "Minters", return_type: Vec<ActorId>, payload: ());
    assert_eq!(minters, vec![api.get_actor_id(), john_actor_id]);

    // Grant Burner Role
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Vft", action: "GrantBurnerRole", payload: (john_actor_id));
    assert!(listener.message_processed(message_id).await?.succeed());
    // Check State
    let burners: Vec<ActorId> = get_state!(api: &api, listener: listener, program_id: program_id, service_name: "Vft", action: "Burners", return_type: Vec<ActorId>, payload: ());
    assert_eq!(burners, vec![api.get_actor_id(), john_actor_id]);

    // John Mint
    let value: U256 = 1_000.into();
    let message_id = send_request!(api: &john_api, program_id: program_id, service_name: "Vft", action: "Mint", payload: (john_actor_id, value));
    assert!(listener.message_processed(message_id).await?.succeed());
    // Check Balance
    let balance_value = get_state!(api: &john_api, listener: listener, program_id: program_id, service_name: "Vft", action: "BalanceOf", return_type: U256, payload: (john_actor_id));
    assert_eq!(balance_value, value);

    // John Burn
    let burn_value: U256 = 100.into();
    let message_id = send_request!(api: &john_api, program_id: program_id, service_name: "Vft", action: "Burn", payload: (john_actor_id, burn_value));
    assert!(listener.message_processed(message_id).await?.succeed());
    // Check Balance
    let balance_value = get_state!(api: &john_api, listener: listener, program_id: program_id, service_name: "Vft", action: "BalanceOf", return_type: U256, payload: (john_actor_id));
    assert_eq!(balance_value, value - burn_value);

    // Revoke Minter Role
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Vft", action: "RevokeMinterRole", payload: (john_actor_id));
    assert!(listener.message_processed(message_id).await?.succeed());
    // Check State
    let minters: Vec<ActorId> = get_state!(api: &api, listener: listener, program_id: program_id, service_name: "Vft", action: "Minters", return_type: Vec<ActorId>, payload: ());
    assert_eq!(minters, vec![api.get_actor_id()]);

    // Revoke Burner Role
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Vft", action: "RevokeBurnerRole", payload: (john_actor_id));
    assert!(listener.message_processed(message_id).await?.succeed());
    // Check State
    let burners: Vec<ActorId> = get_state!(api: &api, listener: listener, program_id: program_id, service_name: "Vft", action: "Burners", return_type: Vec<ActorId>, payload: ());
    assert_eq!(burners, vec![api.get_actor_id()]);

    // Revoke Admin Role
    let message_id = send_request!(api: &api, program_id: program_id, service_name: "Vft", action: "RevokeAdminRole", payload: (john_actor_id));
    assert!(listener.message_processed(message_id).await?.succeed());
    // Check State
    let admins: Vec<ActorId> = get_state!(api: &api, listener: listener, program_id: program_id, service_name: "Vft", action: "Admins", return_type: Vec<ActorId>, payload: ());
    assert_eq!(admins, vec![api.get_actor_id()]);

    Ok(())
}
