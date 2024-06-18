extern crate std;

use crate::test_helpers::gtest::consts::*;
pub use crate::test_helpers::utils::pools_are_identical_no_timestamp;
use contracts::*;
use gstd::{codec::decode_from_bytes, codec::Codec, fmt::Debug, *};
use gtest::{CoreLog, Program, RunResult};
use io::*;
use math::{
    liquidity::Liquidity, percentage::Percentage, sqrt_price::SqrtPrice, token_amount::TokenAmount,
};
use sails_rtl::hex::decode;
use std::println;

pub type ProgramId = [u8; 32];

pub trait InvariantResult {
    fn emitted_events(&self) -> Vec<TestEvent>;
    #[track_caller]
    #[must_use]
    fn events_eq(&self, expected: Vec<TestEvent>) -> bool;
    #[track_caller]
    fn assert_success(&self);
    #[track_caller]
    fn assert_single_event(&self) -> TestEvent {
        self.assert_success();
        assert_eq!(self.emitted_events().len(), 1);
        self.emitted_events().last().unwrap().clone()
    }
    #[track_caller]
    fn last_event(&self) -> TestEvent {
        self.emitted_events().last().unwrap().clone()
    }
}
pub trait TestProgram {
    #[track_caller]
    fn send_and_assert_panic<'a>(
        &'a mut self,
        from: u64,
        payload: impl Codec,
        error: impl Into<String>,
    ) -> RunResult;
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct TestEvent {
    pub payload: Vec<u8>,
    pub source: ProgramId,
    pub destination: ProgramId,
}

impl TestEvent {
    #[allow(dead_code)]
    pub fn decoded_event<T>(&self) -> Result<T, codec::Error>
    where
        T: Decode,
    {
        decode_from_bytes(self.payload.clone().into())
    }
    pub fn empty_event(source: u64, destination: u64) -> Self {
        Self {
            payload: vec![],
            source: ActorId::from(source).into(),
            destination: ActorId::from(destination).into(),
        }
    }
    pub fn invariant_response(destination: u64, payload: impl Encode) -> Self {
        Self {
            payload: payload.encode(),
            source: ActorId::from(INVARIANT_ID).into(),
            destination: ActorId::from(destination).into(),
        }
    }
    #[track_caller]
    pub fn assert_empty(&self) -> &Self {
        self.assert_with_payload(())
    }
    #[track_caller]
    pub fn assert_to(&self, destination: u64) -> &Self {
        assert_eq!(ActorId::from(self.destination), ActorId::from(destination));
        self
    }
    #[track_caller]
    pub fn assert_with_payload<T: Encode + Decode + Debug + PartialEq>(&self, payload: T) -> &Self {
        assert_eq!(
            <(String, String, T)>::decode(&mut self.payload.as_slice())
                .unwrap()
                .2,
            payload
        );
        self
    }
}

impl From<&CoreLog> for TestEvent {
    fn from(log: &CoreLog) -> Self {
        Self {
            payload: log.payload().to_vec(),
            source: log.source().into(),
            destination: log.destination().into(),
        }
    }
}

impl InvariantResult for RunResult {
    fn emitted_events(&self) -> Vec<TestEvent> {
        self.log()
            .iter()
            .filter(|e| {
                e.source() == INVARIANT_ID.into()
                    && ![TOKEN_X_ID.into(), TOKEN_Y_ID.into()].contains(&e.destination())
            })
            .map(TestEvent::from)
            .collect()
    }

    #[track_caller]
    #[must_use]
    fn events_eq(&self, expected_events: Vec<TestEvent>) -> bool {
        self.assert_success();

        let events = self.emitted_events();

        if events == expected_events {
            return true;
        }

        if events.len() != expected_events.len() {
            std::println!(
                "mismatched lengths: {} != {}",
                events.len(),
                expected_events.len()
            );
            return false;
        }

        let zipped_events = events.iter().zip(expected_events.iter());
        for (returned, expected) in zipped_events {
            if returned.source != expected.source {
                std::println!(
                    "mismatched sources: {:?} != {:?}",
                    returned.source,
                    expected.source
                );
                return false;
            }

            if returned.destination != expected.destination {
                std::println!(
                    "mismatched destinations: {:?} != {:?}",
                    returned.destination,
                    expected.destination
                );
                return false;
            }

            let prefix = <(String, String)>::decode(&mut returned.payload.as_slice()).unwrap();
            let returned_payload = &returned.payload[(prefix.encode().len() - 1)..].to_vec();
            if returned_payload != &expected.payload {
                let decoded_expected =
                    decode_from_bytes::<InvariantEvent>(expected.clone().payload.into());
                if decoded_expected.is_ok() {
                    match decode_from_bytes::<InvariantEvent>(returned_payload.clone().into()) {
                        Err(e) => std::println!(
                            "Error decoding event: {:?}, event data: {:?}",
                            e,
                            returned_payload
                        ),
                        Ok(decoded) => {
                            let decoded_expected = decoded_expected.unwrap();
                            if decoded != decoded_expected {
                                println!("expected {:?}\n, got {:?}", decoded_expected, decoded);
                                return false;
                            }
                        }
                    }

                    return false;
                } else {
                    std::println!(
                        "mismatched payloads: expected {:?}\n got {:?}",
                        expected.payload,
                        returned_payload
                    );
                }
            }
        }

        false
    }
    #[track_caller]
    fn assert_success(&self) {
        if self.main_failed() {
            self.assert_panicked_with(
                "message used to get the actual error message in case of an unexpected panic",
            );
        }
    }
}

impl TestProgram for Program<'_> {
    #[track_caller]
    fn send_and_assert_panic<'a>(
        &'a mut self,
        from: u64,
        payload: impl Codec,
        error: impl Into<String>,
    ) -> RunResult {
        let res = self.send(from, payload);
        res.assert_panicked_with(error.into());
        res
    }
}

// Event structs are necessary since the enum id is dropped when encoding
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct PositionCreatedEvent {
    pub block_timestamp: u64,
    pub address: ActorId,
    pub pool_key: PoolKey,
    pub liquidity_delta: Liquidity,
    pub lower_tick: i32,
    pub upper_tick: i32,
    pub current_sqrt_price: SqrtPrice,
}
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct PositionRemovedEvent {
    pub block_timestamp: u64,
    pub caller: ActorId,
    pub pool_key: PoolKey,
    pub liquidity: Liquidity,
    pub lower_tick_index: i32,
    pub upper_tick_index: i32,
    pub sqrt_price: SqrtPrice,
}
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct CrossTickEvent {
    pub timestamp: u64,
    pub address: ActorId,
    pub pool: PoolKey,
    pub indexes: Vec<i32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct SwapEvent {
    pub timestamp: u64,
    pub address: ActorId,
    pub pool: PoolKey,
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub fee: TokenAmount,
    pub start_sqrt_price: SqrtPrice,
    pub target_sqrt_price: SqrtPrice,
    pub x_to_y: bool,
}
