//! The context value: identified, typed, and unchangeable once created.

use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

use crate::id::{ContextId, IdSource, SourceExhausted};

/// The type a context was declared with.
///
/// Nominal: two types are the same exactly when their names are, and nothing
/// infers one from content. Checked when the name is made rather than when a
/// context is, so a context cannot be declared with an invalid type at all.
/// Cheap to clone, since every context of a type carries it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ContextType(Arc<str>);

impl ContextType {
    /// A type named `name`, refusing only an empty name.
    ///
    /// Which characters a name may hold is left to the workflow format, which
    /// is why the error is open to further kinds.
    // @A declared type that refuses an empty name,IMPL_CONTEXT_TYPE,impl,[CREQ_VALUE_DECLARED_TYPE]
    pub fn new(name: &str) -> Result<Self, InvalidTypeName> {
        if name.is_empty() {
            return Err(InvalidTypeName::Empty);
        }
        Ok(Self(Arc::from(name)))
    }

    /// The name, exactly as it was declared.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A type name that cannot declare a context.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum InvalidTypeName {
    /// The name was empty. A type keys global contexts and validates wiring,
    /// and an empty key does neither.
    Empty,
}

impl fmt::Display for InvalidTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("a context type name must not be empty"),
        }
    }
}

impl std::error::Error for InvalidTypeName {}

/// An immutable piece of context: its identifier, its declared type, and its
/// content, which is reached only through [`Context::render`] and
/// [`Context::parts`].
///
/// A handle: cloning one shares the same value, identifier included, rather
/// than copying it. That is how a composition holds its parts by reference, and
/// how one context reaches every consumer wired to it.
///
/// Nothing here can change once the value exists. There is no public field, no
/// method taking `&mut self`, and no interior mutability, so every read of a
/// context sees the value it was created as.
///
/// There is deliberately no metadata. `DEC_METADATA_TRANSFORMED` has no
/// requirement to answer to until Transform nodes exist, and because content is
/// reached through methods, metadata can be added later without breaking any
/// caller. Its absence is a decision, not an omission to fill in.
#[derive(Clone)]
pub struct Context(Arc<Node>);

/// The value a handle shares.
struct Node {
    id: ContextId,
    declared_type: ContextType,
    content: Content,
}

/// What a context holds: text, or other contexts and the separator that joins
/// them - never a flattened copy of their text.
enum Content {
    Text(Box<str>),
    Composed {
        parts: Vec<Context>,
        separator: Box<str>,
    },
}

impl Context {
    /// A context holding `text` exactly as given, with an identifier issued by
    /// `source`.
    ///
    /// Takes the source rather than an identifier: identifiers can be copied,
    /// so accepting one would let a single identifier label two contexts.
    /// Refused only when the source has run out, and then nothing is created.
    // @Text held byte for byte,IMPL_CONTEXT_TEXT,impl,[CREQ_VALUE_TEXT_EXACT]
    pub fn text(
        source: &mut IdSource,
        declared_type: ContextType,
        text: impl Into<String>,
    ) -> Result<Self, SourceExhausted> {
        let text = Content::Text(text.into().into_boxed_str());
        Ok(Self::new(source.issue()?, declared_type, text))
    }

