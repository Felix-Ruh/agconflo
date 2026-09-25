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

use toml_edit::{Document, DocumentMut, Item, TableLike};

use crate::catalogue::TypeCatalogue;
use crate::context::{ContextType, InvalidTypeName};
use crate::workflow::{Binding, NodeInstance, NodeType, Parameter, WorkflowDefinition};

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
    /// The text is not TOML. A name written twice within one table is among
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
    /// A node type declares one parameter as both required and optional. The
    /// two lists are two tables, so the parser sees no repeated key, and the
    /// place is where the name repeats: whichever declaration is written later.
    ParameterDeclaredTwice {
        /// The later declaration, from the top of the document down.
        key: Vec<String>,
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
            Self::ParameterDeclaredTwice { key } => write!(
                f,
                "'{}' is declared as both a required and an optional parameter",
                key.join(".")
            ),
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
/// by declaration - and `output`, the one context type it produces, and may hold
/// a `description` of what it does, shown to a model it is offered to. Only
/// `output` is needed; an absent list is an empty one, an absent description is
/// empty, and a document without `types` declares none.
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

/// A workflow document as it was read, handed on with the definition.
///
/// Handed on so that writing a definition back can edit this document rather
/// than write a new one, and everything a definition does not carry - comments,
/// keys the model does not name, the author's layout - can stay where it was
/// (`CREQ_WRITER_KEEPS_UNREAD`). Its text is what [`fmt::Display`] writes.
#[derive(Clone, Debug)]
pub struct WorkflowDocument {
    pub(crate) document: String,
    pub(crate) toml: DocumentMut,
    /// Whether the text read ended its lines in CRLF, which toml_edit does not
    /// keep (`EVD_TOML_EDIT_WRITES_LF`).
    pub(crate) crlf: bool,
}

impl WorkflowDocument {
    /// The name the caller gave this document when handing it over.
    pub fn document(&self) -> &str {
        &self.document
    }
}

/// The workflow `text` describes, read from a document its caller calls
/// `document`, with the node types of `catalogue` - and the document itself, for
/// the writer.
///
/// The document names the workflow with `name` and may designate its output
/// with `output`. Each instance is a table under `instances`, keyed by its name,
/// holding the `node_type` it names, whether it is an `entry` node, and its
/// `bindings`, each keying the instance a parameter is wired to by the
/// parameter's name. Only `name` and each `node_type` are needed.
///
/// Three absences are read as nothing rather than as something, and each of the
/// other readings is a defect somebody would reach for. An absent `output`
/// designates no output, not the only instance or the last: the validator
/// refuses that with everything else wrong (`DEC_ONE_OUTPUT_KEY`). An absent
/// `entry` is not an entry node, since an entry node's parameters are exempt from
/// being bound and defaulting the other way passes a workflow that is not wired
/// at all. And absent `instances` or `bindings` are none.
///
/// The definition carries every declaration in the catalogue, whether or not an
/// instance names it, since the catalogue is what its instances are checked
/// against.
// @A workflow document read into a definition,IMPL_READER_WORKFLOW,impl,[CREQ_READER_WORKFLOW]
pub fn read_workflow(
    document: &str,
    text: &str,
    catalogue: &TypeCatalogue,
) -> Result<(WorkflowDefinition, WorkflowDocument), ReadFault> {
    let reading = Reading { document, text };
    let parsed = reading.parse()?;

    let name = reading.needed(parsed.as_item(), parsed.as_table(), &[], "name")?;
    let name = reading.string(name, &["name"])?.to_owned();

    let mut instances = Vec::new();
    if let Some(item) = parsed.get("instances") {
        for (instance, written) in reading.table(item, &["instances"])?.iter() {
            instances.push(reading.instance(instance, written)?);
        }
    }

    let mut designated_outputs = Vec::new();
    if let Some(output) = parsed.get("output") {
        designated_outputs.push(reading.string(output, &["output"])?.to_owned());
    }

    let definition = WorkflowDefinition {
        name,
        node_types: catalogue.node_types().to_vec(),
        instances,
        designated_outputs,
    };
    let document = WorkflowDocument {
        document: document.to_owned(),
        toml: parsed.into_mut(),
        crlf: text.contains("\r\n"),
    };
    Ok((definition, document))
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

    /// `item` read as a string.
    fn string<'i>(&self, item: &'i Item, key: &[&str]) -> Result<&'i str, ReadFault> {
        item.as_str()
            .ok_or_else(|| self.wrong_type(item.span(), key, "string", item.type_name()))
    }

    /// One instance, written under `instances.<name>`.
    ///
    /// Every name in it is copied as it is written and looked up nowhere: not the
    /// type it names, not the instance each binding is wired to, not the
    /// parameter each binding fills. A name that resolves to nothing is a wiring
    /// defect, and the validator reports it with every other one; a reader that
    /// looked it up could only refuse the first it met, and the author would learn
    /// of the second after fixing it.
    // @Names read as written and looked up nowhere,IMPL_READER_NAMES_AS_WRITTEN,impl,[CREQ_READER_NAMES_UNRESOLVED]
    fn instance(&self, name: &str, item: &Item) -> Result<NodeInstance, ReadFault> {
        let key = ["instances", name];
        let table = self.table(item, &key)?;

        let node_type = self.needed(item, table, &key, "node_type")?;
        let node_type = self
            .string(node_type, &[&key[..], &["node_type"]].concat())?
            .to_owned();

        let entry = match table.get("entry") {
            None => false,
            Some(entry) => entry.as_bool().ok_or_else(|| {
                self.wrong_type(
                    entry.span(),
                    &[&key[..], &["entry"]].concat(),
                    "boolean",
                    entry.type_name(),
                )
            })?,
        };

        let mut bindings = Vec::new();
        if let Some(item) = table.get("bindings") {
            let key = [&key[..], &["bindings"]].concat();
            for (parameter, source) in self.table(item, &key)?.iter() {
                bindings.push(Binding {
                    parameter: parameter.to_owned(),
                    source: self
                        .string(source, &[&key[..], &[parameter]].concat())?
                        .to_owned(),
                });
            }
        }

        Ok(NodeInstance {
            name: name.to_owned(),
            node_type,
            entry,
            bindings,
            calls: self.calls(table, &key)?,
        })
    }

    /// The node types an instance declares calls to, in the order its `calls`
    /// array lists them, or none when it has no such key.
    ///
    /// Each name is copied as written and looked up nowhere, as every other name
    /// of an instance is: a call to a node type nobody supplied is a wiring
    /// defect (`CREQ_VALIDATOR_CALL_RESOLVES`). A name listed twice is read
    /// twice. A key that is not an array of strings is refused where it is
    /// written rather than read as no calls, since that would declare nothing
    /// without a word.
    // @An instance's calls read in the order written,IMPL_READER_CALLS,impl,[CREQ_READER_CALLS]
    fn calls(&self, table: &dyn TableLike, key: &[&str]) -> Result<Vec<String>, ReadFault> {
        let Some(item) = table.get("calls") else {
            return Ok(Vec::new());
        };
        let key = [key, &["calls"]].concat();
        let Some(array) = item.as_array() else {
            return Err(self.wrong_type(item.span(), &key, "array", item.type_name()));
        };

        array
            .iter()
            .map(|call| {
                call.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| self.wrong_type(call.span(), &key, "string", call.type_name()))
            })
            .collect()
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
        let required = self.parameters(declaration, &key, "required")?;
        let optional = self.parameters(declaration, &key, "optional")?;
        self.declared_once(declaration, &key, &required, &optional)?;

        let description = match declaration.get("description") {
            None => String::new(),
            Some(item) => self
                .string(item, &[&key[..], &["description"]].concat())?
                .to_owned(),
        };

        Ok(NodeType {
            name: name.to_owned(),
            description,
            required,
            optional,
            globals: self.globals(declaration, &key)?,
            output: self.context_type(output, &[&key[..], &["output"]].concat())?,
        })
    }

    /// Nothing, or a fault for a parameter `declaration` lists as both
    /// required and optional.
    ///
    /// The parser refuses a name written twice within one list, since it is a
    /// repeated key, and cannot see one written once in each: the two lists are
    /// two tables. Read as it stands, the validator would judge every wire into
    /// that parameter by one declaration and ignore the other.
    ///
    /// The fault is placed where the name repeats, as the parser places a
    /// repeated key: at whichever of the two declarations is written later. With
    /// several such names, the one that repeats first in the text is reported.
    /// Names are compared exactly, as they are everywhere else - `input` and
    /// `Input` are two parameters.
    // @A parameter declared in both lists refused where it repeats,IMPL_READER_DECLARED_ONCE,impl,[CREQ_READER_FAULT_LOCATED]
    fn declared_once(
        &self,
        declaration: &dyn TableLike,
        key: &[&str],
        required: &[Parameter],
        optional: &[Parameter],
    ) -> Result<(), ReadFault> {
        // A key read from a parsed document has a span - the test places an
        // inline and a header spelling exactly. A missing one would still
        // refuse the document, at its start at worst, rather than let the
        // repetition through.
        let written_at = |list: &str, name: &str| {
            declaration
                .get(list)
                .and_then(Item::as_table_like)
                .and_then(|table| table.get_key_value(name))
                .and_then(|(written, _)| written.span())
                .map(|span| span.start)
        };

        let repeated = optional
            .iter()
            .filter(|parameter| required.iter().any(|other| other.name == parameter.name))
            .map(|parameter| {
                let as_required = written_at("required", &parameter.name);
                let as_optional = written_at("optional", &parameter.name);
                let (list, at) = if as_optional > as_required {
                    ("optional", as_optional)
                } else {
                    ("required", as_required)
                };
                (at, list, parameter.name.as_str())
            })
            .min_by_key(|&(at, ..)| at);

        match repeated {
            None => Ok(()),
            Some((at, list, name)) => Err(self.fault(
                at.map(|at| at..at),
                FaultKind::ParameterDeclaredTwice {
                    key: path(&[key, &[list, name]].concat()),
                },
            )),
        }
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
pub(crate) fn line_and_column(text: &str, offset: usize) -> (usize, usize) {
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
use crate::catalogue::node_type_document;
#[cfg(test)]
use crate::defect::WiringDefect;
#[cfg(test)]
use crate::wiring::validate_wiring;
#[cfg(test)]
use crate::workflow::{context_type, instance, node_type};
#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::prelude::*;

// --- documents the tests write -----------------------------------------------
// Written with a few lines of formatting of their own rather than with the
// writer, so that a reader and a writer wrong in matching ways cannot pass
// together. The writer's tests use them too, for the same reason.

/// A workflow document as a test writes it: a definition's worth of names, and
/// the choices about how each is written that a definition has no place for.
#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct Written {
    pub(crate) name: String,
    pub(crate) output: Option<String>,
    /// Whether the instances not written as header tables are dotted keys at
    /// the top of the document rather than inline tables under `[instances]`.
    /// TOML allows one or the other alongside header tables, never both.
    pub(crate) dotted: bool,
    pub(crate) instances: Vec<WrittenInstance>,
}

/// One instance as a test writes it.
#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct WrittenInstance {
    pub(crate) name: String,
    pub(crate) node_type: String,
    /// The entry key: absent, or present and true or false.
    pub(crate) entry: Option<bool>,
    pub(crate) bindings: Vec<(String, String)>,
    /// The node types it declares calls to, written as an array when there are
    /// any.
    pub(crate) calls: Vec<String>,
    /// Whether it is a header table of its own, `[instances.<name>]`.
    pub(crate) header: bool,
    /// For a header table, whether its bindings get a header of their own too.
    pub(crate) bindings_header: bool,
}

