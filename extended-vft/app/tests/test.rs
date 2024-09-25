use extended_vft_wasm::{
    traits::{ExtendedVftFactory, Vft},
    ExtendedVftFactory as Factory, Vft as VftClient,
};
use sails_rs::calls::*;
use sails_rs::gtest::calls::*;

#[tokio::test]
async fn test_basic_function() {
    let program_space = GTestRemoting::new(100.into());
    program_space.system().init_logger();
    let code_id = program_space
        .system()
        .submit_code_file("../../target/wasm32-unknown-unknown/release/extended_vft_wasm.opt.wasm");
    
    let extended_vft_factory = Factory::new(program_space.clone());
    let extended_vft_id = extended_vft_factory
        .new("name".to_string(), "symbol".to_string(), 10)
        .send_recv(code_id, "123")
        .await
        .unwrap();

    let mut client = VftClient::new(program_space);
    // mint
    client
        .mint(100.into(), 1_000.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(100.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 1_000.into());

    // burn
    client
        .burn(100.into(), 100.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(100.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 900.into());

    // transfer
    client
        .transfer(101.into(), 100.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(100.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 800.into());
    let balance = client
        .balance_of(101.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 100.into());

    // approve
    client
        .approve(102.into(), 100.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(100.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 800.into());
    let balance = client
        .balance_of(102.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 0.into());
    // transfer from
    client
        .transfer_from(100.into(), 101.into(), 100.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(100.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 700.into());
    let balance = client
        .balance_of(101.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 200.into());
    let balance = client
        .balance_of(102.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 0.into());
}

#[tokio::test]
async fn test_grant_role() {
    let program_space = GTestRemoting::new(100.into());
    program_space.system().init_logger();
    let mut client = VftClient::new(program_space.clone());

    let code_id = program_space
        .system()
        .submit_code_file("../../target/wasm32-unknown-unknown/release/extended_vft_wasm.opt.wasm");

    let extended_vft_factory = Factory::new(program_space.clone());
    let extended_vft_id = extended_vft_factory
        .new("name".to_string(), "symbol".to_string(), 10)
        .send_recv(code_id, "123")
        .await
        .unwrap();

    // try minter role
    let res = client
        .mint(101.into(), 1_000.into())
        .with_args(GTestArgs::new(101.into()))
        .send_recv(extended_vft_id)
        .await;
    assert!(res.is_err());
    // grant mint role
    client
        .grant_minter_role(101.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let minters = client.minters().recv(extended_vft_id).await.unwrap();
    assert_eq!(minters, vec![100.into(), 101.into()]);
    let res = client
        .mint(101.into(), 1_000.into())
        .with_args(GTestArgs::new(101.into()))
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    assert!(res);
    let balance = client
        .balance_of(101.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 1_000.into());

    // try burner role
    let res = client
        .burn(101.into(), 1_000.into())
        .with_args(GTestArgs::new(101.into()))
        .send_recv(extended_vft_id)
        .await;
    assert!(res.is_err());
    // grant burner role
    client
        .grant_burner_role(101.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let burners = client.burners().recv(extended_vft_id).await.unwrap();
    assert_eq!(burners, vec![100.into(), 101.into()]);
    let res = client
        .burn(101.into(), 1_000.into())
        .with_args(GTestArgs::new(101.into()))
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    assert!(res);
    let balance = client
        .balance_of(101.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 0.into());

    // grant admin role
    client
        .grant_admin_role(101.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let admins = client.admins().recv(extended_vft_id).await.unwrap();
    assert_eq!(admins, vec![100.into(), 101.into()]);

    // revoke roles
    client
        .revoke_admin_role(101.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let admins = client.admins().recv(extended_vft_id).await.unwrap();
    assert_eq!(admins, vec![100.into()]);
    client
        .revoke_minter_role(101.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let minters = client.minters().recv(extended_vft_id).await.unwrap();
    assert_eq!(minters, vec![100.into()]);
    client
        .revoke_burner_role(101.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let burners = client.burners().recv(extended_vft_id).await.unwrap();
    assert_eq!(burners, vec![100.into()]);
}
