use super::init_token::init_token;
use crate::test_helpers::{consts::TOKEN_PATH, gclient::utils::ProgramId};
use fungible_token_io::InitConfig;
use gclient::{EventListener, EventProcessor, GearApi};
use gstd::prelude::*;

pub async fn init_tokens(
    api: &GearApi,
    listener: &mut EventListener,
    init: InitConfig,
) -> (ProgramId, ProgramId) {
    let (token_x, token_y) = (
        init_token(api, listener, init.clone()).await,
        init_token(api, listener, init).await,
    );
    if token_x < token_y {
        return (token_x, token_y);
    } else {
        return (token_y, token_x);
    };
}
