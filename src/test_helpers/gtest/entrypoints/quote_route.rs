use crate::send_query;
use crate::test_helpers::gtest::*;
use contracts::{pool_key, FeeTier, InvariantError, PoolKey};
use gtest::*;
use io::*;
use math::{
    liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};
use sails_rs::prelude::*;

pub fn quote_route(
    invariant: &Program,
    amount_in: TokenAmount,
    swaps: Vec<SwapHop>,
) -> sails_rs::Result<TokenAmount, InvariantError> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "QuoteRoute",
        payload: (amount_in, swaps),
        response_type: Result<TokenAmount, InvariantError>
    )
}
