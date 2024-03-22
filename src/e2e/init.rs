use gclient::{GearApi, Result};
use gstd::ActorId;
use io::*;
use crate::test_helpers::gclient::init::init_invariant;
use crate::test_helpers::consts::GEAR_PATH;

const USER: [u8; 32] = [0; 32];

#[tokio::test]
async fn test_init() -> Result<()> {
    
    let api = GearApi::dev_from_path(GEAR_PATH).await.unwrap();

    let mut listener = api.subscribe().await?;

    assert!(listener.blocks_running().await?);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: ActorId::new(USER),
            protocol_fee: 100,
        },
    };

    let _invariant = init_invariant(&api, &mut listener, init).await;
    Ok(())
}
