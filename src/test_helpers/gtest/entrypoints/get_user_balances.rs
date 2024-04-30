use contracts::*;
use gstd::{ActorId, Vec};
use gtest::*;

use io::*;
use math::token_amount::TokenAmount;
pub fn get_user_balances(invariant: &Program, user: u64) -> Vec<(ActorId, TokenAmount)> {
    let state: InvariantStateReply = invariant
        .read_state(InvariantStateQuery::GetUserBalances(user.into()))
        .expect("Failed to read state");
    if let InvariantStateReply::UserBalances(balances) = state {
        return balances;
    } else {
        panic!("unexpected state {:?}", state);
    }
}