#[cfg(test)]
impl Written {
    /// The instances in the order the text holds them: TOML writes a table's own
    /// keys before its sub-tables, so every instance that is not a header table
    /// comes first.
    pub(crate) fn in_text_order(&self) -> Vec<&WrittenInstance> {
        let (headers, others): (Vec<_>, Vec<_>) =
            self.instances.iter().partition(|instance| instance.header);
        others.into_iter().chain(headers).collect()
    }

    /// The document's text.
    pub(crate) fn text(&self) -> String {
        let mut text = format!("name = {}\n", quoted(&self.name));
        if let Some(output) = &self.output {
            text += &format!("output = {}\n", quoted(output));
        }

        let others = self.instances.iter().filter(|instance| !instance.header);
        if self.dotted {
            for instance in others {
                let at = format!("instances.{}", key(&instance.name));
                text += &format!("{at}.node_type = {}\n", quoted(&instance.node_type));
                if let Some(entry) = instance.entry {
                    text += &format!("{at}.entry = {entry}\n");
                }
                if !instance.calls.is_empty() {
                    text += &format!("{at}.calls = {}\n", array(&instance.calls));
                }
                for (parameter, source) in &instance.bindings {
                    text += &format!("{at}.bindings.{} = {}\n", key(parameter), quoted(source));
                }
            }
        } else {
            let others: Vec<&WrittenInstance> = others.collect();
            if !others.is_empty() {
                text += "\n[instances]\n";
            }
            for instance in others {
                let mut fields = vec![format!("node_type = {}", quoted(&instance.node_type))];
                if let Some(entry) = instance.entry {
                    fields.push(format!("entry = {entry}"));
                }
                if !instance.calls.is_empty() {
                    fields.push(format!("calls = {}", array(&instance.calls)));
                }
                if !instance.bindings.is_empty() {
                    fields.push(format!(
                        "bindings = {}",
                        inline_bindings(&instance.bindings)
                    ));
                }
                text += &format!("{} = {{ {} }}\n", key(&instance.name), fields.join(", "));
            }
        }

        for instance in self.instances.iter().filter(|instance| instance.header) {
            let at = format!("instances.{}", key(&instance.name));
            text += &format!("\n[{at}]\nnode_type = {}\n", quoted(&instance.node_type));
            if let Some(entry) = instance.entry {
                text += &format!("entry = {entry}\n");
            }
            if !instance.calls.is_empty() {
                text += &format!("calls = {}\n", array(&instance.calls));
            }
            if instance.bindings.is_empty() {
                continue;
            }
            if instance.bindings_header {
                text += &format!("\n[{at}.bindings]\n");
                for (parameter, source) in &instance.bindings {
                    text += &format!("{} = {}\n", key(parameter), quoted(source));
                }
            } else {
                text += &format!("bindings = {}\n", inline_bindings(&instance.bindings));
            }
        }
        text
    }

