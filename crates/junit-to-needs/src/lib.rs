//! Turns the JUnit report cargo-nextest writes into `test_run` needs for the
//! requirements graph: one per test, naming the test, the test case it ran and
//! whether it passed.
//!
//! A test's case is `TEST_` followed by the test's path, uppercased, with `::`
//! as `_`. Nothing else from the report is written.

mod id;
mod import;
mod needs;

pub use import::{ImportError, import};
pub use needs::{Outcome, Run, Runs};