    /// A context composed of `parts` in the order given, rendered as their
    /// content joined by `separator`.
    ///
    /// Each part is held as the context itself, not as a copy of its text, so
    /// it stays addressable and keeps its own identifier. The same part may be
    /// given more than once, parts may be of any declared type, and there may
    /// be none at all. The composition's type is `declared_type`, whatever the
    /// parts' types are. Refused only when the source has run out, and then
    /// nothing is created.
    // @Composing by reference under a declared type,IMPL_CONTEXT_COMPOSE,impl,[CREQ_VALUE_PARTS_BY_REFERENCE, CREQ_VALUE_DECLARED_TYPE]
    pub fn compose<'a>(
        source: &mut IdSource,
        declared_type: ContextType,
        parts: impl IntoIterator<Item = &'a Context>,
        separator: &str,
    ) -> Result<Self, SourceExhausted> {
        let id = source.issue()?;
        let parts = parts.into_iter().cloned().collect();
        let composed = Content::Composed {
            parts,
            separator: separator.into(),
        };
        Ok(Self::new(id, declared_type, composed))
    }

    fn new(id: ContextId, declared_type: ContextType, content: Content) -> Self {
        Self(Arc::new(Node {
            id,
            declared_type,
            content,
        }))
    }

    /// The identifier, which no other context in the run shares.
    pub fn id(&self) -> ContextId {
        self.0.id
    }

    /// The type this context was declared with.
    pub fn declared_type(&self) -> &ContextType {
        &self.0.declared_type
    }

    /// The parts of a composed context, in the order they were given, each the
    /// context itself. A text context has none.
    // @Parts returned as the originals,IMPL_CONTEXT_PARTS,impl,[CREQ_VALUE_PARTS_BY_REFERENCE]
    pub fn parts(&self) -> &[Context] {
        match &self.0.content {
            Content::Text(_) => &[],
            Content::Composed { parts, .. } => parts,
        }
    }

    /// The content, identical on every read.
    ///
    /// Text comes back byte for byte - no line endings converted, nothing
    /// trimmed, no Unicode normalisation - and borrowed rather than copied. A
    /// composition comes back as its parts' content in order, joined by its
    /// separator, computed on each read; that is why this is a `Cow` rather than
    /// a `&str`, which would need a stored copy to point into.
    // @Text rendered exactly and parts joined,IMPL_CONTEXT_RENDER,impl,[CREQ_VALUE_TEXT_EXACT, CREQ_VALUE_RENDER_JOINED]
    pub fn render(&self) -> Cow<'_, str> {
        match &self.0.content {
            Content::Text(text) => Cow::Borrowed(text),
            Content::Composed { .. } => Cow::Owned(self.render_composed()),
        }
    }

    /// Walks the composition with a stack of its own rather than the call
    /// stack. Nesting has no fixed depth, and a recursive render overflowed
    /// even a 32 MiB stack at 100,000 levels - measured - which is an abort,
    /// not a failure anyone can assert on.
    fn render_composed(&self) -> String {
        enum Step<'a> {
            Part(&'a Context),
            Separator(&'a str),
        }

        let mut rendered = String::new();
        let mut steps = vec![Step::Part(self)];
        while let Some(step) = steps.pop() {
            match step {
                Step::Separator(separator) => rendered.push_str(separator),
                Step::Part(context) => match &context.0.content {
                    Content::Text(text) => rendered.push_str(text),
                    Content::Composed { parts, separator } => {
                        // Pushed last first, so that they come off in order.
                        for (index, part) in parts.iter().enumerate().rev() {
                            steps.push(Step::Part(part));
                            if index > 0 {
                                steps.push(Step::Separator(separator));
                            }
                        }
                    }
                },
            }
        }
        rendered
    }
}

/// Written out, because a derived one would recurse into every part: an abort
/// on a deep context, and a whole tree printed on a shallow one. Parts are
/// shown by identifier instead.
impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Context");
        debug
            .field("id", &self.0.id)
            .field("declared_type", &self.0.declared_type.as_str());
        match &self.0.content {
            Content::Text(text) => debug.field("text", text),
            Content::Composed { parts, separator } => {
                let parts: Vec<ContextId> = parts.iter().map(Context::id).collect();
                debug.field("parts", &parts).field("separator", separator)
            }
        };
        debug.finish()
    }
}

/// Releases a composition's parts from a worklist rather than by recursion.
/// The derived drop spends stack frames on every level, and overflowed even a
/// 32 MiB stack at 100,000 levels - measured.
///
/// A part is taken apart here only when this was its last holder, which is
/// exactly when `Arc::into_inner` hands it over. A part still held anywhere
/// else is left whole.
// @A deep chain released without recursion,IMPL_CONTEXT_RELEASE,impl,[CREQ_VALUE_PARTS_BY_REFERENCE]
impl Drop for Node {
    fn drop(&mut self) {
        let Content::Composed { parts, .. } = &mut self.content else {
            return;
        };
        let mut released = std::mem::take(parts);
        while let Some(part) = released.pop() {
            if let Some(mut node) = Arc::into_inner(part.0) {
                if let Content::Composed { parts, .. } = &mut node.content {
                    released.append(parts);
                }
                // `node` is dropped here with no parts left, so its own drop
                // returns at once instead of recursing.
            }
        }
    }
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases, for the reason given in id.rs.

#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use proptest::sample::Index;

/// Deep enough to exhaust any default stack under recursion: measured, a
/// recursive render, drop or walk overflowed an 8 MiB stack at this depth.
#[cfg(test)]
const DEPTH: usize = 100_000;

/// Any text at all. Not `any::<String>()`: its pattern is `\PC*`, which never
/// generates a control character, so CR, LF and NUL - the bytes a normalising
/// defect changes - would never be tried.
#[cfg(test)]
fn any_text() -> impl Strategy<Value = String> {
    vec(any::<char>(), 0..64).prop_map(String::from_iter)
}

#[cfg(test)]
fn any_type_name() -> impl Strategy<Value = String> {
    vec(any::<char>(), 1..12).prop_map(String::from_iter)
}

#[cfg(test)]
fn note() -> ContextType {
    ContextType::new("note").expect("a non-empty name is accepted")
}

/// A context as the test describes it, so that what it should render as is
/// worked out from the texts alone rather than by the code under test.
#[cfg(test)]
#[derive(Clone, Debug)]
enum Tree {
    Text(String),
    Composed(Vec<Tree>, String),
}

#[cfg(test)]
impl Tree {
    /// The context described. Recursive, which is safe here only because the
    /// generated trees are a few levels deep.
    fn build(&self, source: &mut IdSource) -> Context {
        match self {
            Tree::Text(text) => Context::text(source, note(), text.as_str()).unwrap(),
            Tree::Composed(parts, separator) => {
                let parts: Vec<Context> = parts.iter().map(|part| part.build(source)).collect();
                Context::compose(source, note(), &parts, separator).unwrap()
            }
        }
    }

