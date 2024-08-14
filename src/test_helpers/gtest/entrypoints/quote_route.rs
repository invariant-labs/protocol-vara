use super::InvariantResult;
use crate::test_helpers::consts::*;
use crate::test_helpers::gtest::consts::*;
use crate::{send_query, test_helpers::gtest::swap};
use contracts::{pool_key, FeeTier, InvariantError, PoolKey};
use gstd::vec::Vec;
use gtest::*;
use io::*;
use math::{
    liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};
use sails_rs::ActorId;

pub fn quote_route(
    invariant: &Program,
    amount_in: TokenAmount,
    swaps: Vec<SwapHop>,
) -> Result<TokenAmount, InvariantError> {
    send_query!(
        program: invariant,
        user: PROGRAM_OWNER,
        service_name: "Service",
        action: "QuoteRoute",
        payload: (amount_in, swaps),
        response_type: Result<TokenAmount, InvariantError>
    )
}
