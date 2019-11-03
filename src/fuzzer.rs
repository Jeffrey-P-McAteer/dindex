#[macro_use]
use afl;

use dindex;

fn main() {
  afl::fuzz!(|data: &[u8]| {
    std::str::from_utf8(data).unwrap();
  });
}