    fn expected(&self) -> String {
        match self {
            Tree::Text(text) => text.clone(),
            Tree::Composed(parts, separator) => {
                let parts: Vec<String> = parts.iter().map(Tree::expected).collect();
                parts.join(separator)
            }
        }
    }
}

/// Texts and compositions of them, nested up to four deep, each composition
/// with a separator of its own - empty included.
#[cfg(test)]
fn any_tree() -> impl Strategy<Value = Tree> {
    let separator = vec(any::<char>(), 0..3).prop_map(String::from_iter);
    any_text()
        .prop_map(Tree::Text)
        .prop_recursive(4, 32, 4, move |inner| {
            (vec(inner, 0..4), separator.clone())
                .prop_map(|(parts, separator)| Tree::Composed(parts, separator))
        })
}

#[cfg(test)]
fn compose_notes(source: &mut IdSource, parts: &[&Context], separator: &str) -> Context {
    Context::compose(source, note(), parts.iter().copied(), separator).unwrap()
}

#[cfg(test)]
proptest! {
    #[test]
    fn text_round_trips(text in any_text()) {
        let context = Context::text(&mut IdSource::new(), note(), text.clone()).unwrap();
        let rendered = context.render();
        prop_assert_eq!(rendered.as_bytes(), text.as_bytes());
    }

    #[test]
    fn reads_are_identical(tree in any_tree()) {
        let context = tree.build(&mut IdSource::new());
        let first = context.render().into_owned();
        let second = context.render();
        prop_assert_eq!(second.as_bytes(), first.as_bytes());
    }

    #[test]
    fn render_is_joined_parts(tree in any_tree()) {
        let context = tree.build(&mut IdSource::new());
        let rendered = context.render();
        let expected = tree.expected();
        prop_assert_eq!(rendered.as_bytes(), expected.as_bytes());
    }

    #[test]
    fn parts_are_originals(
        // Each entry is text, or - when it picks earlier entries - a
        // composition of them. Compositions have to be among the parts: the
        // defect this exists for flattens a composed part into a copy of its
        // text, and a pool of text alone would never show it.
        pool in vec((any_type_name(), any_text(), vec(any::<Index>(), 0..3)), 1..6),
        picks in vec(any::<Index>(), 0..12),
    ) {
        let mut source = IdSource::new();
        let mut contexts: Vec<Context> = Vec::new();
        for (entry, (name, text, entry_picks)) in pool.iter().enumerate() {
            let declared = ContextType::new(name).unwrap();
            let context = if entry == 0 || entry_picks.is_empty() {
                Context::text(&mut source, declared, text.as_str()).unwrap()
            } else {
                let parts = entry_picks.iter().map(|pick| &contexts[pick.index(entry)]);
                Context::compose(&mut source, declared, parts, text).unwrap()
            };
            contexts.push(context);
        }
        let given: Vec<&Context> = picks.iter().map(|pick| pick.get(&contexts)).collect();

        let composed = compose_notes(&mut source, &given, ", ");
        let expected: Vec<ContextId> = given.iter().map(|part| part.id()).collect();
        let actual: Vec<ContextId> = composed.parts().iter().map(Context::id).collect();
        prop_assert_eq!(actual, expected);
    }

    #[test]
    fn type_is_declared(
        declared in any_type_name(),
        // `true` declares a part with the composition's own name. The first
        // branch never does, which is the case an inferred type gets wrong; the
        // second mixes both, since composing summaries into a summary is
        // ordinary.
        shares_name in prop_oneof![vec(Just(false), 1..6), vec(any::<bool>(), 0..6)],
    ) {
        let mut source = IdSource::new();
        let declared_type = ContextType::new(&declared).unwrap();
        let parts: Vec<Context> = shares_name
            .iter()
            .enumerate()
            .map(|(index, &shares)| {
                // Suffixed, so it differs from the declared name by construction.
                let name = if shares { declared.clone() } else { format!("{declared}~{index}") };
                Context::text(&mut source, ContextType::new(&name).unwrap(), "part").unwrap()
            })
            .collect();

        let composed = Context::compose(&mut source, declared_type, &parts, "").unwrap();
        prop_assert_eq!(composed.declared_type().as_str(), declared.as_str());
    }
}

#[test]
fn text_edge_cases() {
    let cases = [
        ("empty text", ""),
        ("CRLF", "one\r\ntwo\r\n"),
        ("a lone CR", "one\rtwo"),
        ("leading and trailing whitespace", " \t padded \n "),
        ("a letter and a combining accent", "e\u{301}"),
        ("an embedded NUL", "before\0after"),
    ];
    let mut source = IdSource::new();
    for (name, text) in cases {
        let context = Context::text(&mut source, note(), text).unwrap();
        assert_eq!(context.render().as_bytes(), text.as_bytes(), "{name}");
    }
}

#[test]
fn render_edge_cases() {
    let mut source = IdSource::new();
    let a = Context::text(&mut source, note(), "a").unwrap();
    let b = Context::text(&mut source, note(), "b").unwrap();

    let none = compose_notes(&mut source, &[], ", ");
    assert_eq!(none.render(), "", "no parts");
    let one = compose_notes(&mut source, &[&a], ", ");
    assert_eq!(one.render(), "a", "one part");
    let concatenated = compose_notes(&mut source, &[&a, &b], "");
    assert_eq!(concatenated.render(), "ab", "an empty separator");
    let twice = compose_notes(&mut source, &[&a, &a], "+");
    assert_eq!(twice.render(), "a+a", "a part given twice");
}

#[test]
fn deep_nesting_renders() {
    // Every level puts the deeper level first and the one shared leaf second,
    // so the content is the leaf DEPTH + 1 times, joined by commas.
    let mut source = IdSource::new();
    let leaf = Context::text(&mut source, note(), "x").unwrap();
    let mut context = leaf.clone();
    for _ in 0..DEPTH {
        context = compose_notes(&mut source, &[&context, &leaf], ",");
    }
    assert_eq!(context.render(), vec!["x"; DEPTH + 1].join(","));

    // Leaked on purpose. Releasing the chain is `deep_nesting_drops`'s case,
    // and a defect there would otherwise abort this one too, reporting the
    // renderer as broken when it is not.
    std::mem::forget(context);
}

#[test]
fn parts_edge_cases() {
    let mut source = IdSource::new();

    let none = compose_notes(&mut source, &[], ", ");
    assert!(none.parts().is_empty(), "no parts");

    let part = Context::text(&mut source, note(), "twice").unwrap();
    let twice = compose_notes(&mut source, &[&part, &part], ", ");
    let ids: Vec<ContextId> = twice.parts().iter().map(Context::id).collect();
    assert_eq!(ids, [part.id(), part.id()], "a context given twice");

    let summary_type = ContextType::new("summary").unwrap();
    let summary = Context::text(&mut source, summary_type, "summary").unwrap();
    let transcript_type = ContextType::new("transcript").unwrap();
    let transcript = Context::text(&mut source, transcript_type, "transcript").unwrap();
    let mixed = compose_notes(&mut source, &[&summary, &transcript], ", ");
    let types: Vec<&str> = mixed
        .parts()
        .iter()
        .map(|p| p.declared_type().as_str())
        .collect();
    assert_eq!(
        types,
        ["summary", "transcript"],
        "parts of different declared types"
    );
}

#[test]
fn deep_nesting_drops() {
    let mut source = IdSource::new();
    let bottom = Context::text(&mut source, note(), "bottom").unwrap();
    // Not merely "it did not abort": a drop that leaked the chain would pass
    // that, so the bottom has to be seen to go.
    let bottom_alive = Arc::downgrade(&bottom.0);
    let mut context = bottom;
    for _ in 0..DEPTH {
        context = compose_notes(&mut source, &[&context], "");
    }
    drop(context);
    assert!(
        bottom_alive.upgrade().is_none(),
        "the chain was not released"
    );
}

#[test]
fn empty_type_refused() {
    assert_eq!(ContextType::new(""), Err(InvalidTypeName::Empty));

    let mut source = IdSource::new();
    let summary = ContextType::new("summary").expect("a non-empty name is accepted");
    let context = Context::text(&mut source, summary, "text").expect("the source is fresh");
    assert_eq!(context.declared_type().as_str(), "summary");
    assert_eq!(context.render(), "text");
}
