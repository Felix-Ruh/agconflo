//! The lineage walker: every context a context was composed from, at any depth.

use std::collections::HashSet;

use crate::Context;

impl Context {
    /// Every context reachable through this one's parts, each reported once and
    /// in no particular order. The context itself is not among them, and a text
    /// context has none.
    ///
    /// A set rather than a list of paths. Parts are shared, so one ancestor is
    /// often reachable along several paths, and there can be exponentially many
    /// of them; each context is visited once, so a walk costs time in
    /// proportion to the contexts and parts in the ancestry, whatever its shape.
    /// It always ends, since a context can only be composed from contexts that
    /// already exist.
    ///
    /// Contexts are told apart by identifier alone, as
    /// `DEC_NO_CONTENT_ADDRESSING` has it. Two sources in one run would issue
    /// the same identifiers and a walk across both would take different
    /// contexts for one; keeping a run to one source belongs to whatever owns
    /// the run.
    // @Each ancestor walked once,IMPL_LINEAGE_WALK,impl,[CREQ_WALKER_EACH_ONCE]
    pub fn lineage(&self) -> Vec<&Context> {
        let mut seen = HashSet::new();
        let mut ancestry = Vec::new();
        // A stack of its own rather than recursion, for the reason rendering
        // has one: a recursive walk overflowed an 8 MiB stack at 100,000 levels.
        let mut pending: Vec<&Context> = self.parts().iter().collect();
        while let Some(context) = pending.pop() {
            if seen.insert(context.id()) {
                ancestry.push(context);
                pending.extend(context.parts());
            }
        }
        ancestry
    }
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases, for the reason given in id.rs.

#[cfg(test)]
use crate::{ContextId, ContextType, IdSource};
#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use proptest::sample::Index;
#[cfg(test)]
use std::collections::BTreeSet;

#[cfg(test)]
fn note() -> ContextType {
    ContextType::new("note").expect("a non-empty name is accepted")
}

#[cfg(test)]
fn compose_notes(source: &mut IdSource, parts: &[&Context]) -> Context {
    Context::compose(source, note(), parts.iter().copied(), "").unwrap()
}

/// The identifiers reported, after checking that none is reported twice.
#[cfg(test)]
fn reported_once(lineage: &[&Context]) -> HashSet<ContextId> {
    let ids: HashSet<ContextId> = lineage.iter().map(|context| context.id()).collect();
    assert_eq!(ids.len(), lineage.len(), "an ancestor was reported twice");
    ids
}

#[cfg(test)]
proptest! {
    /// Each node of the generated graph is composed of earlier nodes, picked
    /// with repeats, so parts are shared freely; a node picking nothing is
    /// text. What each node should reach is worked out from the picks alone,
    /// in creation order - a different computation from the walk under test.
    #[test]
    fn reaches_each_once(picks in vec(vec(any::<Index>(), 0..4), 1..16)) {
        let mut source = IdSource::new();
        let mut contexts: Vec<Context> = Vec::new();
        let mut reaches: Vec<BTreeSet<usize>> = Vec::new();

        for (node, node_picks) in picks.iter().enumerate() {
            let parts: Vec<usize> = if node == 0 {
                Vec::new()
            } else {
                node_picks.iter().map(|pick| pick.index(node)).collect()
            };
            let context = if parts.is_empty() {
                Context::text(&mut source, note(), "text").unwrap()
            } else {
                let parts: Vec<&Context> = parts.iter().map(|&part| &contexts[part]).collect();
                compose_notes(&mut source, &parts)
            };
            let mut reached = BTreeSet::new();
            for &part in &parts {
                reached.insert(part);
                reached.extend(reaches[part].iter().copied());
            }
            contexts.push(context);
            reaches.push(reached);
        }

        for (context, reached) in contexts.iter().zip(&reaches) {
            let reported = reported_once(&context.lineage());
            prop_assert!(!reported.contains(&context.id()), "a context is its own ancestor");
            let expected: HashSet<ContextId> = reached.iter().map(|&node| contexts[node].id()).collect();
            prop_assert_eq!(reported, expected);
        }
    }
}

#[test]
fn edge_cases() {
    let mut source = IdSource::new();
    let text = Context::text(&mut source, note(), "text").unwrap();
    assert!(text.lineage().is_empty(), "a text context");

    let twice = compose_notes(&mut source, &[&text, &text]);
    let reported = reported_once(&twice.lineage());
    assert_eq!(reported, HashSet::from([text.id()]), "a part given twice");

    // `text` is reachable directly and through `twice`.
    let two_paths = compose_notes(&mut source, &[&text, &twice]);
    let reported = reported_once(&two_paths.lineage());
    let expected = HashSet::from([text.id(), twice.id()]);
    assert_eq!(reported, expected, "an ancestor along two paths");
}

#[test]
fn diamonds_are_linear() {
    // Each diamond is two contexts sharing the level below, joined by one
    // above: 64 of them over one bottom context make 193, and two to the 64
    // paths from the top to the bottom.
    let mut source = IdSource::new();
    let mut top = Context::text(&mut source, note(), "bottom").unwrap();
    for _ in 0..64 {
        let left = compose_notes(&mut source, &[&top]);
        let right = compose_notes(&mut source, &[&top]);
        top = compose_notes(&mut source, &[&left, &right]);
    }
    assert_eq!(reported_once(&top.lineage()).len(), 192);
}

#[test]
fn deep_nesting_walks() {
    let mut source = IdSource::new();
    let mut context = Context::text(&mut source, note(), "bottom").unwrap();
    for _ in 0..100_000 {
        context = compose_notes(&mut source, &[&context]);
    }
    assert_eq!(reported_once(&context.lineage()).len(), 100_000);

    // Leaked on purpose, for the reason `context::deep_nesting_renders` gives:
    // a defect in releasing the chain belongs to its own case.
    std::mem::forget(context);
}