    /// The definition the text describes, read with `catalogue`.
    pub(crate) fn definition(&self, catalogue: &TypeCatalogue) -> WorkflowDefinition {
        WorkflowDefinition {
            name: self.name.clone(),
            node_types: catalogue.node_types().to_vec(),
            instances: self
                .in_text_order()
                .into_iter()
                .map(|written| NodeInstance {
                    name: written.name.clone(),
                    node_type: written.node_type.clone(),
                    entry: written.entry.unwrap_or(false),
                    bindings: written
                        .bindings
                        .iter()
                        .map(|(parameter, source)| Binding {
                            parameter: parameter.clone(),
                            source: source.clone(),
                        })
                        .collect(),
                    calls: written.calls.clone(),
                })
                .collect(),
            designated_outputs: self.output.iter().cloned().collect(),
        }
    }
}

/// `name` as a TOML key: bare where TOML allows it, quoted otherwise.
#[cfg(test)]
fn key(name: &str) -> String {
    let bare = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if bare { name.to_owned() } else { quoted(name) }
}

/// `value` as a TOML basic string.
#[cfg(test)]
fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// `names` as a TOML array written the way a person might: padded inside its
/// brackets, and its first name as a literal string where TOML allows one.
///
/// Not the writer's own way of writing an array, on purpose. An array the writer
/// rewrote while its names stayed the same would then come back as different
/// text, so leaving an unchanged list alone is something a test can see.
#[cfg(test)]
fn array(names: &[String]) -> String {
    let items: Vec<String> = names
        .iter()
        .enumerate()
        .map(|(position, name)| {
            if position == 0 && !name.contains('\'') && !name.contains('\n') {
                format!("'{name}'")
            } else {
                quoted(name)
            }
        })
        .collect();
    format!("[ {} ]", items.join(" , "))
}

