#![no_std]

use gmeta::{In, InOut, Metadata};
use gstd::{prelude::*, ActorId};

pub struct FungibleTokenMetadata;

impl Metadata for FungibleTokenMetadata {
    type Init = In<InitConfig>;
    type Handle = InOut<FTAction, Result<FTEvent, FTError>>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = In<FTStateQuery>;
}

#[derive(Debug, Default, Decode, Encode, Clone, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitConfig {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}
#[derive(Debug, Decode, Encode, Clone, Copy, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTError {
    ZeroAddress,
    NotAllowedToTransfer,
    NotEnoughBalance,
    TxAlreadyExists,
}

#[derive(Debug, Decode, Encode, Clone, Copy, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTAction {
    Transfer {
        tx_id: Option<u64>,
        from: ActorId,
        to: ActorId,
        amount: u128,
    },
    Approve {
        tx_id: Option<u64>,
        to: ActorId,
        amount: u128,
    },
    BalanceOf(ActorId),
    Mint(u128),
    Burn(u128),
    SetFailTransferFlag(bool),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTEvent {
    Transfer {
        from: ActorId,
        to: ActorId,
        amount: u128,
    },
    Approve {
        from: ActorId,
        to: ActorId,
        amount: u128,
    },
    TotalSupply(u128),
    Balance(u128),
}

#[derive(Debug, Clone, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct IoFungibleToken {
    pub name: String,
    pub symbol: String,
    pub total_supply: u128,
    pub balances: Vec<(ActorId, u128)>,
    pub allowances: Vec<(ActorId, Vec<(ActorId, u128)>)>,
    pub decimals: u8,
}
#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTStateQuery {
    // Standard defined queries (keep in the exact same order unless the standard changes!)
    Allowance { spender: ActorId, account: ActorId },
    BalanceOf { account: ActorId },
    Decimals,
    GetTxValidityTime { account: ActorId, tx_id: u64 },
    Name,
    TotalSupply,
    // Non Standard queries
    FullState,
}