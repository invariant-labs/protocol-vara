use contracts::FeeTier;
use gclient::{GearApi, Result};
use gstd::{ActorId, Vec};
use io::*;
use crate::test_helpers::gclient::{get_fee_tiers, get_protocol_fee};
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

    let invariant = init_invariant(&api, &mut listener, init).await;
    assert_eq!(Vec::<FeeTier>::new(), get_fee_tiers(&api, invariant).await);
    assert_eq!(100, get_protocol_fee(&api, invariant).await);

    Ok(())
}
