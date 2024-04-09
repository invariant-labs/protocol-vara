extern crate std;

use crate::test_helpers::gtest::consts::*;
use contracts::InvariantError;
use gstd::{codec::Codec, codec::decode_from_bytes, *};
use gtest::{CoreLog, Program, RunResult};
use io::*;
use std::println;
use core::sync::atomic::{AtomicU64, Ordering};
pub use crate::test_helpers::utils::pools_are_identical_no_timestamp;

pub type ProgramId = [u8; 32];
static FILE_BACKUP_NR: AtomicU64 = AtomicU64::new(0);

pub trait InvariantResult {
    fn emitted_events(&self) -> Vec<TestEvent>;
    #[track_caller]
    #[must_use]
    fn events_eq(&self, expected: Vec<TestEvent>) -> bool;
}
pub trait RevertibleProgram {
    #[track_caller]
    fn send_and_assert_panic<'a>(&'a mut self, from: u64, payload: impl Codec, error: InvariantError)->RunResult;
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
        if self.main_failed() {
            self.assert_panicked_with(
                "message used to get the actual error message in case of an unexpected panic",
            )
        }

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

}

impl RevertibleProgram for Program<'_> {
    #[track_caller]
    fn send_and_assert_panic<'a>(&'a mut self, from: u64, payload: impl Codec, error: InvariantError)->RunResult {
        // state is reverted manually to match the behavior of the runtime
        let path = std::format!("./target/tmp/invariant_memory_dump{}", FILE_BACKUP_NR.fetch_add(1, Ordering::Relaxed));
        self.save_memory_dump(path.clone());
        let res = self.send(from, payload);
        res.assert_panicked_with(error);
        self.load_memory_dump(path);
        res
    }
}