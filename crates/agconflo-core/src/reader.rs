//! The topology reader: one document's text turned into what it describes, and a
//! fault in that text refused with its place.
//!
//! Read with toml_edit alone, walking the parsed document by hand rather than
//! deserializing it. Every table and value in a parsed document carries the span
//! a fault is located by (`EVD_TOML_EDIT_SPANS`), and its tables hand their keys
//! on in the order they were written (`EVD_TOML_EDIT_KEEPS_ORDER`) - which is
//! the order a node assembles its inputs in, so it is data here.
//!
//! Keys the model does not name are passed over rather than refused. A document
//! an editor has annotated is the ordinary case, not a fault in it.

use std::fmt;
use std::ops::Range;

use toml_edit::{Document, Item, TableLike};

use crate::context::{ContextType, InvalidTypeName};
use crate::workflow::{NodeType, Parameter};

/// The node types one document declares, under the name its caller gave the
/// document.
///
/// The name travels with the declarations because a type declared twice is
/// found only once several documents are gathered, and the fault then has to
/// say which documents it was (`CREQ_CATALOGUE_DECLARED_ONCE`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeTypeDocument {
    pub(crate) document: String,
    pub(crate) node_types: Vec<NodeType>,
}

impl NodeTypeDocument {
    /// The name the caller gave this document when handing it over.
    pub fn document(&self) -> &str {
        &self.document
    }

    /// The node types this document declares, in the order it declares them.
    pub fn node_types(&self) -> &[NodeType] {
        &self.node_types
    }
}

/// A document that cannot be read, and where.
///
/// The place is three values rather than a sentence, for the reason a wiring
/// defect carries its place as fields (`CREQ_DEFECT_NAMES_PLACE`): the caller
/// this is written for is an agent correcting its own document, and it reads
/// fields. The line and column are counted from one, and the column in
/// characters, exactly as the parser's own message counts them - a place one off
/// from the message beside it sends an author to the wrong character.
///
/// The document is named by whatever its caller called it, since which documents
/// are read is the caller's to say (`DEC_TYPES_IN_OWN_DOCUMENTS`).
#[derive(Clone, Debug, PartialEq, Eq)]
// @A fault in the text located as values,IMPL_READER_PLACE,impl,[CREQ_READER_FAULT_LOCATED]
pub struct ReadFault {
    document: String,
    line: usize,
    column: usize,
    kind: FaultKind,
}

impl ReadFault {
    /// The name the caller gave the document that cannot be read.
    pub fn document(&self) -> &str {
        &self.document
    }

    /// The line of the fault, counting the first line as 1.
    pub fn line(&self) -> usize {
        self.line
    }

    /// The column of the fault in characters, counting the first as 1.
    pub fn column(&self) -> usize {
        self.column
    }

    /// What is wrong at that place.
    pub fn kind(&self) -> &FaultKind {
        &self.kind
    }
}

/// What makes a document unreadable.
///
/// Every one of these is a fault in the text rather than in the workflow it
/// describes. A name that resolves to nothing is not among them: it is a wiring
/// defect, and reading lets it through for the validator to report with
/// everything else (`CREQ_READER_NAMES_UNRESOLVED`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FaultKind {
    /// The text is not TOML. A name written twice within one document is among
    /// these, since every name is a table key (`DEC_NAMES_AS_KEYS`) and TOML
    /// forbids a key written twice.
    Syntax {
        /// The parser's own account of the fault, without its place.
        message: String,
    },
    /// A value is not of the kind its key holds.
    WrongType {
        /// The key holding the value, from the top of the document down.
        key: Vec<String>,
        /// The kind of value that key holds, as toml_edit names kinds.
        expected: &'static str,
        /// The kind of value it holds instead.
        found: &'static str,
    },
    /// A key the reader needs is absent. The place is the table that lacks it.
    MissingKey {
        /// The absent key, from the top of the document down.
        key: Vec<String>,
    },
    /// A context type name that the model refuses (`CREQ_VALUE_DECLARED_TYPE`),
    /// though the text holds it as a well-formed string.
    InvalidContextType {
        /// The key holding the name, from the top of the document down.
        key: Vec<String>,
        /// Why the model refuses it.
        reason: InvalidTypeName,
    },
}

