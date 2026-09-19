//! Turns the JUnit report cargo-nextest writes into `test_run` needs for the
//! requirements graph: one per test, naming the test case it ran and whether it
//! passed.
//!
//! Development tooling rather than engine, so it answers to no requirement and
//! carries no trace markers; codelinks reads `agconflo-core` alone.
//!
//! A test is matched to its test case by name, never by annotation. The id is
//! `TEST_` and the test's path uppercased with `::` as `_`, so
//! `context::text_round_trips` ran `TEST_CONTEXT_TEXT_ROUND_TRIPS`. A test whose
//! case does not exist is not something this crate can see: the run it writes
//! names a case that is missing, and the graph's check fails on that dead link.
//!
//! Only what changes when an outcome does is written. A report also carries the
//! run's id, timestamps, durations and failure text, which all differ between
//! runs of the same tests (EVD_NEXTEST_TEST_PATHS); written out, they would
//! change a committed file on every run whatever the tests did.

mod id;
mod import;
mod needs;

pub use import::{ImportError, import};
pub use needs::{Outcome, Run, Runs};
