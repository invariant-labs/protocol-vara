use gtest::*;
use math::percentage::Percentage;
use crate::test_helpers::gtest::consts::*;
use crate::test_helpers::consts::*;

use io::*;

pub fn init_invariant(sys: &System, protocol_fee: Percentage) -> Program<'_> {
  let program = Program::from_file_with_id(sys, INVARIANT_ID, INVARIANT_PATH);

  assert!(!program
      .send(
          PROGRAM_OWNER,
          InitInvariant {
              config: InvariantConfig {
                  admin: ADMIN.into(),
                  protocol_fee,
              },
          },
      )
      .main_failed());
  program
}