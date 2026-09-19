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
/// content, which is reached only through [`Context::render`].
///
/// Nothing here can change once the value exists. There is no public field and
/// no method taking `&mut self`, so every read of a context sees the value it
/// was created as.
///
/// There is deliberately no metadata. `DEC_METADATA_TRANSFORMED` has no
/// requirement to answer to until Transform nodes exist, and because content is
/// reached through methods, metadata can be added later without breaking any
/// caller. Its absence is a decision, not an omission to fill in.
#[derive(Debug)]
pub struct Context {
    id: ContextId,
    declared_type: ContextType,
    content: Content,
}

/// What a context holds. Only text so far; composition adds a variant holding
/// its parts and the separator that joins them.
#[derive(Debug)]
enum Content {
    Text(Box<str>),
}

impl Context {
    /// A context holding `text` exactly as given, with an identifier issued by
    /// `source`.
    ///
    /// Takes the source rather than an identifier: identifiers can be copied,
    /// so accepting one would let a single identifier label two contexts.
    /// Refused only when the source has run out, and then nothing is created.
    pub fn text(
        source: &mut IdSource,
        declared_type: ContextType,
        text: impl Into<String>,
    ) -> Result<Self, SourceExhausted> {
        Ok(Self {
            id: source.issue()?,
            declared_type,
            content: Content::Text(text.into().into_boxed_str()),
        })
    }

    /// The identifier, which no other context in the run shares.
    pub fn id(&self) -> ContextId {
        self.id
    }

    /// The type this context was declared with.
    pub fn declared_type(&self) -> &ContextType {
        &self.declared_type
    }

    /// The content, identical on every read.
    ///
    /// Text comes back byte for byte - no line endings converted, nothing
    /// trimmed, no Unicode normalisation - and borrowed rather than copied.
    /// Content computed from parts will come back owned, which is why this is a
    /// `Cow` rather than a `&str` that would need a stored copy to point into.
    pub fn render(&self) -> Cow<'_, str> {
        match &self.content {
            Content::Text(text) => Cow::Borrowed(text),
        }
    }
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases, for the reason given in id.rs.

#[cfg(test)]
use proptest::prelude::*;

/// Any text at all. Not `any::<String>()`: its pattern is `\PC*`, which never
/// generates a control character, so CR, LF and NUL - the bytes a normalising
/// defect changes - would never be tried.
#[cfg(test)]
fn any_text() -> impl Strategy<Value = String> {
    proptest::collection::vec(any::<char>(), 0..64).prop_map(String::from_iter)
}

#[cfg(test)]
fn note() -> ContextType {
    ContextType::new("note").expect("a non-empty name is accepted")
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
    fn reads_are_identical(text in any_text()) {
        let context = Context::text(&mut IdSource::new(), note(), text).unwrap();
        let first = context.render().into_owned();
        let second = context.render();
        prop_assert_eq!(second.as_bytes(), first.as_bytes());
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
fn empty_type_refused() {
    assert_eq!(ContextType::new(""), Err(InvalidTypeName::Empty));

    let mut source = IdSource::new();
    let summary = ContextType::new("summary").expect("a non-empty name is accepted");
    let context = Context::text(&mut source, summary, "text").expect("the source is fresh");
    assert_eq!(context.declared_type().as_str(), "summary");
    assert_eq!(context.render(), "text");
}