#[cfg(test)]
fn inline_bindings(bindings: &[(String, String)]) -> String {
    let pairs: Vec<String> = bindings
        .iter()
        .map(|(parameter, source)| format!("{} = {}", key(parameter), quoted(source)))
        .collect();
    format!("{{ {} }}", pairs.join(", "))
}

/// Where the names a written document holds are drawn from.
#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) struct Pools {
    /// Node types an instance may name.
    pub(crate) types: &'static [&'static str],
    /// Parameters a binding may fill.
    pub(crate) parameters: &'static [&'static str],
    /// Names a binding or the output may give besides the instances'.
    pub(crate) strangers: &'static [&'static str],
}

/// Names an instance may have, some of which TOML only takes quoted.
#[cfg(test)]
const INSTANCE_NAMES: [&str; 6] = ["a", "b", "fetch", "review", "two words", "x-1"];

/// Written workflow documents of every shape the format allows: instances as
/// header tables, inline tables and dotted keys, bindings inline and under a
/// header of their own, the entry key absent, true and false, and an output or
/// none - naming what `pools` offers.
#[cfg(test)]
pub(crate) fn any_written(pools: Pools) -> impl Strategy<Value = Written> {
    let shapes = vec(
        (
            any::<usize>(),
            proptest::option::of(any::<bool>()),
            any::<bool>(),
            any::<bool>(),
            vec((any::<usize>(), any::<usize>()), 0..=3),
            vec(any::<usize>(), 0..=3),
        ),
        INSTANCE_NAMES.len(),
    );
    (
        proptest::sample::subsequence(INSTANCE_NAMES.to_vec(), 0..=5).prop_shuffle(),
        shapes,
        "[a-zA-Z0-9 _\u{df}-]{0,10}",
        proptest::option::of(any::<usize>()),
        any::<bool>(),
    )
        .prop_map(move |(names, shapes, name, output, dotted)| {
            let named = |pick: usize| {
                let count = names.len() + pools.strangers.len();
                (count > 0).then(|| {
                    let pick = pick % count;
                    names
                        .get(pick)
                        .copied()
                        .unwrap_or_else(|| pools.strangers[pick - names.len()])
                        .to_owned()
                })
            };
            let instances = names
                .iter()
                .zip(&shapes)
                .map(
                    |(&instance, (declared, entry, header, bindings_header, wires, calls))| {
                        let mut bindings: Vec<(String, String)> = Vec::new();
                        for &(parameter, source) in wires {
                            let parameter = pools.parameters[parameter % pools.parameters.len()];
                            let Some(source) = named(source) else {
                                continue;
                            };
                            if bindings.iter().all(|(filled, _)| filled != parameter) {
                                bindings.push((parameter.to_owned(), source));
                            }
                        }
                        WrittenInstance {
                            name: instance.to_owned(),
                            node_type: pools.types[declared % pools.types.len()].to_owned(),
                            entry: *entry,
                            bindings,
                            calls: calls
                                .iter()
                                .map(|&call| pools.types[call % pools.types.len()].to_owned())
                                .collect(),
                            header: *header,
                            bindings_header: *bindings_header,
                        }
                    },
                )
                .collect();
            Written {
                name,
                output: output.and_then(named),
                dotted,
                instances,
            }
        })
}

