use crate::test_helpers::gclient::init_invariant;
use crate::test_helpers::gclient::token::init_tokens;
use crate::test_helpers::gclient::utils::*;

use decimal::*;
use fungible_token_io::*;
use gclient::{EventListener, GearApi};
use io::*;
use math::percentage::Percentage;

pub async fn init_slippage_invariant_and_tokens(
    api: &GearApi,
    listener: &mut EventListener,
) -> (ProgramId, ProgramId, ProgramId) {
    let protocol_fee = Percentage::from_scale(1, 2);

    let init = InitInvariant {
        config: InvariantConfig {
            admin: get_api_user_id(api).into(),
            protocol_fee: protocol_fee.0 as u128,
        },
    };

    let invariant = init_invariant(api, listener, init).await;

    let (token_x, token_y) = init_tokens(api, listener, InitConfig::default()).await;

    (invariant, token_x, token_y)
}
