use crate::{send_query, test_helpers::gtest::*};
use contracts::{InvariantError, PoolKey};
use gtest::*;
use io::*;
use math::{sqrt_price::SqrtPrice, token_amount::TokenAmount};
use sails_rs::prelude::*;

#[track_caller]
pub fn quote(
    invariant: &Program,
    from: u64,
    pool_key: PoolKey,
    x_to_y: bool,
    amount: TokenAmount,
    by_amount_in: bool,
    sqrt_price_limit: SqrtPrice,
) -> sails_rs::Result<QuoteResult, InvariantError> {
    send_query!(
        program: invariant,
        user: from,
        service_name: "Service",
        action: "Quote",
        payload: (pool_key, x_to_y, amount, by_amount_in, sqrt_price_limit),
        response_type: sails_rs::Result<QuoteResult, InvariantError>
    )
}