/// The node types most workflow documents here are read with.
#[cfg(test)]
pub(crate) fn catalogue() -> TypeCatalogue {
    TypeCatalogue::gather(vec![node_type_document(
        "types.toml",
        vec![
            node_type("source", &[], "note"),
            node_type("sink", &[("input", "note")], "note").with_optional(&[("hint", "note")]),
        ],
    )])
    .expect("two distinct types gather")
}

/// Names that all resolve against `catalogue()`, so a reader that looked them
/// up would still read every document drawn from them.
#[cfg(test)]
const RESOLVABLE: Pools = Pools {
    types: &["source", "sink"],
    parameters: &["input", "hint"],
    strangers: &[],
};

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
colour = \"teal\"
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

#[cfg(test)]
proptest! {
    /// For any document written in any of the forms TOML allows, reading gives
    /// exactly what was written - the name, every instance with its type, entry
    /// flag and bindings, and the output or none - and the catalogue's
    /// declarations as its node types.
    ///
    /// The generator writes the entry key absent, present and true, and present
    /// and false. A reader defaulting an absent key to true reads every instance
    /// as an entry node, and nothing else here would see it: the validator
    /// exempts an entry node's parameters, so the definition it makes passes
    /// validation.
    #[test]
    fn reads_what_is_written(written in any_written(RESOLVABLE)) {
        let catalogue = catalogue();
        let read = read_workflow("workflow.toml", &written.text(), &catalogue);
        prop_assert_eq!(
            read.map(|(definition, _)| definition),
            Ok(written.definition(&catalogue))
        );
    }
}

#[test]
fn absent_output_is_no_output() {
    // One instance, which is the shape where a reader filling the gap has an
    // obvious candidate - and designating it would give the workflow an output
    // its author never named.
    let text = "name = \"lonely\"\n\n[instances.only]\nnode_type = \"source\"\n";
    let (definition, _) = read_workflow("lonely.toml", text, &catalogue()).expect("it reads");

    assert_eq!(definition.designated_outputs, Vec::<String>::new());
    // And what holds afterwards: the absence is refused where every other wiring
    // defect is, by the validator, rather than lost.
    assert_eq!(
        validate_wiring(&definition),
        vec![WiringDefect::SignatureOutputs {
            definition: "lonely".to_owned(),
            designated: 0,
        }]
    );
}