impl fmt::Display for ReadFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.document, self.line, self.column, self.kind
        )
    }
}

impl fmt::Display for FaultKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { message } => write!(f, "not TOML: {message}"),
            Self::WrongType {
                key,
                expected,
                found,
            } => write!(
                f,
                "'{}' is of type {found}, where type {expected} is expected",
                key.join(".")
            ),
            Self::MissingKey { key } => write!(f, "'{}' is missing", key.join(".")),
            Self::InvalidContextType { key, reason } => {
                write!(f, "'{}' is not a context type: {reason}", key.join("."))
            }
        }
    }
}

impl std::error::Error for ReadFault {}

/// The node types `text` declares, read from a document its caller calls
/// `document`.
///
/// Each type is a table under `types`, keyed by its name, holding up to three
/// tables and an array - `required` and `optional`, each keying a parameter's
/// context type by the parameter's name, and `globals`, the context types read
/// by declaration - and `output`, the one context type it produces. Only
/// `output` is needed; an absent list is an empty one, and a document without
/// `types` declares none.
///
/// Parameters come back in the order they are written, whether a list is
/// written as a header table, an inline table or dotted keys. Nothing here
/// sorts them, collects them into a sorted map, or reads the three lists by
/// position rather than by key.
// @A node type document read in the order it is written,IMPL_READER_TYPES,impl,[CREQ_READER_TYPES]
pub fn read_node_types(document: &str, text: &str) -> Result<NodeTypeDocument, ReadFault> {
    let reading = Reading { document, text };
    let parsed = reading.parse()?;

    let mut node_types = Vec::new();
    if let Some(types) = parsed.get("types") {
        let types = reading.table(types, &["types"])?;
        for (name, declaration) in types.iter() {
            node_types.push(reading.node_type(name, declaration)?);
        }
    }

    Ok(NodeTypeDocument {
        document: document.to_owned(),
        node_types,
    })
}

/// One document being read: what its caller calls it, and its text, which is
/// what every fault in it is located against.
struct Reading<'t> {
    document: &'t str,
    text: &'t str,
}

