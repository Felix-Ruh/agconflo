//! Context identifiers, and the one source that issues them.

use std::fmt;

/// The identifier of one context, shared with no other context in the same run.
///
/// Only an [`IdSource`] makes one, or a run's record naming one a source issued
/// before the run was interrupted. It cannot be written down, defaulted or
/// parsed into existence, and it has no ordering.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
// @Identifiers only a source can make,IMPL_ID_CONTEXT_ID,impl,[CREQ_SOURCE_SOLE_ISSUER],[NOTE_ID_TRAITS]
pub struct ContextId(u64);

/// Issues context identifiers, never the same one twice. One source stands for
/// one run, and it can be neither cloned nor copied.
#[derive(Debug)]
// @A source that cannot be copied,IMPL_ID_SOURCE,impl,[CREQ_SOURCE_NO_REPEAT]
pub struct IdSource {
    /// The next identifier to issue, or `None` once the last has been issued.
    next: Option<u64>,
}

impl IdSource {
    /// A source that has issued nothing yet.
    pub fn new() -> Self {
        Self { next: Some(0) }
    }

    /// A source whose next identifier is `first`, for a test reaching the end of
    /// the space without counting to it. Private to this module, and test-only.
    #[cfg(test)]
    fn starting_at(first: u64) -> Self {
        Self { next: Some(first) }
    }

    /// Issues the next identifier, or refuses once every identifier has been
    /// issued. The refusal is permanent.
    // @Issuing each identifier once,IMPL_ID_ISSUE,impl,[CREQ_SOURCE_NO_REPEAT]
    pub(crate) fn issue(&mut self) -> Result<ContextId, SourceExhausted> {
        let id = self.next.ok_or(SourceExhausted)?;
        self.next = id.checked_add(1);
        Ok(ContextId(id))
    }

    /// The identifier this source would issue next, or `None` once it has
    /// issued them all.
    // @A source's position for its record,TRACE_ID_POSITION,trace,[],[DEC_RECORD_CARRIES_THE_SOURCE]
    pub(crate) fn position(&self) -> Option<u64> {
        self.next
    }

    /// A source standing where a recorded one stood. The run record calls it only
    /// after refusing a record holding an identifier at or past `position`.
    pub(crate) fn resumed_at(position: Option<u64>) -> Self {
        Self { next: position }
    }
}

impl ContextId {
    /// An identifier read back from a run's record, named again rather than
    /// issued. Called only by the run record, which hands it back beside a source
    /// positioned past it.
    // @An identifier named again from a record,TRACE_ID_RESUMED,trace,[],[DEC_RECORD_CARRIES_THE_SOURCE]
    pub(crate) fn resumed(value: u64) -> Self {
        Self(value)
    }

    /// The number this identifier is, for writing it into a record.
    pub(crate) fn value(self) -> u64 {
        self.0
    }
}

/// A source that has issued nothing yet, as [`IdSource::new`] is.
// @A default source written out,TRACE_ID_DEFAULT,trace,[],[NOTE_ID_TRAITS]
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
// Bare functions, each named after the test case it implements.

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

    // Creating a context from text or by composing is refused the same way, and
    // creates nothing.
    let note = ContextType::new("note").unwrap();
    let text = Context::text(&mut source, note.clone(), "text");
    assert_eq!(text.map(|context| context.id()), Err(SourceExhausted));
    let composed = Context::compose(&mut source, note, [], "");
    assert_eq!(composed.map(|context| context.id()), Err(SourceExhausted));
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