#[test]
fn unnamed_keys_are_read() {
    // A table at the top and keys on each instance that the model does not
    // name - the document every editor produces.
    let annotated = "\
# laid out by an editor
name = \"review\"
output = \"b\"
version = 3

[editor]
zoom = 2
grid = { snap = true }

[instances.a]
node_type = \"source\"
position = [10, 20]
colour = \"teal\"

[instances.b]
node_type = \"sink\"
note = \"checked by hand\"
bindings = { input = \"a\" }
";
    let catalogue = catalogue();
    let (definition, _) = read_workflow("annotated.toml", annotated, &catalogue).expect("it reads");

    assert_eq!(
        definition,
        WorkflowDefinition {
            name: "review".to_owned(),
            node_types: catalogue.node_types().to_vec(),
            instances: vec![
                instance("a", "source", &[]),
                instance("b", "sink", &[("input", "a")]),
            ],
            designated_outputs: vec!["b".to_owned()],
        }
    );
}

#[test]
fn empty_documents_read() {
    let catalogue = catalogue();

    let (nothing, _) =
        read_workflow("empty.toml", "name = \"empty\"\n", &catalogue).expect("no instances reads");
    assert_eq!(
        nothing,
        WorkflowDefinition {
            name: "empty".to_owned(),
            node_types: catalogue.node_types().to_vec(),
            instances: Vec::new(),
            designated_outputs: Vec::new(),
        }
    );
    // A wiring question, and the one the validator refuses for its signature
    // alone.
    assert_eq!(
        validate_wiring(&nothing),
        vec![WiringDefect::SignatureOutputs {
            definition: "empty".to_owned(),
            designated: 0,
        }]
    );

    // No bindings anywhere: one instance without the key, one with an empty
    // table under it.
    let unwired = "\
name = \"unwired\"

[instances.a]
node_type = \"sink\"

[instances.b]
node_type = \"sink\"
bindings = {}
";
    let (definition, _) = read_workflow("unwired.toml", unwired, &catalogue).expect("it reads");
    assert_eq!(
        definition.instances,
        vec![instance("a", "sink", &[]), instance("b", "sink", &[])]
    );
}

#[test]
fn faults_carry_their_place() {
    // The four ways a workflow document cannot be read, each under a name of its
    // own. The places are the fault's, counted from one, and read from the
    // fault's fields - never from its message.
    let key = |parts: &[&str]| parts.iter().map(|&part| part.to_owned()).collect();
    let cases = [
        (
            "not-toml.toml",
            "name = \"w\"\n[instances.a\nnode_type = \"source\"\n",
            (2, 13),
            FaultKind::Syntax {
                message: "unclosed table, expected `]`".to_owned(),
            },
        ),
        // The wrong value follows a two-byte character on its line: its column
        // is 38 in characters, and 39 counted from the span's bytes.
        (
            "wrong-type.toml",
            "name = \"w\"\n\n[instances.b]\nnode_type = \"sink\"\nbindings = { \"stra\u{df}e\" = \"a\", input = 5 }\n",
            (5, 38),
            FaultKind::WrongType {
                key: key(&["instances", "b", "bindings", "input"]),
                expected: "string",
                found: "integer",
            },
        ),
        (
            "untyped.toml",
            "name = \"w\"\n\n[instances.a]\nentry = true\n",
            (3, 1),
            FaultKind::MissingKey {
                key: key(&["instances", "a", "node_type"]),
            },
        ),
        (
            "repeated.toml",
            "name = \"w\"\n\n[instances.a]\nnode_type = \"source\"\n\n[instances.a]\nnode_type = \"sink\"\n",
            (6, 12),
            FaultKind::Syntax {
                message: "duplicate key".to_owned(),
            },
        ),
    ];

    for (document, text, (line, column), kind) in cases {
        let fault = read_workflow(document, text, &catalogue()).expect_err("it is refused");
        assert_eq!(
            (fault.document(), fault.line(), fault.column(), fault.kind()),
            (document, line, column, &kind),
            "{fault}"
        );
    }
}

/// The line and column in the first line of a toml_edit refusal's message.
#[cfg(test)]
fn place_in_message(message: &str) -> Option<(usize, usize)> {
    let rest = message.strip_prefix("TOML parse error at line ")?;
    let (line, rest) = rest.split_once(", column ")?;
    let column = rest.split(|c: char| !c.is_ascii_digit()).next()?;
    Some((line.parse().ok()?, column.parse().ok()?))
}

/// A valid document - a workflow or a node type document - with characters
/// deleted, inserted or swapped, or one value replaced by a value of another
/// kind. Arbitrary text almost never parses far enough to reach a value the
/// reader interprets, which is where an `unwrap` would sit; a valid document
/// nearly broken is what gets there.
///
/// The replaced value is there because the other three changes almost never
/// make one: measured over 512 documents, they gave faults in the syntax and
/// missing keys, and not one value of the wrong kind.
#[cfg(test)]
fn nearly_valid() -> impl Strategy<Value = String> {
    const TYPES: &str = "\
[types.review]
globals = [\"policy\"]
optional = { hint = \"note\" }
required = { diff = \"diff\" }
output = \"summary\"
";
    const PERMISSIVE: Pools = Pools {
        types: &["source", "sink", "ghost"],
        parameters: &["input", "hint"],
        strangers: &["missing"],
    };
    const OTHER_KINDS: [&str; 6] = ["5", "true", "[]", "{}", "\"s\"", "1979-05-27"];
    (
        any_written(PERMISSIVE),
        any::<bool>(),
        0..4usize,
        any::<usize>(),
        any::<usize>(),
        any::<char>(),
    )
        .prop_map(|(written, types, change, at, other, inserted)| {
            let text = if types {
                TYPES.to_owned()
            } else {
                written.text()
            };
            if change == 3 {
                // One value, the rest of its line after " = ", replaced.
                let assignments: Vec<usize> =
                    text.match_indices(" = ").map(|(at, _)| at + 3).collect();
                if assignments.is_empty() {
                    return text;
                }
                let start = assignments[at % assignments.len()];
                let end = text[start..]
                    .find('\n')
                    .map_or(text.len(), |line| start + line);
                return format!(
                    "{}{}{}",
                    &text[..start],
                    OTHER_KINDS[other % OTHER_KINDS.len()],
                    &text[end..]
                );
            }

            let mut text: Vec<char> = text.chars().collect();
            if text.is_empty() {
                return String::new();
            }
            let at = at % text.len();
            match change {
                0 => {
                    let end = (at + 1 + other % 4).min(text.len());
                    text.drain(at..end);
                }
                1 => text.insert(at, inserted),
                _ => {
                    let other = other % text.len();
                    text.swap(at, other);
                }
            }
            text.into_iter().collect()
        })
}

#[cfg(test)]
proptest! {
    /// For any text, reading it as either kind of document returns what it
    /// describes or a refusal with a place, and never ends the run. Where the
    /// parser is what refused the text, the place is the one its own message
    /// gives, since a caller sees the two side by side.
    #[test]
    fn any_text_is_read_or_refused(text in prop_oneof![any::<String>(), nearly_valid()]) {
        let read_as_workflow = read_workflow("any.toml", &text, &catalogue()).map(|_| ());
        let read_as_types = read_node_types("any.toml", &text).map(|_| ());

        for fault in [read_as_workflow, read_as_types].into_iter().filter_map(Result::err) {
            prop_assert_eq!(fault.document(), "any.toml");
            prop_assert!(fault.line() >= 1 && fault.column() >= 1, "{}", fault);
            if let FaultKind::Syntax { .. } = fault.kind() {
                let message = Document::parse(text.as_str())
                    .expect_err("the parser refused it")
                    .to_string();
                prop_assert_eq!(
                    Some((fault.line(), fault.column())),
                    place_in_message(&message),
                    "{}",
                    message
                );
            }
        }
    }
}

#[test]
fn every_wiring_defect_reads() {
    // An instance of a type no document declares, a binding to an instance that
    // is not there, a wire whose ends declare different context types, a
    // binding to a parameter its type does not declare, and no output. Each is
    // a name, or an absent key, and the text is fine.
    let text = "\
name = \"broken\"

[instances.b]
node_type = \"not-declared\"

[instances.c]
node_type = \"sink\"
bindings = { input = \"deleted\" }

[instances.d]
node_type = \"sink\"
bindings = { input = \"e\" }

[instances.e]
node_type = \"differ\"

[instances.f]
node_type = \"differ\"
bindings = { nonesuch = \"e\" }
";
    let catalogue = TypeCatalogue::gather(vec![node_type_document(
        "types.toml",
        vec![
            node_type("sink", &[("input", "note")], "note"),
            node_type("differ", &[], "diff"),
        ],
    )])
    .expect("two distinct types gather");

    let (definition, _) = read_workflow("broken.toml", text, &catalogue).expect("it reads");

    // The point of the case: all five reach the validator together. A reader
    // that looked one name up would report that one alone.
    assert_eq!(
        validate_wiring(&definition),
        vec![
            WiringDefect::UnresolvedNodeType {
                instance: "b".to_owned(),
                unresolved: "not-declared".to_owned(),
            },
            WiringDefect::UnresolvedInstance {
                instance: "c".to_owned(),
                parameter: "input".to_owned(),
                unresolved: "deleted".to_owned(),
            },
            WiringDefect::ContextTypeDisagreement {
                instance: "d".to_owned(),
                parameter: "input".to_owned(),
                expected: context_type("note"),
                produced: context_type("diff"),
            },
            WiringDefect::UndeclaredParameter {
                instance: "f".to_owned(),
                parameter: "nonesuch".to_owned(),
            },
            WiringDefect::SignatureOutputs {
                definition: "broken".to_owned(),
                designated: 0,
            },
        ]
    );

    // The one wiring defect that cannot share a document with those: an output
    // naming no instance, where the first document has no output at all.
    let text = "\
name = \"renamed\"
output = \"nowhere\"

[instances.a]
node_type = \"differ\"
";
    let (definition, _) = read_workflow("renamed.toml", text, &catalogue).expect("it reads");
    assert_eq!(
        validate_wiring(&definition),
        vec![WiringDefect::UnresolvedOutput {
            definition: "renamed".to_owned(),
            unresolved: "nowhere".to_owned(),
        }]
    );
}

#[test]
fn parameter_declared_twice_is_a_fault() {
    let key = |parts: &[&str]| parts.iter().map(|&part| part.to_owned()).collect();
    let cases = [
        // The required list first, inline: the repetition is in the optional
        // one, after a parameter declared once.
        (
            "required-first.toml",
            "[types.review]\noutput = \"note\"\nrequired = { input = \"note\" }\noptional = { hint = \"note\", input = \"diff\" }\n",
            (4, 29),
            key(&["types", "review", "optional", "input"]),
        ),
        // The optional list first, as header tables: now the repetition is in
        // the required one, so a reader always pointing at one list is wrong in
        // one of these two.
        (
            "optional-first.toml",
            "[types.review]\noutput = \"note\"\n\n[types.review.optional]\ninput = \"diff\"\n\n[types.review.required]\ninput = \"note\"\n",
            (8, 1),
            key(&["types", "review", "required", "input"]),
        ),
        // Two names in both lists, in opposite orders: a repeats first in the
        // text, though b comes first in the optional list.
        (
            "two-names.toml",
            "[types.t]\noutput = \"note\"\noptional = { b = \"note\", a = \"note\" }\nrequired = { a = \"note\", b = \"note\" }\n",
            (4, 14),
            key(&["types", "t", "required", "a"]),
        ),
    ];

    for (document, text, (line, column), key) in cases {
        let fault = read_node_types(document, text).expect_err("it is refused");
        assert_eq!(
            (fault.document(), fault.line(), fault.column(), fault.kind()),
            (
                document,
                line,
                column,
                &FaultKind::ParameterDeclaredTwice { key }
            ),
            "{fault}"
        );
    }

    // Names differing only in case are two parameters. Comparing them loosely
    // is the tidy-looking way to refuse the shape above, and it refuses this.
    let read = read_node_types(
        "cases.toml",
        "[types.review]\noutput = \"note\"\nrequired = { input = \"note\" }\noptional = { Input = \"note\" }\n",
    )
    .expect("two parameters read");
    assert_eq!(
        read.node_types(),
        [node_type("review", &[("input", "note")], "note").with_optional(&[("Input", "note")])]
    );
}

#[cfg(test)]
proptest! {
    /// Documents naming types only some of which the catalogue declares,
    /// binding parameters and sources only some of which exist, and designating
    /// an output that may be no instance at all, read as written.
    ///
    /// The pools hold a binding to a parameter its type does not declare and an
    /// output naming no instance, since those are names too, and a reader
    /// resolving them would refuse a document for a defect the validator
    /// reports beside every other.
    #[test]
    fn reading_resolves_nothing(written in any_written(Pools {
        types: &["source", "sink", "not-declared"],
        parameters: &["input", "nonesuch"],
        strangers: &["deleted", "nowhere"],
    })) {
        let catalogue = catalogue();
        let read = read_workflow("unresolved.toml", &written.text(), &catalogue);
        prop_assert_eq!(
            read.map(|(definition, _)| definition),
            Ok(written.definition(&catalogue))
        );
    }
}

#[test]
fn calls_read_in_order() {
    // Three calls, one listed twice, on one instance; none on another; and a
    // calls key on a node type document, where it is a key the reader does not
    // name. The workflow's calls name a type the catalogue lacks as well, since
    // resolving is the validator's.
    let types = read_node_types(
        "types.toml",
        "[types.lookup]\noutput = \"note\"\ncalls = [\"sink\"]\ndescription = \" Looks a term up. \"\n\n[types.bare]\noutput = \"note\"\n",
    )
    .expect("a node type with an unread key reads");
    // The description as written, spaces and all; none is empty.
    assert_eq!(
        types.node_types(),
        &[
            node_type("lookup", &[], "note").described(" Looks a term up. "),
            node_type("bare", &[], "note"),
        ]
    );

    let text = r#"name = "w"
output = "b"

[instances.a]
node_type = "source"
calls = ["sink", "lookup", "sink", "nowhere"]

[instances.b]
node_type = "sink"
bindings = { input = "a" }
"#;
    let (definition, _) = read_workflow("w.toml", text, &catalogue()).expect("it reads");

    assert_eq!(
        definition.instances[0].calls,
        ["sink", "lookup", "sink", "nowhere"],
        "in the order written, the repeat kept"
    );
    assert!(definition.instances[1].calls.is_empty(), "no key, no calls");
}

#[test]
fn malformed_calls_located() {
    // Each is a fault at the value that is wrong - a calls key that is not an
    // array, and an array holding something other than a string - rather than
    // an instance read as calling nothing.
    let cases = [
        ("calls = \"sink\"", (4, 9), "array", "string"),
        ("calls = [\"sink\", 7]", (4, 18), "string", "integer"),
        ("calls = { sink = true }", (4, 9), "array", "inline table"),
    ];

    for (line, place, expected, found) in cases {
        let text = format!("name = \"w\"\n\n[instances.a]\n{line}\nnode_type = \"source\"\n");
        let fault = read_workflow("calls.toml", &text, &catalogue())
            .expect_err("a malformed calls key is refused");
        assert_eq!(
            (fault.document(), fault.line(), fault.column()),
            ("calls.toml", place.0, place.1),
            "{fault}"
        );
        assert_eq!(
            fault.kind(),
            &FaultKind::WrongType {
                key: vec!["instances".to_owned(), "a".to_owned(), "calls".to_owned()],
                expected,
                found,
            },
            "{line}"
        );
    }

    // A call to a node type nobody declares is not a fault in the text.
    let text = "name = \"w\"\n[instances.a]\nnode_type = \"source\"\ncalls = [\"nowhere\"]\n";
    assert!(read_workflow("calls.toml", text, &catalogue()).is_ok());
}
