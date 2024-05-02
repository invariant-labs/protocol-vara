#![no_std]

use fungible_token_io::*;
use gstd::{
    collections::{HashMap, HashSet},
    errors::Result as GstdResult,
    exec::block_timestamp,
    msg,
    prelude::*,
    ActorId, MessageId,
};

#[cfg(test)]
mod tests;

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
#[derive(Debug, Clone, Default)]
struct Config {
    #[allow(dead_code)]
    pub fail_transfer: bool,
}
type ValidUntil = u64;
type TxId = u64;
const VALIDITY_PERIOD: u64 = 3000 * 10;
#[derive(Debug, Clone, Default)]
struct FungibleToken {
    /// Name of the token.
    name: String,
    /// Symbol of the token.
    symbol: String,
    /// Number of decimal places for the token.
    decimals: u8,
    /// Total supply of the token.
    total_supply: u128,
    /// A map of account addresses to their token balances.
    balances: HashMap<ActorId, u128>,
    /// A map that records how much an authorized spender is allowed to transfer from a user's account
    allowances: HashMap<ActorId, HashMap<ActorId, u128>>,
    /// A map of executed transactions to the time they are valid.
    tx_ids: HashMap<(ActorId, TxId), ValidUntil>,
    /// A map of accounts to their transaction IDs.
    account_to_tx_ids: HashMap<ActorId, HashSet<TxId>>,
    /// Configuration parameters for the fungible token contract.
    #[allow(dead_code)]
    config: Config,
}

static mut FUNGIBLE_TOKEN: Option<FungibleToken> = None;

impl FungibleToken {
    /// Executed on receiving `fungible-token-messages::MintInput`.
    fn mint(&mut self, amount: u128) {
        let source = msg::source();
        self.balances
            .entry(source)
            .and_modify(|balance| *balance += amount)
            .or_insert(amount);
        self.total_supply += amount;
        reply(Ok(FTEvent::Transfer {
            from: ZERO_ID,
            to: source,
            amount,
        }))
        .unwrap();
    }
    /// Executed on receiving `fungible-token-messages::BurnInput`.
    fn burn(&mut self, amount: u128) {
        let source = msg::source();
        if self.balances.get(&source).unwrap_or(&0) < &amount {
            panic!("Amount exceeds account balance");
        }
        self.balances
            .entry(source)
            .and_modify(|balance| *balance -= amount);
        self.total_supply -= amount;

        reply(Ok(FTEvent::Transfer {
            from: source,
            to: ZERO_ID,
            amount,
        }))
        .unwrap();
    }
    /// Executed on receiving `fungible-token-messages::TransferInput` or `fungible-token-messages::TransferFromInput`.
    /// Transfers `amount` tokens from `sender` account to `recipient` account.
    fn transfer(
        &mut self,
        tx_id: Option<u64>,
        from: &ActorId,
        to: &ActorId,
        amount: u128,
    ) -> Result<(), FTError> {
        #[cfg(feature = "test")]
        {
            if self.config.fail_transfer {
                panic!("Test panic")
            }
        }

        if from == &ZERO_ID || to == &ZERO_ID {
            Err(FTError::ZeroAddress)?
        };
        if !self.can_transfer(from, amount) {
            Err(FTError::NotAllowedToTransfer)?
        }
        if self.balances.get(from).unwrap_or(&0) < &amount {
            Err(FTError::NotEnoughBalance)?
        }
        if self.tx_exits(tx_id) {
            Err(FTError::TxAlreadyExists)?
        }

        self.balances
            .entry(*from)
            .and_modify(|balance| *balance -= amount);
        self.balances
            .entry(*to)
            .and_modify(|balance| *balance += amount)
            .or_insert(amount);

        self.insert_tx(tx_id);

        Ok(())
    }

    /// Executed on receiving `fungible-token-messages::ApproveInput`.
    fn approve(&mut self, tx_id: Option<u64>, to: &ActorId, amount: u128) -> Result<(), FTError> {
        if to == &ZERO_ID {
            Err(FTError::ZeroAddress)?
        }
        if self.tx_exits(tx_id) {
            Err(FTError::TxAlreadyExists)?
        }

        let source = msg::source();

        self.allowances
            .entry(source)
            .or_default()
            .insert(*to, amount);

        self.insert_tx(tx_id);

        Ok(())
    }

    fn can_transfer(&mut self, from: &ActorId, amount: u128) -> bool {
        let source = msg::source();
        if from == &source {
            return true;
        }
        if let Some(allowed_amount) = self.allowances.get(from).and_then(|m| m.get(&source)) {
            if allowed_amount >= &amount {
                self.allowances.entry(*from).and_modify(|m| {
                    m.entry(source).and_modify(|a| *a -= amount);
                });
                return true;
            }
        }
        false
    }

    fn tx_exits(&self, tx_id: Option<u64>) -> bool {
        let current_time = block_timestamp();
        let source = msg::source();
        if let Some(tx_id) = tx_id {
            if let Some(valid_until) = self.tx_ids.get(&(source, tx_id)) {
                return current_time < *valid_until;
            }
        }
        false
    }

