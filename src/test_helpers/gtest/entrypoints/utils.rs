extern crate std;

use crate::test_helpers::gtest::consts::*;
pub use crate::test_helpers::utils::pools_are_identical_no_timestamp;
use contracts::InvariantError;
use gstd::{codec::decode_from_bytes, codec::Codec, *};
use gtest::{CoreLog, Program, RunResult};
use io::*;
use std::println;

pub type ProgramId = [u8; 32];

pub trait InvariantResult {
    fn emitted_events(&self) -> Vec<TestEvent>;
    #[track_caller]
    #[must_use]
    fn events_eq(&self, expected: Vec<TestEvent>) -> bool;
    #[track_caller]
    fn assert_success(&self);
}
pub trait RevertibleProgram {
    #[track_caller]
    fn send_and_assert_panic<'a>(
        &'a mut self,
        from: u64,
        payload: impl Codec,
        error: impl Into<String>,
    ) -> RunResult;
    #[track_caller]
    fn send_and_assert_error<'a>(
        &'a mut self,
        from: u64,
        payload: impl Codec,
        error: InvariantError,
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
    pub fn empty_invariant_response(destination: u64) -> Self {
        Self::empty_event(INVARIANT_ID, destination)
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

            if returned.payload != expected.payload {
                let decoded_expected =
                    decode_from_bytes::<InvariantEvent>(expected.clone().payload.into());
                if decoded_expected.is_ok() {
                    match decode_from_bytes::<InvariantEvent>(returned.clone().payload.into()) {
                        Err(e) => std::println!(
                            "Error decoding event: {:?}, event data: {:?}",
                            e,
                            returned
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
                        returned.payload
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
        } else if let Some(event) = self.emitted_events().last() {
            if let Ok(inv_event) = decode_from_bytes::<InvariantEvent>(event.payload.clone().into())
            {
                if let InvariantEvent::ActionFailed(error) = inv_event {
                    panic!("Unexpected error: {:?}", error)
                }
            }
        }
    }
}

impl RevertibleProgram for Program<'_> {
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
    #[track_caller]
    fn send_and_assert_error<'a>(
        &'a mut self,
        from: u64,
        payload: impl Codec,
        error: InvariantError,
    ) -> RunResult {
        let res = self.send(from, payload);
        if let Some(event) = res.emitted_events().last() {
            if let Ok(inv_event) = decode_from_bytes::<InvariantEvent>(event.payload.clone().into())
            {
                if let InvariantEvent::ActionFailed(inv_error) = inv_event {
                    assert_eq!(inv_error, error)
                }
            }
        }
        res
    }
}
