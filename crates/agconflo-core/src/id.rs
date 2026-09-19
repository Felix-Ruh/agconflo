//! Context identifiers, and the one source that issues them.

use std::fmt;

/// The identifier of one context, shared with no other context in the same run.
///
/// Only an [`IdSource`] makes one. The field is private and nothing converts
/// into this type, so an identifier cannot be written down, defaulted or parsed
/// into existence. Copying one is harmless: a copy names the same context, and
/// a context is created from a source rather than from an identifier, so a
/// copy can never label a second one.
///
/// Deliberately without an ordering. Nothing promises that a later context
/// carries a larger identifier, and an `Ord` would invite code to assume it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ContextId(u64);

/// Issues context identifiers, never the same one twice.
///
/// One source stands for one run. It is neither `Clone` nor `Copy`, because two
/// copies would issue the same sequence, each correct on its own and repeating
/// jointly. Two sources created separately would do the same, and preventing
/// that belongs to whatever owns a run once the engine has one.
#[derive(Debug)]
pub struct IdSource {
    /// The next identifier to issue, or `None` once the last has been issued.
    next: Option<u64>,
}

impl IdSource {
    /// A source that has issued nothing yet.
    pub fn new() -> Self {
        Self { next: Some(0) }
    }

    /// A source whose next identifier is `first`, so that a test can reach the
    /// end of the space without counting to it. Being private to this module is
    /// what keeps every other caller from restarting the sequence; it is
    /// compiled into test builds only because the tests below are its one use.
    #[cfg(test)]
    fn starting_at(first: u64) -> Self {
        Self { next: Some(first) }
    }

    /// Issues the next identifier, or refuses once every identifier has been
    /// issued. The refusal is permanent: wrapping round would reissue the first
    /// identifier without a word, which is the defect this exists to rule out.
    pub(crate) fn issue(&mut self) -> Result<ContextId, SourceExhausted> {
        let id = self.next.ok_or(SourceExhausted)?;
        self.next = id.checked_add(1);
        Ok(ContextId(id))
    }
}

/// Written out rather than derived. A derived `Default` sets `next` to `None`,
/// which is a source that has already run out - measured, and exactly the
/// derive that clippy's `new_without_default` invites.
impl Default for IdSource {
    fn default() -> Self {
        Self::new()
    }
}

/// The source has issued every identifier it has, and refuses every request
/// from then on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SourceExhausted;

impl fmt::Display for SourceExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the identifier source has issued every identifier it has")
    }
}

impl std::error::Error for SourceExhausted {}

// --- tests -------------------------------------------------------------------
// Bare functions rather than a `tests` module, because each name is the id of
// the test case it implements: the runner reports a unit test by its full
// module path, so `id::never_repeats` is TEST_ID_NEVER_REPEATS in
// docs/tests/context.rst, and a `tests` module would add a segment to every one
// (EVD_NEXTEST_TEST_PATHS). A test without a case would have nothing to report
// its result against, so there are exactly as many tests here as cases.

#[cfg(test)]
use crate::compile_fail::{assert_compiles, assert_refused};
#[cfg(test)]
use crate::{Context, ContextType};
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use std::collections::HashSet;

#[cfg(test)]
proptest! {
    #[test]
    fn never_repeats(requests in 0..=10_000usize) {
        // Through `default()` on purpose, since that is the constructor a
        // derive would quietly break: its source is exhausted from the start.
        let mut source = IdSource::default();
        let mut issued = HashSet::new();
        for _ in 0..requests {
            let Ok(id) = source.issue() else {
                let message = format!("a fresh source refused after {} ids", issued.len());
                return Err(TestCaseError::fail(message));
            };
            prop_assert!(issued.insert(id), "{:?} was issued twice", id);
        }
    }
}

#[test]
fn exhaustion_is_permanent() {
    let mut source = IdSource::starting_at(u64::MAX);
    assert_eq!(source.issue(), Ok(ContextId(u64::MAX)));

    for _ in 0..3 {
        assert_eq!(source.issue(), Err(SourceExhausted));
    }

    // Creating a context is a request for an identifier too, and is refused
    // the same way rather than creating something with no identity.
    let refused = Context::text(&mut source, ContextType::new("note").unwrap(), "text");
    assert_eq!(refused.map(|context| context.id()), Err(SourceExhausted));
    assert_eq!(source.issue(), Err(SourceExhausted));
}

#[test]
fn source_cannot_be_copied() {
    // The control shares every line with the two refusals except the one that
    // copies, so a refusal is about the copy and not about the lines around it.
    assert_compiles(
        "source_move",
        "let source = agconflo_core::IdSource::new(); let _moved = source;",
    );
    assert_refused(
        "source_copy",
        "let source = agconflo_core::IdSource::new(); let _moved = source; let _again = source;",
        "E0382",
        "does not implement the `Copy` trait",
    );
    assert_refused(
        "source_clone",
        "let source = agconflo_core::IdSource::new(); let _moved = source.clone();",
        "E0599",
        "no method named `clone` found for struct `IdSource`",
    );
}

#[test]
fn cannot_be_forged() {
    // The control obtains an identifier the only way there is, and names its
    // type, so each refusal is about its route and not about the name.
    assert_compiles(
        "id_from_source",
        "let mut source = agconflo_core::IdSource::new(); \
         let note = agconflo_core::ContextType::new(\"note\").unwrap(); \
         let context = agconflo_core::Context::text(&mut source, note, \"text\").unwrap(); \
         let _id: agconflo_core::ContextId = context.id();",
    );
    assert_refused(
        "id_forged",
        "let _id: agconflo_core::ContextId = agconflo_core::ContextId(7);",
        "E0423",
        "constructor is not visible here due to private fields",
    );

    // The routes a helpful derive or impl would open. Each would hand out an
    // identifier that a source has issued or will issue to a context.
    assert_refused(
        "id_defaulted",
        "let _id = agconflo_core::ContextId::default();",
        "E0599",
        "named `default` found for struct `ContextId`",
    );
    assert_refused(
        "id_converted",
        "let _id: agconflo_core::ContextId = 7u64.into();",
        "E0277",
        "the trait bound `ContextId: From<u64>` is not satisfied",
    );
    assert_refused(
        "id_parsed",
        "let _id: agconflo_core::ContextId = \"7\".parse().unwrap();",
        "E0277",
        "the trait bound `ContextId: FromStr` is not satisfied",
    );
}