    fn insert_tx(&mut self, tx_id: Option<u64>) {
        let current_time = block_timestamp();
        let source = msg::source();
        if let Some(tx_id) = tx_id {
            if let Some(tx_ids) = self.account_to_tx_ids.get_mut(&source) {
                tx_ids.insert(tx_id);
            } else {
                let mut new_tx_ids = HashSet::new();
                new_tx_ids.insert(tx_id);
                self.account_to_tx_ids.insert(source, new_tx_ids);
            }

            self.tx_ids
                .insert((source, tx_id), current_time + VALIDITY_PERIOD);
        }
    }
}

fn common_state() -> IoFungibleToken {
    let state = static_mut_state();
    let FungibleToken {
        name,
        symbol,
        total_supply,
        balances,
        allowances,
        decimals,
        tx_ids: _,
        account_to_tx_ids: _,
        config: _,
    } = state.clone();

    let balances = balances.iter().map(|(k, v)| (*k, *v)).collect();
    let allowances = allowances
        .iter()
        .map(|(id, allowance)| (*id, allowance.iter().map(|(k, v)| (*k, *v)).collect()))
        .collect();
    IoFungibleToken {
        name,
        symbol,
        total_supply,
        balances,
        allowances,
        decimals,
    }
}

fn static_mut_state() -> &'static mut FungibleToken {
    unsafe { FUNGIBLE_TOKEN.get_or_insert(Default::default()) }
}

fn static_state() -> FungibleToken {
    unsafe { FUNGIBLE_TOKEN.clone().expect("State uninitialized") }
}

#[no_mangle]
extern "C" fn state() {
    let query = msg::load::<FTStateQuery>().expect("Failed to decode message");
    let state = static_state();

    match query {
        FTStateQuery::Name => {
            msg::reply(FTStateReply::Name(state.name), 0).expect("Unable to reply");
        }
        FTStateQuery::GetTxValidityTime { account, tx_id } => {
            let valid = state.tx_ids.get(&(account, tx_id)).copied();
            msg::reply(FTStateReply::TxValidityTime(valid), 0).expect("Unable to reply");
        }
        FTStateQuery::Decimals => {
            let state = common_state();
            msg::reply(FTStateReply::Decimals(state.decimals), 0).expect("Unable to reply");
        }
        FTStateQuery::TotalSupply => {
            msg::reply(FTStateReply::TotalSupply(state.total_supply), 0).expect("Unable to reply");
        }
        FTStateQuery::BalanceOf { account } => {
            let balance = state.balances.get(&account).copied().unwrap_or(0);
            msg::reply(FTStateReply::BalanceOf(balance), 0).expect("Unable to reply");
        }
        FTStateQuery::Allowance { spender, account } => {
            let allowance = state
                .allowances
                .get(&account)
                .and_then(|m| m.get(&spender))
                .copied()
                .unwrap_or(0);
            msg::reply(FTStateReply::Allowance(allowance), 0).expect("Unable to reply");
        }
        FTStateQuery::FullState => {
            let state = common_state();
            msg::reply(FTStateReply::FullState(state), 0).expect("Unable to reply");
        }
    }
}

fn reply(payload: Result<FTEvent, FTError>) -> GstdResult<MessageId> {
    msg::reply(payload, 0)
}

#[no_mangle]
extern "C" fn handle() {
    let action: FTAction = msg::load().expect("Could not load Action");
    let ft: &mut FungibleToken = unsafe { FUNGIBLE_TOKEN.get_or_insert(Default::default()) };
    match action {
        FTAction::Transfer {
            tx_id,
            from,
            to,
            amount,
        } => match ft.transfer(tx_id, &from, &to, amount) {
            Ok(_) => {
                reply(Ok(FTEvent::Transfer {
                    from: from,
                    to: to,
                    amount,
                }))
                .expect("Unable to reply");
            }
            Err(e) => {
                reply(Err(e)).expect("Unable to reply");
            }
        },
        FTAction::Approve { tx_id, to, amount } => match ft.approve(tx_id, &to, amount) {
            Ok(_) => {
                reply(Ok(FTEvent::Approve {
                    from: msg::source(),
                    to,
                    amount,
                }))
                .expect("Unable to reply");
            }
            Err(e) => {
                reply(Err(e)).expect("Unable to reply");
            }
        },
        FTAction::Mint(amount) => {
            ft.mint(amount);
        }
        FTAction::Burn(amount) => {
            ft.burn(amount);
        }
        FTAction::BalanceOf(account) => {
            let balance = ft.balances.get(&account).unwrap_or(&0);
            reply(Ok(FTEvent::Balance(*balance))).unwrap();
        }
        #[allow(unused_variables)]
        FTAction::SetFailTransferFlag(fail) => {
            #[cfg(feature = "test")]
            {
                ft.config.fail_transfer = fail;
            }
        }
    }
}

#[no_mangle]
extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    let ft = FungibleToken {
        name: config.name,
        symbol: config.symbol,
        decimals: config.decimals,
        ..Default::default()
    };
    unsafe { FUNGIBLE_TOKEN = Some(ft) };
}
