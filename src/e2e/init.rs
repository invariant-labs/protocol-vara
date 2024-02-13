use gclient::{EventProcessor, GearApi, Result};
use gstd::{vec, ActorId};
use io::*;
use scale::Encode;

const PATH: &str = "./target/wasm32-unknown-unknown/release/invariant.opt.wasm";
const USER: [u8; 32] = [0; 32];

#[tokio::test]
async fn test_init() -> Result<()> {
    let api = GearApi::dev().await.unwrap();

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: ActorId::new(USER),
            protocol_fee: 100,
        },
    };

    let init_payload = init.encode();

    let gas_info = api
        .calculate_upload_gas(
            None,
            gclient::code_from_os(PATH)?,
            init_payload.clone(),
            0,
            true,
        )
        .await?;

    let (message_id, _program_id, _hash) = api
        .upload_program_bytes_by_path(
            PATH,
            gclient::now_micros().to_le_bytes(),
            init_payload,
            gas_info.min_limit,
            0,
        )
        .await?;

    assert!(listener.message_processed(message_id).await?.succeed());

    Ok(())
}