impl<'t> Reading<'t> {
    /// The text parsed, or refused where the parser stopped.
    fn parse(&self) -> Result<Document<&'t str>, ReadFault> {
        Document::parse(self.text).map_err(|refused| {
            self.fault(
                refused.span(),
                FaultKind::Syntax {
                    message: refused.message().to_owned(),
                },
            )
        })
    }

    /// A fault of `kind` at the start of `span`, or at the start of the document
    /// for an item without one.
    fn fault(&self, span: Option<Range<usize>>, kind: FaultKind) -> ReadFault {
        let (line, column) = line_and_column(self.text, span.map_or(0, |span| span.start));
        ReadFault {
            document: self.document.to_owned(),
            line,
            column,
            kind,
        }
    }

    /// `item` read as a table, whichever of TOML's ways of writing one it is.
    fn table<'i>(&self, item: &'i Item, key: &[&str]) -> Result<&'i dyn TableLike, ReadFault> {
        item.as_table_like()
            .ok_or_else(|| self.wrong_type(item.span(), key, "table", item.type_name()))
    }

    /// The item under `name` in `table`, which `owner` is written as, or a fault
    /// at `owner` for its absence.
    fn needed<'i>(
        &self,
        owner: &Item,
        table: &'i dyn TableLike,
        key: &[&str],
        name: &str,
    ) -> Result<&'i Item, ReadFault> {
        table.get(name).ok_or_else(|| {
            self.fault(
                owner.span(),
                FaultKind::MissingKey {
                    key: path(&[key, &[name]].concat()),
                },
            )
        })
    }

    /// `item` read as a context type, whose name has to be a string and one the
    /// model accepts.
    fn context_type(&self, item: &Item, key: &[&str]) -> Result<ContextType, ReadFault> {
        self.context_type_at(item.as_str(), item.type_name(), item.span(), key)
    }

    /// A context type from what a value holds: `name` if it is a string, `kind`
    /// naming what it is otherwise, and `span` where it is written.
    fn context_type_at(
        &self,
        name: Option<&str>,
        kind: &'static str,
        span: Option<Range<usize>>,
        key: &[&str],
    ) -> Result<ContextType, ReadFault> {
        let Some(name) = name else {
            return Err(self.wrong_type(span, key, "string", kind));
        };
        ContextType::new(name).map_err(|reason| {
            self.fault(
                span,
                FaultKind::InvalidContextType {
                    key: path(key),
                    reason,
                },
            )
        })
    }

    fn wrong_type(
        &self,
        span: Option<Range<usize>>,
        key: &[&str],
        expected: &'static str,
        found: &'static str,
    ) -> ReadFault {
        self.fault(
            span,
            FaultKind::WrongType {
                key: path(key),
                expected,
                found,
            },
        )
    }

    /// One node type, declared under `types.<name>`.
    fn node_type(&self, name: &str, item: &Item) -> Result<NodeType, ReadFault> {
        let key = ["types", name];
        let declaration = self.table(item, &key)?;
        let output = self.needed(item, declaration, &key, "output")?;

        Ok(NodeType {
            name: name.to_owned(),
            required: self.parameters(declaration, &key, "required")?,
            optional: self.parameters(declaration, &key, "optional")?,
            globals: self.globals(declaration, &key)?,
            output: self.context_type(output, &[&key[..], &["output"]].concat())?,
        })
    }

    /// The parameters listed under `list`, in the order they are written: each
    /// key a parameter's name, each value its context type.
    fn parameters(
        &self,
        declaration: &dyn TableLike,
        key: &[&str],
        list: &str,
    ) -> Result<Vec<Parameter>, ReadFault> {
        let Some(item) = declaration.get(list) else {
            return Ok(Vec::new());
        };
        let key = [key, &[list]].concat();

        self.table(item, &key)?
            .iter()
            .map(|(name, declared)| {
                Ok(Parameter {
                    name: name.to_owned(),
                    context_type: self.context_type(declared, &[&key[..], &[name]].concat())?,
                })
            })
            .collect()
    }

    /// The context types a node type reads by declaration, in the order written.
    fn globals(
        &self,
        declaration: &dyn TableLike,
        key: &[&str],
    ) -> Result<Vec<ContextType>, ReadFault> {
        let Some(item) = declaration.get("globals") else {
            return Ok(Vec::new());
        };
        let key = [key, &["globals"]].concat();
        let Some(array) = item.as_array() else {
            return Err(self.wrong_type(item.span(), &key, "array", item.type_name()));
        };

        array
            .iter()
            .map(|global| {
                self.context_type_at(global.as_str(), global.type_name(), global.span(), &key)
            })
            .collect()
    }
}

/// A key path as the fault carries it.
fn path(key: &[&str]) -> Vec<String> {
    key.iter().map(|&part| part.to_owned()).collect()
}

/// The line and column of byte `offset` in `text`, counted from one, the column
/// in characters.
///
/// Counted exactly as toml_edit counts them when it writes a refusal's message,
/// since the two reach a caller side by side, and that count lives in a private
/// function of the library (`EVD_TOML_EDIT_SPANS`). So this is a port of it,
/// down to its two edges: an offset at or past the end of the text is placed
/// just after the last character, on its line, and an offset inside a character
/// wider than one byte falls back to counting bytes.
fn line_and_column(text: &str, offset: usize) -> (usize, usize) {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return (1, offset + 1);
    }

    let within = offset.min(bytes.len() - 1);
    let past_the_end = offset - within;
    let line_start = bytes[..within]
        .iter()
        .rposition(|&byte| byte == b'\n')
        .map_or(0, |newline| newline + 1);
    let line = bytes[..line_start]
        .iter()
        .filter(|&&byte| byte == b'\n')
        .count();
    let column = std::str::from_utf8(&bytes[line_start..=within])
        .map_or(within - line_start, |up_to| up_to.chars().count() - 1);

    (line + 1, column + past_the_end + 1)
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases, for the reason given in id.rs.

#[cfg(test)]
use crate::workflow::node_type;
#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::prelude::*;

#[cfg(test)]
proptest! {
    /// Any list of distinct names, written as a node type's required parameters,
    /// reads back in the order written - in an inline table, in dotted keys and
    /// under a header of their own, and each of those in the order generated and
    /// reverse-sorted too, since that is the order in which every name moves.
    ///
    /// The whole declaration is compared, not the names alone, so a reader that
    /// kept the order and lost a type, or filed a parameter in another list,
    /// fails here too.
    #[test]
    fn parameter_order_is_kept(names in vec("[a-z]{1,6}", 1..=8)) {
        let mut given: Vec<String> = Vec::new();
        for name in names {
            if !given.contains(&name) {
                given.push(name);
            }
        }
        let mut reversed = given.clone();
        reversed.sort();
        reversed.reverse();

        let mut text = String::new();
        let mut expected = Vec::new();
        for (label, order) in [("given", &given), ("reversed", &reversed)] {
            let inline: Vec<String> = order.iter().map(|name| format!("{name} = \"note\"")).collect();
            text += &format!(
                "[types.{label}_inline]\noutput = \"note\"\nrequired = {{ {} }}\n\n",
                inline.join(", ")
            );
            text += &format!("[types.{label}_dotted]\noutput = \"note\"\n");
            for name in order.iter() {
                text += &format!("required.{name} = \"note\"\n");
            }
            text += &format!("\n[types.{label}_header]\noutput = \"note\"\n\n[types.{label}_header.required]\n");
            for name in order.iter() {
                text += &format!("{name} = \"note\"\n");
            }
            text += "\n";

            let declared: Vec<(&str, &str)> = order.iter().map(|name| (name.as_str(), "note")).collect();
            for form in ["inline", "dotted", "header"] {
                expected.push(node_type(&format!("{label}_{form}"), &declared, "note"));
            }
        }

        let read = read_node_types("types.toml", &text);
        prop_assert_eq!(read.map(|document| document.node_types), Ok(expected));
    }
}

#[test]
fn three_lists_kept_apart() {
    // Written globals, optional, required - an order other than the one the
    // model holds them in - so a reader taking the lists by position puts each
    // in the wrong one. Each list declares its own context types, so a list read
    // as another cannot come out equal by accident.
    //
    // It carries keys the model does not name as well, at the top and on a
    // type, which an editor's document always will: a strict reader refuses it.
    let text = "\
[editor]
zoom = 2

[types.review]
description = \"summarises a diff\"
globals = [\"policy\", \"style\"]
optional = { hint = \"note\" }
required = { diff = \"diff\", notes = \"summary\" }
output = \"summary\"

[types.bare]
output = \"note\"
";

    let read = read_node_types("types.toml", text).expect("the document reads");
    assert_eq!(
        read.node_types(),
        [
            node_type(
                "review",
                &[("diff", "diff"), ("notes", "summary")],
                "summary"
            )
            .with_optional(&[("hint", "note")])
            .with_globals(&["policy", "style"]),
            // Declaring none of the three is a type with three empty lists and
            // its output, not a fault.
            node_type("bare", &[], "note"),
        ]
    );
    assert_eq!(read.document(), "types.toml");
}

#[test]
fn empty_context_type_is_a_fault() {
    // A parameter, an output and a global each declared with the context type
    // "", which is a well-formed TOML string and a name the model refuses. Each
    // is a fault at that value - the document it is in, its line, and the
    // column of its opening quote.
    let cases = [
        (
            "parameter.toml",
            "[types.t]\noutput = \"note\"\nrequired = { input = \"\" }\n",
            (3, 22),
            vec!["types", "t", "required", "input"],
        ),
        (
            "output.toml",
            "# a type producing nothing nameable\n[types.t]\noutput = \"\"\n",
            (3, 10),
            vec!["types", "t", "output"],
        ),
        (
            "global.toml",
            "[types.t]\noutput = \"note\"\nglobals = [\"policy\", \"\"]\n",
            (3, 22),
            vec!["types", "t", "globals"],
        ),
    ];

    for (document, text, place, key) in cases {
        let fault = read_node_types(document, text).expect_err("an empty type name is refused");
        assert_eq!(
            (fault.document(), fault.line(), fault.column()),
            (document, place.0, place.1),
            "{fault}"
        );
        assert_eq!(
            fault.kind(),
            &FaultKind::InvalidContextType {
                key: key.iter().map(|&part| part.to_owned()).collect(),
                reason: InvalidTypeName::Empty,
            }
        );
    }
}
