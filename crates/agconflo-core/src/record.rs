//! The run record: a run written down as text while it is under way, and a run
//! taken up again from the text in another process.
//!
//! A record is a TOML document holding the run's budget, the activations it
//! spent, the position of the identifier source its contexts came from, its
//! arguments, every event the run accepted in the order it accepted them, and
//! every context those hold, once each, in a table keyed by identifier - a
//! composition naming its parts. Every number is a decimal string. A run is
//! resumed by reporting each recorded event to it in turn, and nothing here
//! opens a file.

use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

use toml_edit::{ArrayOfTables, Document, DocumentMut, InlineTable, Item, Table, TableLike, value};

use crate::context::{Context, ContextType, InvalidTypeName};
use crate::id::{ContextId, IdSource};
use crate::reader::line_and_column;
use crate::run::{
    Arguments, Call, CallRefusal, Event, Exchange, OutputRefusal, Run, StartRefusal, Step,
};
use crate::workflow::WorkflowDefinition;

/// The version of the record this module writes, and the only one it reads.
// @Records at version 2,TRACE_RECORD_VERSION,trace,[],[DEC_RECORD_HOLDS_EXCHANGES, DEC_RECORD_IN_TOML, DEC_RECORD_IN_CORE]
const VERSION: &str = "2";

/// What a record says of a source that has issued every identifier it has.
const EXHAUSTED: &str = "exhausted";

/// The keys a record holds at its top, and nothing else.
const TOP: [&str; 7] = [
    "version", "budget", "spent", "source", "argument", "event", "context",
];

/// Why a record was not resumed, asked in the order a caller fixes them.
// @Refusals asked in the order they are fixed,TRACE_RECORD_REFUSAL,trace,[],[NOTE_RECORD_REFUSAL_ORDER, DEC_FAILURES_NON_EXHAUSTIVE]
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResumeRefusal {
    /// The text is not a record this version writes.
    Unreadable(RecordFault),
    /// A run of the workflow with the recorded arguments would not start, and
    /// this is the refusal its start gives.
    Start(StartRefusal),
    /// The workflow would not have produced this record: the first recorded
    /// output it disagrees with.
    Diverged {
        /// Where that output stands among the recorded ones, the first being 0.
        output: usize,
        /// The instance the record says produced it.
        instance: String,
        /// How the workflow disagrees.
        divergence: Divergence,
    },
    /// A recorded call the workflow would not accept from the activation that
    /// made it, and the run's refusal of it.
    CallRefused {
        /// The instance whose activation the record says made it.
        instance: String,
        /// The call, by the provider's identifier the record holds.
        call: String,
        /// Why the run refuses it.
        refusal: CallRefusal,
    },
    /// The activations recorded as spent are not what a run with the recorded
    /// events could have spent: fewer than its outputs, or more than the
    /// activations it would have outstanding afterwards - the one performing,
    /// and the one waiting on its call - where the workflow would offer
    /// nothing more.
    SpentDisagrees {
        /// The activations the record says were spent.
        spent: usize,
        /// The outputs it holds.
        outputs: usize,
    },
}

/// How a workflow disagrees with one recorded output.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Divergence {
    /// The run offered another instance at that point.
    OtherOffered {
        /// The instance it offered.
        offered: String,
    },
    /// The run offered nothing at that point, having ended.
    NothingOffered,
    /// The run offered the instance with other inputs: a workflow bound
    /// differently. Each list is a parameter and the identifier of its context,
    /// ordered by parameter.
    InputsDiffer {
        /// What the record says the activation was given.
        recorded: Vec<(String, ContextId)>,
        /// What the run would give it.
        offered: Vec<(String, ContextId)>,
    },
    /// The run refused the recorded output, for the reason it gives.
    Refused(OutputRefusal),
}

/// Text that cannot be resumed as a run's record, and where in it: a line and
/// a column counted from one, the column in characters, as the topology reader
/// counts them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordFault {
    line: usize,
    column: usize,
    kind: RecordFaultKind,
}

impl RecordFault {
    /// The line of the fault, counting the first line as 1.
    pub fn line(&self) -> usize {
        self.line
    }

    /// The column of the fault in characters, counting the first as 1.
    pub fn column(&self) -> usize {
        self.column
    }

    /// What is wrong at that place.
    pub fn kind(&self) -> &RecordFaultKind {
        &self.kind
    }
}

/// What makes a text not a record.
///
/// Every key is given from the top of the record down, an entry of an array
/// named by its place in it counting from 0.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RecordFaultKind {
    /// The text is not TOML.
    Syntax {
        /// The parser's own account of the fault, without its place.
        message: String,
    },
    /// The record is of another version, or says of none.
    Version {
        /// The version it says it is.
        found: String,
    },
    /// A key a record holds is absent.
    MissingKey {
        /// The absent key.
        key: Vec<String>,
    },
    /// A key no record holds, refused rather than passed over.
    UnexpectedKey {
        /// The key.
        key: Vec<String>,
    },
    /// A value is not of the kind its key holds.
    WrongType {
        /// The key holding the value.
        key: Vec<String>,
        /// The kind that key holds, as toml_edit names kinds.
        expected: &'static str,
        /// The kind it holds instead.
        found: &'static str,
    },
    /// A number not written as the decimal a record writes: digits alone,
    /// without a leading zero, within `u64`. Identifiers, the budget, the count
    /// and the source's position are all numbers.
    NotANumber {
        /// The key holding it, or naming it when the number is the key itself.
        key: Vec<String>,
        /// What is written there.
        found: String,
    },
    /// A context type name the model refuses.
    InvalidContextType {
        /// The key holding the name.
        key: Vec<String>,
        /// Why the model refuses it.
        reason: InvalidTypeName,
    },
    /// An identifier naming no context the record holds.
    UnknownContext {
        /// The key naming it.
        key: Vec<String>,
        /// The identifier.
        id: u64,
    },
    /// A composition holding itself, directly or through its parts, which no
    /// context can: a part exists before what is composed of it.
    ComposedOfItself {
        /// The context table where the circle was found closing.
        key: Vec<String>,
    },
    /// An identifier at or past the recorded source's position, which that
    /// source could not have issued.
    NotIssued {
        /// The context table carrying it.
        key: Vec<String>,
    },
    /// A context nothing the run holds reaches: no argument, output or part of
    /// either. A record holds what its run held and nothing besides.
    HeldByNothing {
        /// The context table.
        key: Vec<String>,
    },
}

impl fmt::Display for ResumeRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(fault) => write!(f, "not a run record: {fault}"),
            Self::Start(refusal) => refusal.fmt(f),
            Self::Diverged {
                output,
                instance,
                divergence,
            } => write!(
                f,
                "recorded output {output}, of {instance}, is not one the workflow would have produced: {divergence}"
            ),
            Self::CallRefused {
                instance,
                call,
                refusal,
            } => write!(
                f,
                "recorded call {call}, of {instance}, is not one the workflow would accept: {refusal}"
            ),
            Self::SpentDisagrees { spent, outputs } => write!(
                f,
                "{spent} activations recorded as spent for {outputs} outputs, which no run of the workflow could have spent"
            ),
        }
    }
}

impl std::error::Error for ResumeRefusal {}

impl fmt::Display for Divergence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OtherOffered { offered } => write!(f, "the run offers {offered} there"),
            Self::NothingOffered => f.write_str("the run offers nothing there"),
            Self::InputsDiffer { recorded, offered } => {
                write!(f, "it was given {recorded:?} and the run gives {offered:?}")
            }
            Self::Refused(refusal) => write!(f, "the run refuses it: {refusal}"),
        }
    }
}

impl fmt::Display for RecordFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.kind)
    }
}

impl fmt::Display for RecordFaultKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { message } => write!(f, "not TOML: {message}"),
            Self::Version { found } => {
                write!(f, "version '{found}', where this reads version {VERSION}")
            }
            Self::MissingKey { key } => write!(f, "'{}' is missing", key.join(".")),
            Self::UnexpectedKey { key } => write!(f, "'{}' is not in a record", key.join(".")),
            Self::WrongType {
                key,
                expected,
                found,
            } => write!(
                f,
                "'{}' is of type {found}, where type {expected} is expected",
                key.join(".")
            ),
            Self::NotANumber { key, found } => write!(
                f,
                "'{}' holds '{found}', which is not a number as a record writes one",
                key.join(".")
            ),
            Self::InvalidContextType { key, reason } => {
                write!(f, "'{}' is not a context type: {reason}", key.join("."))
            }
            Self::UnknownContext { key, id } => {
                write!(
                    f,
                    "'{}' names context {id}, which the record does not hold",
                    key.join(".")
                )
            }
            Self::ComposedOfItself { key } => {
                write!(f, "'{}' is composed of itself", key.join("."))
            }
            Self::NotIssued { key } => write!(
                f,
                "'{}' is an identifier the recorded source had not issued",
                key.join(".")
            ),
            Self::HeldByNothing { key } => {
                write!(f, "'{}' is held by nothing the run holds", key.join("."))
            }
        }
    }
}

impl std::error::Error for RecordFault {}

impl<'a, F> Run<'a, F> {
    /// This run written down as text, whole, with the position of `source`, which
    /// must be the source its contexts are being made from. Contexts are written
    /// in the order of their identifiers.
    // @A run written down whole,IMPL_RECORD_WRITE,impl,[CREQ_RECORD_HOLDS_THE_RUN],[DEC_RECORD_WRITTEN_WHOLE]
    pub fn record(&self, source: &IdSource) -> String {
        let state = self.recorded();
        let mut record = DocumentMut::new();
        record["version"] = value(VERSION);
        record["budget"] = value(state.budget.to_string());
        record["spent"] = value(state.spent.to_string());
        record["source"] = value(match source.position() {
            Some(next) => next.to_string(),
            None => EXHAUSTED.to_owned(),
        });

        let mut arguments = ArrayOfTables::new();
        for (instance, parameter, context) in state.arguments.iter() {
            let mut argument = Table::new();
            argument["instance"] = value(instance);
            argument["parameter"] = value(parameter);
            argument["context"] = value(context.id().value().to_string());
            arguments.push(argument);
        }
        record["argument"] = Item::ArrayOfTables(arguments);

        let mut events = ArrayOfTables::new();
        for event in state.log {
            events.push(written_event(event));
        }
        record["event"] = Item::ArrayOfTables(events);

        let held: HashMap<ContextId, &Context> = state
            .held
            .iter()
            .flat_map(|contexts| contexts.iter())
            .map(|(id, context)| (*id, context))
            .collect();
        let mut ids: Vec<ContextId> = held.keys().copied().collect();
        ids.sort_by_key(|id| id.value());
        let mut contexts = Table::new();
        contexts.set_implicit(true);
        for id in ids {
            let context = held[&id];
            let mut written = Table::new();
            written["type"] = value(context.declared_type().as_str());
            match context.separator() {
                None => written["text"] = value(context.render().as_ref()),
                Some(separator) => {
                    let parts: toml_edit::Array = context
                        .parts()
                        .iter()
                        .map(|part| part.id().value().to_string())
                        .collect();
                    written["parts"] = value(parts);
                    written["separator"] = value(separator);
                }
            }
            contexts.insert(&id.value().to_string(), Item::Table(written));
        }
        record["context"] = Item::Table(contexts);

        record.to_string()
    }

    /// The run `record` is a record of, resumed against `definition`, together
    /// with a source standing where the recorded one stood - or why it cannot
    /// be.
    ///
    /// The run is started from the recorded arguments and budget and handed
    /// each recorded event in turn, in the activation the record says it
    /// happened in: an output, which the run must offer the recorded activation
    /// with the recorded inputs for and accept; a call, which it must accept
    /// from that activation; and an exchange, which it holds with the
    /// activation. Then whatever was outstanding when the record was taken is
    /// offered again, counted once. Nothing is performed.
    // @A record replayed through the run,IMPL_RECORD_RESUME,impl,[CREQ_RECORD_CONTINUES_THE_RUN, CREQ_RECORD_REFUSES_DIVERGENCE, CREQ_RECORD_REFUSES_WHAT_START_REFUSES, CREQ_RECORD_KEEPS_EXCHANGES, CREQ_RECORD_REFUSES_UNDECLARED_CALL],[DEC_RESUME_REPLAYS_CALLS, DEC_RECORD_CARRIES_THE_SOURCE]
    pub fn resume(
        definition: &'a WorkflowDefinition,
        record: &str,
    ) -> Result<(Self, IdSource), ResumeRefusal> {
        let read = Reading::new(record)
            .read()
            .map_err(ResumeRefusal::Unreadable)?;

        let outputs = read
            .events
            .iter()
            .filter(|event| matches!(event.what, Happened::Output { .. }))
            .count();
        let disagrees = ResumeRefusal::SpentDisagrees {
            spent: read.spent,
            outputs,
        };
        if read.spent < outputs {
            return Err(disagrees);
        }

        let mut arguments = Arguments::new();
        for (instance, parameter, id) in &read.arguments {
            arguments = arguments.supply(instance, parameter, read.contexts[id].clone());
        }
        let mut run =
            Run::start(definition, arguments, read.budget).map_err(ResumeRefusal::Start)?;

        let mut replayed = 0;
        for recorded in &read.events {
            let diverged = |divergence| ResumeRefusal::Diverged {
                output: replayed,
                instance: recorded.instance.clone(),
                divergence,
            };
            let activation = run
                .reached(&recorded.instance, recorded.performing.as_deref())
                .map_err(diverged)?;

            match &recorded.what {
                Happened::Output { context, inputs } => {
                    let mut offered: Vec<(String, ContextId)> = activation
                        .inputs()
                        .iter()
                        .map(|(parameter, context)| (parameter.clone(), context.id()))
                        .collect();
                    offered.sort_by(|a, b| a.0.cmp(&b.0));
                    if offered != *inputs {
                        return Err(diverged(Divergence::InputsDiffer {
                            recorded: inputs.clone(),
                            offered,
                        }));
                    }
                    run.produced(read.contexts[context].clone())
                        .map_err(|refusal| diverged(Divergence::Refused(refusal)))?;
                    replayed += 1;
                }
                Happened::Call(call) => {
                    run.call(call.clone())
                        .map_err(|refusal| ResumeRefusal::CallRefused {
                            instance: recorded.instance.clone(),
                            call: call.id().to_owned(),
                            refusal,
                        })?;
                }
                Happened::Exchange(exchange) => {
                    if let Err(refusal) = run.exchange(exchange.clone()) {
                        // An activation is outstanding - it was just reached -
                        // and every context of a record is the one context under
                        // its identifier, so the run has nothing to refuse.
                        unreachable!("a record's exchange is refused: {refusal}");
                    }
                }
            }
        }

        // Whatever was outstanding when the record was taken is offered again,
        // counted once as it was: the activation performing, and the call it was
        // waiting on. Each step has to offer something new.
        while run.recorded().spent < read.spent {
            let before = run.recorded().spent;
            if !matches!(run.step(), Step::Activate(_)) || run.recorded().spent == before {
                return Err(disagrees);
            }
        }
        if run.recorded().spent != read.spent {
            return Err(disagrees);
        }

        // Asked last, once every argument and event has been brought in.
        let held = run.recorded().held;
        let holds = |id: &ContextId| held.iter().any(|contexts| contexts.contains_key(id));
        if let Some(stray) = read.order.iter().find(|id| !holds(id)) {
            let key = context_key(*stray);
            return Err(ResumeRefusal::Unreadable(read.reading.fault(
                read.spans[stray].clone(),
                RecordFaultKind::HeldByNothing { key },
            )));
        }

        Ok((run, IdSource::resumed_at(read.source)))
    }
}

impl<F> Run<'_, F> {
    /// The activation for `instance` performing `performing` - outstanding
    /// already, or offered by the next step - or how the run disagrees.
    fn reached(
        &mut self,
        instance: &str,
        performing: Option<&str>,
    ) -> Result<crate::Activation, Divergence> {
        let is_it = |activation: &crate::Activation| {
            activation.instance() == instance && activation.call() == performing
        };
        if let Some(outstanding) = self.outstanding() {
            return if is_it(outstanding) {
                Ok(outstanding.clone())
            } else {
                Err(Divergence::OtherOffered {
                    offered: outstanding.instance().to_owned(),
                })
            };
        }
        match self.step() {
            Step::Activate(activation) if is_it(&activation) => Ok(activation),
            Step::Activate(activation) => Err(Divergence::OtherOffered {
                offered: activation.instance().to_owned(),
            }),
            Step::Ended(_) => Err(Divergence::NothingOffered),
        }
    }
}

/// One event as a record writes it: the instance its activation is for, the call
/// that activation performs when it is a call's, and what happened - an output
/// with its activation's inputs, an exchange, or a call - every context by
/// identifier, a window included, and a call's identifier as the provider issued
/// it.
// @Exchanges and calls and outputs written in the order accepted,IMPL_RECORD_EVENTS,impl,[CREQ_RECORD_HOLDS_EXCHANGES, CREQ_RECORD_HOLDS_THE_RUN],[NOTE_RUN_ONE_EVENT_LIST]
fn written_event(event: &Event) -> Table {
    let id = |context: &Context| context.id().value().to_string();
    let inputs = |given: &[(String, Context)]| {
        let mut inputs = InlineTable::new();
        for (parameter, context) in given {
            inputs.insert(parameter, id(context).into());
        }
        inputs
    };

    let mut entry = Table::new();
    match event {
        Event::Output { activation, output } => {
            entry["instance"] = value(activation.instance());
            if let Some(call) = activation.call() {
                entry["performing"] = value(call);
            }
            entry["output"] = value(id(output));
            entry["inputs"] = value(inputs(activation.inputs()));
        }
        Event::Exchange {
            instance,
            performing,
            exchange,
        } => {
            entry["instance"] = value(instance);
            if let Some(call) = performing {
                entry["performing"] = value(call);
            }
            let mut written = InlineTable::new();
            let offer: toml_edit::Array = exchange.offer().iter().map(id).collect();
            written.insert("offer", offer.into());
            written.insert("window", id(exchange.window()).into());
            written.insert("answer", id(exchange.answer()).into());
            let mut calls = toml_edit::Array::new();
            for call in exchange.calls() {
                calls.push(written_call(call));
            }
            written.insert("calls", calls.into());
            entry["exchange"] = value(written);
        }
        Event::Call { instance, call } => {
            entry["instance"] = value(instance);
            entry["call"] = value(written_call(call));
        }
    }
    entry
}

/// One call as a record writes it: the provider's identifier, the node type and
/// each parameter's context by identifier.
fn written_call(call: &Call) -> InlineTable {
    let mut inputs = InlineTable::new();
    for (parameter, context) in call.inputs() {
        inputs.insert(parameter, context.id().value().to_string().into());
    }
    let mut written = InlineTable::new();
    written.insert("id", call.id().into());
    written.insert("node_type", call.node_type().into());
    written.insert("inputs", inputs.into());
    written
}

/// A record's content once read: its values, and every context it holds made.
struct Read<'t> {
    reading: Reading<'t>,
    budget: usize,
    spent: usize,
    source: Option<u64>,
    arguments: Vec<(String, String, ContextId)>,
    events: Vec<Recorded>,
    contexts: HashMap<ContextId, Context>,
    /// The contexts' identifiers in the order the record writes them, so that a
    /// fault found among several names the first.
    order: Vec<ContextId>,
    /// Where each context's table is written.
    spans: HashMap<ContextId, Option<Range<usize>>>,
}

/// How a record's reader turns an item naming a context into its identifier,
/// refusing one the record does not hold.
type Known<'k> = dyn Fn(&Item, &[&str]) -> Result<ContextId, RecordFault> + 'k;

/// One recorded event: the activation it belongs to - the instance it is for, and
/// the call it performs when it is a call's - and what happened.
struct Recorded {
    instance: String,
    performing: Option<String>,
    what: Happened,
}

/// What one recorded event says happened.
enum Happened {
    /// An output, and its activation's inputs ordered by parameter.
    Output {
        context: ContextId,
        inputs: Vec<(String, ContextId)>,
    },
    /// An exchange, its contexts made.
    Exchange(Exchange),
    /// A call, its contexts made.
    Call(Call),
}

/// One context as the record writes it, before it is made.
enum Written {
    Text(String),
    Composed {
        parts: Vec<(u64, Option<Range<usize>>)>,
        separator: String,
    },
}

/// Where a context stands while the contexts are made.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Making {
    Started,
    Made,
}

/// The key of a context's table.
fn context_key(id: ContextId) -> Vec<String> {
    vec!["context".to_owned(), id.value().to_string()]
}

/// `text` as the number a record writes: digits alone, without a leading zero,
/// within `u64`.
fn number(text: &str) -> Option<u64> {
    let canonical = text == "0"
        || (text.starts_with(|c: char| ('1'..='9').contains(&c))
            && text.bytes().all(|b| b.is_ascii_digit()));
    canonical.then(|| text.parse().ok()).flatten()
}

/// One record being read, and the text every fault in it is located against.
#[derive(Clone, Copy)]
struct Reading<'t> {
    text: &'t str,
}

impl<'t> Reading<'t> {
    fn new(text: &'t str) -> Self {
        Self { text }
    }

    fn fault(&self, span: Option<Range<usize>>, kind: RecordFaultKind) -> RecordFault {
        let (line, column) = line_and_column(self.text, span.map_or(0, |span| span.start));
        RecordFault { line, column, kind }
    }

    /// Everything the record holds, every context made, or the first fault.
    // @A record read strictly and located,IMPL_RECORD_READ,impl,[CREQ_RECORD_REFUSES_UNREADABLE]
    fn read(self) -> Result<Read<'t>, RecordFault> {
        let parsed = Document::parse(self.text).map_err(|refused| {
            self.fault(
                refused.span(),
                RecordFaultKind::Syntax {
                    message: refused.message().to_owned(),
                },
            )
        })?;
        let top = parsed.as_table();

        // The version first.
        let version = self.needed(top, None, &["version"])?;
        let written = self.string(version, &["version"])?;
        if written != VERSION {
            return Err(self.fault(
                version.span(),
                RecordFaultKind::Version {
                    found: written.to_owned(),
                },
            ));
        }
        self.only(top, &[], &TOP)?;

        let budget = self.count(self.needed(top, None, &["budget"])?, "budget")?;
        let spent = self.count(self.needed(top, None, &["spent"])?, "spent")?;
        let source = self.needed(top, None, &["source"])?;
        let source = match self.string(source, &["source"])? {
            EXHAUSTED => None,
            _ => Some(self.number(source, &["source"])?),
        };

        let (contexts, order, spans) = self.contexts(top.get("context"), source)?;
        let known = |item: &Item, key: &[&str]| -> Result<ContextId, RecordFault> {
            let id = ContextId::resumed(self.number(item, key)?);
            if contexts.contains_key(&id) {
                Ok(id)
            } else {
                Err(self.fault(
                    item.span(),
                    RecordFaultKind::UnknownContext {
                        key: owned(key),
                        id: id.value(),
                    },
                ))
            }
        };

        let mut arguments = Vec::new();
        for (index, argument) in self.entries(top.get("argument"), "argument")? {
            let at = index.to_string();
            let place = argument.span();
            self.only(
                argument,
                &["argument", &at],
                &["instance", "parameter", "context"],
            )?;
            let instance = self.needed(argument, place.clone(), &["argument", &at, "instance"])?;
            let parameter =
                self.needed(argument, place.clone(), &["argument", &at, "parameter"])?;
            let context = self.needed(argument, place, &["argument", &at, "context"])?;
            arguments.push((
                self.string(instance, &["argument", &at, "instance"])?
                    .to_owned(),
                self.string(parameter, &["argument", &at, "parameter"])?
                    .to_owned(),
                known(context, &["argument", &at, "context"])?,
            ));
        }

        let mut events = Vec::new();
        for (index, event) in self.entries(top.get("event"), "event")? {
            events.push(self.event(index, event, &known, &contexts)?);
        }

        Ok(Read {
            reading: self,
            budget,
            spent,
            source,
            arguments,
            events,
            contexts,
            order,
            spans,
        })
    }

    /// One `[[event]]` entry: the instance, the call it performs if any, and
    /// exactly one of an output with its inputs, an exchange, or a call.
    fn event(
        &self,
        index: usize,
        event: &Table,
        known: &Known<'_>,
        contexts: &HashMap<ContextId, Context>,
    ) -> Result<Recorded, RecordFault> {
        let at = index.to_string();
        let at = at.as_str();
        let place = event.span();
        let instance = self.needed(event, place.clone(), &["event", at, "instance"])?;
        let instance = self
            .string(instance, &["event", at, "instance"])?
            .to_owned();
        let performing = match event.get("performing") {
            None => None,
            Some(item) => Some(self.string(item, &["event", at, "performing"])?.to_owned()),
        };
        let inputs_of =
            |item: &Item, key: &[&str]| -> Result<Vec<(String, ContextId)>, RecordFault> {
                let mut inputs = Vec::new();
                for (parameter, given) in self.table(item, key)?.iter() {
                    let mut key = key.to_vec();
                    key.push(parameter);
                    inputs.push((parameter.to_owned(), known(given, &key)?));
                }
                Ok(inputs)
            };

        let what = if let Some(output) = event.get("output") {
            self.only(
                event,
                &["event", at],
                &["instance", "performing", "output", "inputs"],
            )?;
            let written = self.needed(event, place, &["event", at, "inputs"])?;
            let mut inputs = inputs_of(written, &["event", at, "inputs"])?;
            inputs.sort_by(|a, b| a.0.cmp(&b.0));
            Happened::Output {
                context: known(output, &["event", at, "output"])?,
                inputs,
            }
        } else if let Some(written) = event.get("exchange") {
            self.only(
                event,
                &["event", at],
                &["instance", "performing", "exchange"],
            )?;
            let key = ["event", at, "exchange"];
            let fields = self.table(written, &key)?;
            self.only(fields, &key, &["offer", "window", "answer", "calls"])?;
            let window =
                self.needed(fields, written.span(), &["event", at, "exchange", "window"])?;
            let answer =
                self.needed(fields, written.span(), &["event", at, "exchange", "answer"])?;
            let mut exchange = Exchange::new(
                contexts[&known(window, &["event", at, "exchange", "window"])?].clone(),
                contexts[&known(answer, &["event", at, "exchange", "answer"])?].clone(),
            );
            let mut offer = Vec::new();
            for (position, item) in
                self.strings(fields.get("offer"), &["event", at, "exchange", "offer"])?
            {
                let position = position.to_string();
                let key = ["event", at, "exchange", "offer", position.as_str()];
                offer.push(contexts[&known(&item, &key)?].clone());
            }
            exchange = exchange.offering(offer);
            for (position, item) in
                self.strings(fields.get("calls"), &["event", at, "exchange", "calls"])?
            {
                let position = position.to_string();
                let key = ["event", at, "exchange", "calls", position.as_str()];
                exchange = exchange.calling(self.call(&item, &key, known, contexts)?);
            }
            Happened::Exchange(exchange)
        } else if let Some(written) = event.get("call") {
            self.only(event, &["event", at], &["instance", "performing", "call"])?;
            Happened::Call(self.call(written, &["event", at, "call"], known, contexts)?)
        } else {
            return Err(self.fault(
                place,
                RecordFaultKind::MissingKey {
                    key: owned(&["event", at, "output"]),
                },
            ));
        };

        Ok(Recorded {
            instance,
            performing,
            what,
        })
    }

    /// One call written under `key`: its identifier, node type and inputs.
    fn call(
        &self,
        written: &Item,
        key: &[&str],
        known: &Known<'_>,
        contexts: &HashMap<ContextId, Context>,
    ) -> Result<Call, RecordFault> {
        let fields = self.table(written, key)?;
        self.only(fields, key, &["id", "node_type", "inputs"])?;
        let under = |name: &'static str| [key, &[name]].concat();
        let id = self.needed(fields, written.span(), &under("id"))?;
        let node_type = self.needed(fields, written.span(), &under("node_type"))?;
        let inputs = self.needed(fields, written.span(), &under("inputs"))?;
        let mut call = Call::new(
            self.string(id, &under("id"))?,
            self.string(node_type, &under("node_type"))?,
        );
        let inputs_key = under("inputs");
        for (parameter, given) in self.table(inputs, &inputs_key)?.iter() {
            let mut at = inputs_key.clone();
            at.push(parameter);
            call = call.input(parameter, contexts[&known(given, &at)?].clone());
        }
        Ok(call)
    }

    /// The items of an array the record may leave out, each with its place.
    fn strings(
        &self,
        item: Option<&Item>,
        key: &[&str],
    ) -> Result<Vec<(usize, Item)>, RecordFault> {
        let Some(item) = item else {
            return Ok(Vec::new());
        };
        let Some(array) = item.as_array() else {
            return Err(self.wrong_type(item.span(), key, "array", item.type_name()));
        };
        Ok(array
            .iter()
            .map(|value| Item::Value(value.clone()))
            .enumerate()
            .collect())
    }

    /// Every context the record holds, made - each once, however many hold it -
    /// with the order they are written in and where. A part is made before what
    /// holds it, with a stack of the module's own; a part still being made when
    /// it is reached again is a composition holding itself.
    #[allow(clippy::type_complexity)]
    // @Each context made once and without recursion,IMPL_RECORD_CONTEXTS,impl,[CREQ_RECORD_KEEPS_CONTEXTS, CREQ_RECORD_REFUSES_UNREADABLE],[NOTE_CONTEXT_NO_RECURSION]
    fn contexts(
        &self,
        table: Option<&Item>,
        source: Option<u64>,
    ) -> Result<
        (
            HashMap<ContextId, Context>,
            Vec<ContextId>,
            HashMap<ContextId, Option<Range<usize>>>,
        ),
        RecordFault,
    > {
        let mut written: HashMap<u64, (ContextType, Written)> = HashMap::new();
        let mut order = Vec::new();
        let mut spans = HashMap::new();

        if let Some(table) = table {
            let table = self.table(table, &["context"])?;
            for (key, entry) in table.iter() {
                let span = table.key(key).and_then(|written| written.span());
                let Some(id) = number(key) else {
                    return Err(self.fault(
                        span,
                        RecordFaultKind::NotANumber {
                            key: vec!["context".to_owned(), key.to_owned()],
                            found: key.to_owned(),
                        },
                    ));
                };
                let at = ["context", key];
                // @An identifier only beside a source past it,IMPL_RECORD_SOURCE,impl,[CREQ_RECORD_SOURCE_CONTINUES]
                if source.is_some_and(|next| id >= next) {
                    return Err(self.fault(span, RecordFaultKind::NotIssued { key: owned(&at) }));
                }
                let fields = self.table(entry, &at)?;
                let place = entry.span();
                let declared = self.needed(fields, place.clone(), &["context", key, "type"])?;
                let declared = self.string(declared, &["context", key, "type"])?;
                let declared = ContextType::new(declared).map_err(|reason| {
                    self.fault(
                        fields.get("type").and_then(Item::span),
                        RecordFaultKind::InvalidContextType {
                            key: owned(&["context", key, "type"]),
                            reason,
                        },
                    )
                })?;
                let content = if fields.contains_key("text") {
                    self.only(fields, &at, &["type", "text"])?;
                    let text = self.needed(fields, place, &["context", key, "text"])?;
                    Written::Text(self.string(text, &["context", key, "text"])?.to_owned())
                } else {
                    self.only(fields, &at, &["type", "parts", "separator"])?;
                    let separator =
                        self.needed(fields, place.clone(), &["context", key, "separator"])?;
                    let separator = self
                        .string(separator, &["context", key, "separator"])?
                        .to_owned();
                    let listed = self.needed(fields, place, &["context", key, "parts"])?;
                    let Some(listed) = listed.as_array() else {
                        return Err(self.wrong_type(
                            listed.span(),
                            &["context", key, "parts"],
                            "array",
                            listed.type_name(),
                        ));
                    };
                    let mut parts = Vec::new();
                    for (index, part) in listed.iter().enumerate() {
                        let index = index.to_string();
                        let part_key = ["context", key, "parts", index.as_str()];
                        let Some(text) = part.as_str() else {
                            return Err(self.wrong_type(
                                part.span(),
                                &part_key,
                                "string",
                                part.type_name(),
                            ));
                        };
                        let Some(part_id) = number(text) else {
                            return Err(self.fault(
                                part.span(),
                                RecordFaultKind::NotANumber {
                                    key: owned(&part_key),
                                    found: text.to_owned(),
                                },
                            ));
                        };
                        parts.push((part_id, part.span()));
                    }
                    Written::Composed { parts, separator }
                };
                written.insert(id, (declared, content));
                order.push(ContextId::resumed(id));
                spans.insert(ContextId::resumed(id), span);
            }
        }

        let mut made: HashMap<ContextId, Context> = HashMap::new();
        let mut making: HashMap<u64, Making> = HashMap::new();
        for root in &order {
            let mut stack = vec![(root.value(), false)];
            while let Some((id, parts_made)) = stack.pop() {
                let (declared, content) = &written[&id];
                let key = ["context".to_owned(), id.to_string()];
                match (content, parts_made) {
                    _ if making.get(&id) == Some(&Making::Made) => {}
                    (Written::Text(text), _) => {
                        let context = Context::resumed_text(
                            ContextId::resumed(id),
                            declared.clone(),
                            text.clone(),
                        );
                        made.insert(context.id(), context);
                        making.insert(id, Making::Made);
                    }
                    (Written::Composed { parts, .. }, false) => {
                        if making.insert(id, Making::Started).is_some() {
                            return Err(self.fault(
                                spans[&ContextId::resumed(id)].clone(),
                                RecordFaultKind::ComposedOfItself { key: key.to_vec() },
                            ));
                        }
                        stack.push((id, true));
                        for (index, (part, span)) in parts.iter().enumerate() {
                            if !written.contains_key(part) {
                                let mut part_key = key.to_vec();
                                part_key.extend(["parts".to_owned(), index.to_string()]);
                                return Err(self.fault(
                                    span.clone(),
                                    RecordFaultKind::UnknownContext {
                                        key: part_key,
                                        id: *part,
                                    },
                                ));
                            }
                            match making.get(part) {
                                Some(Making::Made) => {}
                                Some(Making::Started) => {
                                    return Err(self.fault(
                                        spans[&ContextId::resumed(id)].clone(),
                                        RecordFaultKind::ComposedOfItself { key: key.to_vec() },
                                    ));
                                }
                                None => stack.push((*part, false)),
                            }
                        }
                    }
                    (Written::Composed { parts, separator }, true) => {
                        let parts = parts
                            .iter()
                            .map(|(part, _)| made[&ContextId::resumed(*part)].clone())
                            .collect();
                        let context = Context::resumed_composed(
                            ContextId::resumed(id),
                            declared.clone(),
                            parts,
                            separator,
                        );
                        made.insert(context.id(), context);
                        making.insert(id, Making::Made);
                    }
                }
            }
        }

        Ok((made, order, spans))
    }

    /// The entries of an array of tables the record may leave out, each with
    /// its place in the array.
    fn entries<'i>(
        &self,
        item: Option<&'i Item>,
        key: &str,
    ) -> Result<Vec<(usize, &'i Table)>, RecordFault> {
        let Some(item) = item else {
            return Ok(Vec::new());
        };
        let Some(entries) = item.as_array_of_tables() else {
            return Err(self.wrong_type(item.span(), &[key], "array of tables", item.type_name()));
        };
        Ok(entries.iter().enumerate().collect())
    }

    /// Refuse the first key of `table` not among `allowed`.
    fn only(
        &self,
        table: &dyn TableLike,
        at: &[&str],
        allowed: &[&str],
    ) -> Result<(), RecordFault> {
        for (key, _) in table.iter() {
            if !allowed.contains(&key) {
                let mut full = owned(at);
                full.push(key.to_owned());
                return Err(self.fault(
                    table.key(key).and_then(|written| written.span()),
                    RecordFaultKind::UnexpectedKey { key: full },
                ));
            }
        }
        Ok(())
    }

    /// The value under the last part of `key` in `table`, or a fault naming
    /// it placed at `place` - the table's own place, where it has one.
    fn needed<'i>(
        &self,
        table: &'i dyn TableLike,
        place: Option<Range<usize>>,
        key: &[&str],
    ) -> Result<&'i Item, RecordFault> {
        let last = key.last().expect("a key has a last part");
        table
            .get(last)
            .ok_or_else(|| self.fault(place, RecordFaultKind::MissingKey { key: owned(key) }))
    }

    fn string<'i>(&self, item: &'i Item, key: &[&str]) -> Result<&'i str, RecordFault> {
        item.as_str()
            .ok_or_else(|| self.wrong_type(item.span(), key, "string", item.type_name()))
    }

    fn number(&self, item: &Item, key: &[&str]) -> Result<u64, RecordFault> {
        let text = self.string(item, key)?;
        number(text).ok_or_else(|| {
            self.fault(
                item.span(),
                RecordFaultKind::NotANumber {
                    key: owned(key),
                    found: text.to_owned(),
                },
            )
        })
    }

    fn count(&self, item: &Item, key: &str) -> Result<usize, RecordFault> {
        let value = self.number(item, &[key])?;
        usize::try_from(value).map_err(|_| {
            self.fault(
                item.span(),
                RecordFaultKind::NotANumber {
                    key: vec![key.to_owned()],
                    found: value.to_string(),
                },
            )
        })
    }

    fn table<'i>(&self, item: &'i Item, key: &[&str]) -> Result<&'i dyn TableLike, RecordFault> {
        item.as_table_like()
            .ok_or_else(|| self.wrong_type(item.span(), key, "table", item.type_name()))
    }

    fn wrong_type(
        &self,
        span: Option<Range<usize>>,
        key: &[&str],
        expected: &'static str,
        found: &'static str,
    ) -> RecordFault {
        self.fault(
            span,
            RecordFaultKind::WrongType {
                key: owned(key),
                expected,
                found,
            },
        )
    }
}

fn owned(key: &[&str]) -> Vec<String> {
    key.iter().map(|part| (*part).to_owned()).collect()
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases.

#[cfg(test)]
use crate::wiring::well_formed_definition;
#[cfg(test)]
use crate::workflow::{context_type, definition, instance, node_type};
#[cfg(test)]
use proptest::prelude::*;

/// A text context of type `note` holding `text`.
#[cfg(test)]
fn note(source: &mut IdSource, text: &str) -> Context {
    Context::text(source, context_type("note"), text).expect("a fresh source issues")
}

/// A chain of `count` instances: `n0` taking `input` from the run, each later one
/// bound to the one before, the last designated. Every type is `note`.
#[cfg(test)]
fn chain(count: usize) -> WorkflowDefinition {
    let types = vec![
        node_type("Src", &[("input", "note")], "note"),
        node_type("Step", &[("input", "note")], "note"),
    ];
    let mut instances = vec![instance("n0", "Src", &[])];
    for step in 1..count {
        instances.push(instance(
            &format!("n{step}"),
            "Step",
            &[("input", &format!("n{}", step - 1))],
        ));
    }
    let last = format!("n{}", count - 1);
    definition(types, instances, &[&last])
}

/// A context for `activation`: its first input composed with a word, twice -
/// so that a part is held by one composition in two places - or a word alone
/// when it is given nothing.
#[cfg(test)]
fn answer(source: &mut IdSource, activation: &crate::Activation) -> Context {
    let declared = activation.output().clone();
    let word = Context::text(source, declared.clone(), "w").expect("a fresh source issues");
    match activation.inputs().first() {
        Some((_, input)) => Context::compose(source, declared, [input, &word, input], " ")
            .expect("a fresh source issues"),
        None => word,
    }
}

/// How a run ended, as something two endings can be compared by.
#[cfg(test)]
fn ending(ending: &crate::RunEnding<()>) -> String {
    match ending {
        crate::RunEnding::Completed(result) => format!("completed {:?}", result.id()),
        crate::RunEnding::NodeFailed { instance, .. } => format!("failed {instance}"),
        crate::RunEnding::BudgetExceeded { budget } => format!("budget {budget}"),
        crate::RunEnding::Quiescent { waiting } => format!("quiescent {waiting:?}"),
    }
}

/// One activation as it was offered and answered: the instance, its inputs by
/// identifier, and the identifier of the output it was given.
#[cfg(test)]
type Answered = (String, Vec<(String, ContextId)>, ContextId);

/// Drive `run` to its ending, answering each activation with [`answer`], and
/// hand `each` a record before every step and again while an activation is
/// outstanding. Returns what was answered and how the run ended.
#[cfg(test)]
fn drive(
    run: &mut Run<'_, ()>,
    source: &mut IdSource,
    each: &mut dyn FnMut(String, usize),
) -> (Vec<Answered>, String) {
    let mut answered = Vec::new();
    loop {
        each(run.record(source), answered.len());
        match run.step() {
            Step::Ended(end) => return (answered, ending(&end)),
            Step::Activate(activation) => {
                each(run.record(source), answered.len());
                let output = answer(source, &activation);
                let inputs = activation
                    .inputs()
                    .iter()
                    .map(|(parameter, context)| (parameter.clone(), context.id()))
                    .collect();
                answered.push((activation.instance().to_owned(), inputs, output.id()));
                run.produced(output)
                    .expect("an answer of the declared type");
            }
        }
    }
}

/// The arguments filling every parameter nothing binds.
#[cfg(test)]
fn filled(workflow: &WorkflowDefinition, source: &mut IdSource) -> Arguments {
    let mut arguments = Arguments::new();
    for node in &workflow.instances {
        let declared = workflow
            .node_types
            .iter()
            .find(|declared| declared.name == node.node_type)
            .expect("a well-formed definition declares every type it names");
        for parameter in &declared.required {
            if node.bindings.iter().any(|b| b.parameter == parameter.name) {
                continue;
            }
            let supplied = Context::text(source, parameter.context_type.clone(), "x")
                .expect("a fresh source issues");
            arguments = arguments.supply(&node.name, &parameter.name, supplied);
        }
    }
    arguments
}

/// A record of `chain(3)` after `n0` and `n1` have produced, with a context
/// made and dropped first so that the source stands past every identifier
/// held, and the source it was taken with.
#[cfg(test)]
fn two_of_three(workflow: &WorkflowDefinition) -> (String, IdSource) {
    let mut source = IdSource::new();
    let argument = note(&mut source, "hello");
    let _dropped = note(&mut source, "dropped");
    let mut run = Run::<()>::start(
        workflow,
        Arguments::new().supply("n0", "input", argument),
        5,
    )
    .expect("a sound chain starts");
    for _ in 0..2 {
        let Step::Activate(activation) = run.step() else {
            panic!("a chain of three offers three")
        };
        let output = answer(&mut source, &activation);
        run.produced(output).expect("accepted");
    }
    (run.record(&source), source)
}

/// The fault a refusal carries, or a panic naming what came instead.
#[cfg(test)]
fn fault(resumed: Result<(Run<'_, ()>, IdSource), ResumeRefusal>) -> RecordFault {
    match resumed {
        Err(ResumeRefusal::Unreadable(fault)) => fault,
        Err(other) => panic!("expected an unreadable record, got {other:?}"),
        Ok(_) => panic!("expected an unreadable record, and it resumed"),
    }
}

#[cfg(test)]
#[test]
fn holds_the_run() {
    let workflow = chain(3);
    let mut source = IdSource::new();
    let argument = note(&mut source, "hello");
    let mut run = Run::<()>::start(
        &workflow,
        Arguments::new().supply("n0", "input", argument),
        7,
    )
    .expect("a sound chain starts");
    for _ in 0..2 {
        let Step::Activate(activation) = run.step() else {
            panic!("a chain of three offers three")
        };
        let output = answer(&mut source, &activation);
        run.produced(output).expect("accepted");
    }
    let Step::Activate(_) = run.step() else {
        panic!("the third is offered")
    };

    // Read back as TOML rather than resumed, so that what is asserted is what
    // the text holds and not what a reader makes of it.
    let text = run.record(&source);
    let record: DocumentMut = text.parse().expect("a record is TOML");
    assert_eq!(record["version"].as_str(), Some("2"));
    assert_eq!(record["budget"].as_str(), Some("7"));
    // Two outputs and one activation outstanding.
    assert_eq!(record["spent"].as_str(), Some("3"));
    assert_eq!(record["source"].as_str(), Some("5"));

    let arguments = record["argument"].as_array_of_tables().expect("arguments");
    assert_eq!(arguments.len(), 1);
    let argument = arguments.get(0).expect("one");
    assert_eq!(argument["instance"].as_str(), Some("n0"));
    assert_eq!(argument["parameter"].as_str(), Some("input"));
    assert_eq!(argument["context"].as_str(), Some("0"));

    let events = record["event"].as_array_of_tables().expect("events");
    let outputs: Vec<(&str, &str, &str)> = events
        .iter()
        .map(|event| {
            (
                event["instance"].as_str().expect("an instance"),
                event["output"].as_str().expect("an output"),
                event["inputs"]["input"].as_str().expect("an input"),
            )
        })
        .collect();
    assert_eq!(outputs, [("n0", "2", "0"), ("n1", "4", "2")]);

    // Every context the run holds, once each, keyed by identifier; a
    // composition names its parts - one of them twice - and holds none.
    let contexts = record["context"].as_table().expect("contexts");
    let keys: Vec<&str> = contexts.iter().map(|(key, _)| key).collect();
    assert_eq!(keys, ["0", "1", "2", "3", "4"]);
    let parts: Vec<Option<&str>> = contexts["4"]["parts"]
        .as_array()
        .expect("parts are listed")
        .iter()
        .map(|part| part.as_str())
        .collect();
    assert_eq!(parts, [Some("2"), Some("3"), Some("2")]);
    assert_eq!(contexts["4"]["separator"].as_str(), Some(" "));
    assert_eq!(contexts["0"]["text"].as_str(), Some("hello"));
    assert_eq!(text.matches("[context.2]").count(), 1);
}

#[cfg(test)]
proptest! {
    /// For any well-formed workflow and budget, a run recorded at any point
    /// and resumed offers what the uninterrupted run offered after that point,
    /// with the same inputs, and ends the same way.
    #[test]
    fn resumed_run_ends_alike(workflow in well_formed_definition(), budget in 0..6usize) {
        let mut source = IdSource::new();
        let arguments = filled(&workflow, &mut source);
        let mut run = Run::<()>::start(&workflow, arguments, budget).expect("sound");
        let mut records = Vec::new();
        let (answered, ended) = drive(&mut run, &mut source, &mut |record, done| records.push((record, done)));
        prop_assert!(!records.is_empty());

        for (record, done) in records {
            let (mut resumed, mut again) = Run::<()>::resume(&workflow, &record)
                .map_err(|refusal| TestCaseError::fail(format!("{refusal}\n{record}")))?;
            let (rest, resumed_end) = drive(&mut resumed, &mut again, &mut |_, _| {});
            prop_assert_eq!(&rest[..], &answered[done..], "after {} outputs", done);
            prop_assert_eq!(&resumed_end, &ended);
        }
    }
}

#[cfg(test)]
#[test]
fn outstanding_not_charged_again() {
    let workflow = chain(3);
    let mut source = IdSource::new();
    let argument = note(&mut source, "hello");
    let mut run = Run::<()>::start(
        &workflow,
        Arguments::new().supply("n0", "input", argument),
        3,
    )
    .expect("a sound chain starts");
    let mut last = None;
    for _ in 0..3 {
        let Step::Activate(activation) = run.step() else {
            panic!("a budget of three offers three")
        };
        if activation.instance() == "n2" {
            last = Some(activation);
            break;
        }
        let output = answer(&mut source, &activation);
        run.produced(output).expect("accepted");
    }
    let last = last.expect("n2 is offered third");
    let record = run.record(&source);

    let (mut resumed, mut again) = Run::<()>::resume(&workflow, &record).expect("resumes");
    let Step::Activate(offered) = resumed.step() else {
        panic!("the outstanding activation is offered again, not the budget exhausted")
    };
    assert_eq!(offered.instance(), "n2");
    assert_eq!(offered.inputs()[0].1.id(), last.inputs()[0].1.id());
    let output = answer(&mut again, &offered);
    resumed.produced(output).expect("accepted");
    let Step::Ended(crate::RunEnding::Completed(_)) = resumed.step() else {
        panic!("a budget of exactly three completes a chain of three")
    };
    // Offered again and not counted again: the resumed record says three.
    assert!(resumed.record(&again).contains("spent = \"3\""));
}

/// Texts a format is likeliest to change, and anything else.
#[cfg(test)]
fn awkward_text() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just("\r\n".to_owned()),
        Just("a\rb".to_owned()),
        Just("\0\u{1}\u{7f}".to_owned()),
        Just("\u{feff}\u{2028}".to_owned()),
        Just("'''\"\"\"\\".to_owned()),
        Just("\n\n  an answer, spaced \n\n".to_owned()),
        any::<String>(),
    ]
}

#[cfg(test)]
proptest! {
    /// Any texts, composed any way - parts shared, repeated, or none - come
    /// back from a record as the contexts they were, and a part two
    /// compositions hold as one value.
    #[test]
    fn contexts_kept(
        texts in proptest::collection::vec(awkward_text(), 1..6),
        shapes in proptest::collection::vec(proptest::collection::vec(0..6usize, 0..4), 1..4),
        separator in awkward_text(),
    ) {
        let mut source = IdSource::new();
        let mut made: Vec<Context> = texts.iter().map(|text| note(&mut source, text)).collect();
        for shape in &shapes {
            let parts: Vec<&Context> = shape.iter().map(|index| &made[index % made.len()]).collect();
            let composed = Context::compose(&mut source, context_type("note"), parts, &separator)
                .expect("a fresh source issues");
            made.push(composed);
        }
        let everything = Context::compose(&mut source, context_type("note"), made.iter(), &separator)
            .expect("a fresh source issues");

        let workflow = definition(vec![node_type("Take", &[("input", "note")], "note")], vec![instance("only", "Take", &[])], &["only"]);
        let run = Run::<()>::start(&workflow, Arguments::new().supply("only", "input", everything.clone()), 1)
            .expect("starts");
        let (mut resumed, _) = Run::<()>::resume(&workflow, &run.record(&source)).expect("resumes");
        let Step::Activate(activation) = resumed.step() else { panic!("offered") };
        let back = activation.inputs()[0].1.clone();

        let mut originals: Vec<&Context> = everything.lineage();
        originals.push(&everything);
        let mut resumed_ones: Vec<&Context> = back.lineage();
        resumed_ones.push(&back);
        prop_assert_eq!(originals.len(), resumed_ones.len());
        for original in &originals {
            let found = resumed_ones.iter().find(|c| c.id() == original.id()).expect("same identifiers");
            prop_assert_eq!(found.declared_type(), original.declared_type());
            prop_assert_eq!(found.render(), original.render());
            prop_assert_eq!(found.separator(), original.separator());
            let ids = |c: &Context| c.parts().iter().map(Context::id).collect::<Vec<_>>();
            prop_assert_eq!(ids(found), ids(original));
        }
        // One value per identifier, wherever it is held.
        for holder in &resumed_ones {
            for part in holder.parts() {
                let other = resumed_ones.iter().find(|c| c.id() == part.id()).expect("held");
                prop_assert!(part.is(other), "{:?} came back as two values", part.id());
            }
        }
    }
}

#[cfg(test)]
#[test]
fn deep_composition_kept() {
    const DEPTH: u64 = 100_000;
    // Each level from a source of its own standing below its part's, so that
    // identifiers fall outward and making the outermost goes down every level.
    let mut deep = note(&mut IdSource::resumed_at(Some(DEPTH)), "x");
    for level in (0..DEPTH).rev() {
        let mut source = IdSource::resumed_at(Some(level));
        deep = Context::compose(&mut source, context_type("note"), [&deep], "")
            .expect("a fresh source issues");
    }
    let source = IdSource::resumed_at(Some(DEPTH + 1));
    let workflow = definition(
        vec![node_type("Take", &[("input", "note")], "note")],
        vec![instance("only", "Take", &[])],
        &["only"],
    );
    let run = Run::<()>::start(
        &workflow,
        Arguments::new().supply("only", "input", deep.clone()),
        1,
    )
    .expect("starts");
    let record = run.record(&source);

    // A stack a recursive walk of this depth overflows many times over.
    let resumed = std::thread::Builder::new()
        .stack_size(256 << 10)
        .spawn(move || {
            let (mut resumed, _) = Run::<()>::resume(&workflow, &record).expect("resumes");
            let Step::Activate(activation) = resumed.step() else {
                panic!("offered")
            };
            let back = activation.inputs()[0].1.clone();
            (back.id(), back.render().into_owned())
        })
        .expect("a thread")
        .join()
        .expect("resumed without overflowing");
    assert_eq!(resumed, (deep.id(), "x".to_owned()));
}

#[cfg(test)]
proptest! {
    /// However many contexts were made and dropped before a run was recorded,
    /// the source that comes back with it issues none of the identifiers the
    /// recorded source had.
    #[test]
    fn source_continues(before in 0..40usize, after in 0..5usize, later in 1..20usize) {
        let workflow = chain(2);
        let mut source = IdSource::new();
        let mut issued = std::collections::HashSet::new();
        for _ in 0..before {
            issued.insert(note(&mut source, "dropped").id());
        }
        let argument = note(&mut source, "hello");
        issued.insert(argument.id());
        // Made after everything the run holds, and dropped: only the source's
        // position says they were issued.
        for _ in 0..after {
            issued.insert(note(&mut source, "dropped").id());
        }
        let run = Run::<()>::start(&workflow, Arguments::new().supply("n0", "input", argument), 3)
            .expect("starts");
        let record = run.record(&source);
        let (_, mut again) = Run::<()>::resume(&workflow, &record).expect("resumes");
        for _ in 0..later {
            let id = note(&mut again, "later").id();
            prop_assert!(!issued.contains(&id), "{:?} was issued before the record", id);
        }

        // Recorded exhausted, it comes back exhausted rather than fresh.
        let exhausted = run.record(&IdSource::resumed_at(None));
        prop_assert!(exhausted.contains("source = \"exhausted\""));
        let (_, mut again) = Run::<()>::resume(&workflow, &exhausted).expect("resumes");
        prop_assert_eq!(again.issue(), Err(crate::SourceExhausted));
    }
}

#[cfg(test)]
#[test]
fn diverged_record_refused() {
    // The measured shape: x and y entries, s bound to x, t bound to s - and the
    // same with s bound to y.
    let bound = |source: &str, order: &[&str]| {
        let types = vec![
            node_type("Src", &[("input", "note")], "note"),
            node_type("Step", &[("input", "note")], "note"),
        ];
        let all = [
            instance("x", "Src", &[]),
            instance("y", "Src", &[]),
            instance("s", "Step", &[("input", source)]),
            instance("t", "Step", &[("input", "s")]),
        ];
        let instances = order
            .iter()
            .map(|name| {
                all.iter()
                    .find(|node| node.name == *name)
                    .expect("named")
                    .clone()
            })
            .collect();
        definition(types, instances, &["t"])
    };
    let original = bound("x", &["x", "y", "s", "t"]);
    let mut source = IdSource::new();
    let (ax, ay) = (note(&mut source, "x"), note(&mut source, "y"));
    let arguments = Arguments::new()
        .supply("x", "input", ax)
        .supply("y", "input", ay);
    let mut run = Run::<()>::start(&original, arguments, 10).expect("starts");
    for _ in 0..3 {
        let Step::Activate(activation) = run.step() else {
            panic!("offered")
        };
        let output = answer(&mut source, &activation);
        run.produced(output).expect("accepted");
    }
    let record = run.record(&source);
    assert!(
        Run::<()>::resume(&original, &record).is_ok(),
        "the control resumes"
    );

    let refused =
        |workflow: &WorkflowDefinition, record: &str| match Run::<()>::resume(workflow, record) {
            Err(ResumeRefusal::Diverged {
                output,
                instance,
                divergence,
            }) => (output, instance, divergence),
            Err(other) => panic!("expected a divergence, got {other:?}"),
            Ok(_) => panic!("expected a divergence, and it resumed"),
        };

    // Bound to another input: the instance is offered, its inputs differ.
    let (output, instance_name, divergence) = refused(&bound("y", &["x", "y", "s", "t"]), &record);
    assert_eq!((output, instance_name.as_str()), (2, "s"));
    let Divergence::InputsDiffer { recorded, offered } = divergence else {
        panic!("expected inputs to differ, got {divergence:?}")
    };
    assert_ne!(recorded, offered);

    // The same instances in another order.
    let (output, instance_name, divergence) = refused(&bound("x", &["y", "x", "s", "t"]), &record);
    assert_eq!((output, instance_name.as_str()), (0, "x"));
    assert_eq!(
        divergence,
        Divergence::OtherOffered {
            offered: "y".to_owned()
        }
    );

    // An output for an instance the workflow does not have, and one after its
    // designated output: a complete chain of three against chains of two.
    let three = chain(3);
    let mut source = IdSource::new();
    let argument = note(&mut source, "hello");
    let mut run = Run::<()>::start(&three, Arguments::new().supply("n0", "input", argument), 5)
        .expect("starts");
    let (_, _) = drive(&mut run, &mut source, &mut |_, _| {});
    let complete = run.record(&source);
    let (output, instance_name, divergence) = refused(&chain(2), &complete);
    assert_eq!((output, instance_name.as_str()), (2, "n2"));
    assert_eq!(divergence, Divergence::NothingOffered);
    let mut renamed = chain(3);
    renamed.instances[1].name = "m1".to_owned();
    renamed.instances[2].bindings[0].source = "m1".to_owned();
    let (output, instance_name, divergence) = refused(&renamed, &complete);
    assert_eq!((output, instance_name.as_str()), (1, "n1"));
    assert_eq!(
        divergence,
        Divergence::OtherOffered {
            offered: "m1".to_owned()
        }
    );

    // An output of a type the workflow no longer declares for its instance.
    let mut retyped = chain(3);
    retyped
        .node_types
        .push(node_type("End", &[("input", "note")], "summary"));
    retyped.instances[2].node_type = "End".to_owned();
    let (output, instance_name, divergence) = refused(&retyped, &complete);
    assert_eq!((output, instance_name.as_str()), (2, "n2"));
    assert!(
        matches!(
            divergence,
            Divergence::Refused(OutputRefusal::UndeclaredType { .. })
        ),
        "{divergence:?}"
    );

    // Activations spent below the outputs, two above them, and one above them
    // where the workflow would offer nothing more.
    for (spent, expected) in [("2", 2), ("5", 5), ("4", 4)] {
        let altered = complete.replace("spent = \"3\"", &format!("spent = \"{spent}\""));
        assert_ne!(altered, complete, "the edit landed");
        assert_eq!(
            Run::<()>::resume(&three, &altered).err(),
            Some(ResumeRefusal::SpentDisagrees {
                spent: expected,
                outputs: 3
            })
        );
    }
}

#[test]
fn resumes_mid_repetition() {
    // One instance reading its own output, given its first context: each pass
    // is given the output of the one before.
    let workflow = definition(
        vec![node_type("Take", &[("seed", "note")], "note")],
        vec![
            instance("x", "Take", &[("seed", "x")]),
            instance("never", "Take", &[("seed", "never")]),
        ],
        &["never"],
    );
    let mut source = IdSource::new();
    let first = note(&mut source, "first");
    let arguments = Arguments::new().supply("x", "seed", first.clone());
    let mut run = Run::<()>::start(&workflow, arguments, 10).expect("starts");
    let mut outputs = Vec::new();
    for _ in 0..2 {
        let Step::Activate(activation) = run.step() else {
            panic!("offered")
        };
        let output = answer(&mut source, &activation);
        outputs.push(output.clone());
        run.produced(output).expect("accepted");
    }
    let record = run.record(&source);

    // Resumed, the third pass is offered with the second's output: the edge's
    // place is rebuilt from the record, not started again.
    let (mut resumed, _) = Run::<()>::resume(&workflow, &record).expect("it resumes");
    let Step::Activate(third) = resumed.step() else {
        panic!("the third pass is offered")
    };
    assert_eq!(third.instance(), "x");
    assert_eq!(third.inputs()[0].1.id(), outputs[1].id());

    // A record saying the second pass was given the run's context again, as
    // taking the edge from its start would, is refused at that output.
    let at = format!("seed = \"{}\"", outputs[0].id().value());
    assert_eq!(record.matches(&at).count(), 1, "{record}");
    let altered = record.replace(&at, &format!("seed = \"{}\"", first.id().value()));
    match Run::<()>::resume(&workflow, &altered) {
        Err(ResumeRefusal::Diverged {
            output: 1,
            instance,
            divergence: Divergence::InputsDiffer { .. },
        }) => assert_eq!(instance, "x"),
        other => panic!("expected the second output refused, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn start_refusal_carried() {
    let workflow = chain(3);
    let (record, _) = two_of_three(&workflow);
    let argument = || {
        let (resumed, _) = Run::<()>::resume(&workflow, &record).expect("the control resumes");
        resumed.recorded().arguments.clone()
    };

    // A wiring defect: a binding to an instance the workflow does not have.
    let mut defective = chain(3);
    defective.instances[2].bindings[0].source = "nowhere".to_owned();
    let started = Run::<()>::start(&defective, argument(), 5).err();
    assert!(matches!(started, Some(StartRefusal::Wiring(_))));
    assert_eq!(
        Run::<()>::resume(&defective, &record).err(),
        started.map(ResumeRefusal::Start)
    );

    // A parameter nothing binds that the recorded argument does not fill.
    let mut renamed = chain(3);
    renamed.node_types[0].required[0].name = "prompt".to_owned();
    let started = Run::<()>::start(&renamed, argument(), 5).err();
    assert!(matches!(started, Some(StartRefusal::Signature(_))));
    assert_eq!(
        Run::<()>::resume(&renamed, &record).err(),
        started.map(ResumeRefusal::Start)
    );
}

#[cfg(test)]
#[test]
fn unreadable_refused() {
    let workflow = chain(3);
    let (record, _) = two_of_three(&workflow);
    // Identifiers: the argument 0 and a dropped context 1, then for each of the
    // two outputs a word and the output composed of its input, the word and its
    // input again - 2 and 3, then 4 and 5. The source stands at 6.
    assert!(
        Run::<()>::resume(&workflow, &record).is_ok(),
        "the control resumes"
    );
    let damaged = |from: &str, to: &str| {
        assert!(record.contains(from), "{from:?} is in the record");
        fault(Run::<()>::resume(&workflow, &record.replacen(from, to, 1)))
    };
    let key = |parts: &[&str]| owned(parts);
    let line_of = |text: &str, needle: &str| {
        text.lines()
            .position(|line| line.contains(needle))
            .expect("present")
            + 1
    };

    let not_toml = fault(Run::<()>::resume(&workflow, "version = \"2\"\nbudget = \n"));
    assert!(
        matches!(not_toml.kind(), RecordFaultKind::Syntax { .. }),
        "{not_toml}"
    );
    assert_eq!(not_toml.line(), 2);

    assert_eq!(
        damaged("version = \"2\"", "version = \"3\"").kind(),
        &RecordFaultKind::Version {
            found: "3".to_owned()
        }
    );
    assert_eq!(
        damaged("budget = \"5\"\n", "").kind(),
        &RecordFaultKind::MissingKey {
            key: key(&["budget"])
        }
    );
    assert_eq!(
        damaged("spent = \"2\"\n", "spent = \"2\"\nextra = \"x\"\n").kind(),
        &RecordFaultKind::UnexpectedKey {
            key: key(&["extra"])
        }
    );
    assert_eq!(
        damaged("budget = \"5\"", "budget = 5").kind(),
        &RecordFaultKind::WrongType {
            key: key(&["budget"]),
            expected: "string",
            found: "integer"
        }
    );
    assert_eq!(
        damaged("[context.3]", "[context.03]").kind(),
        &RecordFaultKind::NotANumber {
            key: key(&["context", "03"]),
            found: "03".to_owned()
        }
    );
    assert_eq!(
        damaged(
            "parts = [\"3\", \"4\", \"3\"]",
            "parts = [\"3\", \"9\", \"3\"]"
        )
        .kind(),
        &RecordFaultKind::UnknownContext {
            key: key(&["context", "5", "parts", "1"]),
            id: 9
        }
    );
    // 3 made of 5, which is made of 3.
    let circle = damaged("parts = [\"0\", \"2\", \"0\"]", "parts = [\"5\"]");
    assert!(
        matches!(circle.kind(), RecordFaultKind::ComposedOfItself { .. }),
        "{circle}"
    );
    let at_source = damaged("source = \"6\"", "source = \"5\"");
    assert_eq!(
        at_source.kind(),
        &RecordFaultKind::NotIssued {
            key: key(&["context", "5"])
        }
    );
    assert_eq!(at_source.line(), line_of(&record, "[context.5]"));

    // A context nothing reaches: the dropped one, written in as if held.
    let stray = damaged(
        "[context.2]",
        "[context.1]\ntype = \"note\"\ntext = \"dropped\"\n\n[context.2]",
    );
    assert_eq!(
        stray.kind(),
        &RecordFaultKind::HeldByNothing {
            key: key(&["context", "1"])
        }
    );
}

#[cfg(test)]
#[test]
fn identifiers_only_with_a_source() {
    let workflow = chain(2);
    let mut source = IdSource::new();
    let argument = note(&mut source, "hello");
    let _ = note(&mut source, "dropped");
    let run = Run::<()>::start(
        &workflow,
        Arguments::new().supply("n0", "input", argument),
        3,
    )
    .expect("starts");
    let record = run.record(&source);
    assert!(record.contains("source = \"2\""));

    // The argument moved to identifier 1, beside a source standing at 0: one
    // past it.
    let behind = record
        .replace("[context.0]", "[context.1]")
        .replace("context = \"0\"", "context = \"1\"");
    let behind = behind.replace("source = \"2\"", "source = \"0\"");
    assert_eq!(
        fault(Run::<()>::resume(&workflow, &behind)).kind(),
        &RecordFaultKind::NotIssued {
            key: owned(&["context", "1"])
        }
    );

    // The highest identifier there is, beside a source that has issued them
    // all: it resumes, and the source issues nothing.
    let highest = u64::MAX.to_string();
    let last = record
        .replace("[context.0]", &format!("[context.{highest}]"))
        .replace("context = \"0\"", &format!("context = \"{highest}\""))
        .replace("source = \"2\"", "source = \"exhausted\"");
    let (resumed, mut again) = Run::<()>::resume(&workflow, &last).expect("resumes");
    assert!(
        resumed
            .recorded()
            .held
            .iter()
            .flat_map(|held| held.keys())
            .any(|id| id.value() == u64::MAX)
    );
    assert_eq!(again.issue(), Err(crate::SourceExhausted));
}

// --- calls and exchanges -------------------------------------------------------

/// A workflow of instances that may each call `lookup`, which takes one `query`,
/// the last designated.
#[cfg(test)]
fn asking(instances: usize) -> WorkflowDefinition {
    let types = vec![
        node_type("ask", &[], "note"),
        node_type("lookup", &[("query", "note")], "note"),
    ];
    let names: Vec<String> = (0..instances).map(|n| format!("a{n}")).collect();
    let nodes = names
        .iter()
        .map(|name| instance(name, "ask", &[]).with_calls(&["lookup"]))
        .collect();
    definition(types, nodes, &[names.last().expect("one at least")])
}

/// The events of a record's text, read as TOML rather than resumed.
#[cfg(test)]
fn events_of(text: &str) -> Vec<Table> {
    let record: DocumentMut = text.parse().expect("a record is TOML");
    record["event"]
        .as_array_of_tables()
        .expect("events")
        .iter()
        .cloned()
        .collect()
}

#[cfg(test)]
#[test]
fn holds_exchanges() {
    let workflow = asking(1);
    let mut source = IdSource::new();
    let mut run = Run::<()>::start(&workflow, Arguments::new(), 10).expect("sound");
    let Step::Activate(_) = run.step() else {
        panic!("the asker is offered")
    };

    // An answer that made no call, then a window composing the first exchange
    // and an answer making three calls under three providers' identifiers.
    let prompt = note(&mut source, "prompt");
    let first = note(&mut source, "first");
    run.exchange(Exchange::new(prompt.clone(), first.clone()))
        .expect("an exchange that made no call");
    let window = Context::compose(&mut source, context_type("note"), [&prompt, &first], "")
        .expect("a fresh source issues");
    let offer = note(&mut source, "lookup");
    let long = "9f".repeat(16);
    let ids = ["call_7", "toolu_7", long.as_str()];
    let mut exchange =
        Exchange::new(window.clone(), note(&mut source, "calls")).offering(vec![offer.clone()]);
    let calls: Vec<Call> = ids
        .iter()
        .map(|&id| Call::new(id, "lookup").input("query", note(&mut source, id)))
        .collect();
    for call in &calls {
        exchange = exchange.calling(call.clone());
    }
    run.exchange(exchange)
        .expect("an exchange that made three calls");
    for (call, id) in calls.into_iter().zip(ids) {
        run.call(call).expect("declared");
        let Step::Activate(called) = run.step() else {
            panic!("the call is offered")
        };
        assert_eq!(called.call(), Some(id));
        run.produced(note(&mut source, "found"))
            .expect("lookup's output");
    }

    let text = run.record(&source);
    let record: DocumentMut = text.parse().expect("a record is TOML");
    assert_eq!(record["version"].as_str(), Some("2"));
    let events = events_of(&text);
    let kinds: Vec<&str> = events
        .iter()
        .map(|event| {
            ["exchange", "call", "output"]
                .into_iter()
                .find(|kind| event.contains_key(kind))
                .expect("one kind each")
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "exchange", "exchange", "call", "output", "call", "output", "call", "output"
        ]
    );

    let id = |context: &Context| context.id().value().to_string();
    let second = &events[1]["exchange"];
    assert_eq!(second["window"].as_str(), Some(id(&window).as_str()));
    let calls: Vec<(&str, &str)> = second["calls"]
        .as_array()
        .expect("calls")
        .iter()
        .filter_map(|call| call.as_inline_table())
        .map(|call| {
            (
                call["id"].as_str().expect("an identifier"),
                call["node_type"].as_str().expect("a node type"),
            )
        })
        .collect();
    let expected: Vec<(&str, &str)> = ids.iter().map(|&id| (id, "lookup")).collect();
    assert_eq!(
        calls, expected,
        "each call whole, its identifier as the provider gave it"
    );
    let offered: Vec<&str> = second["offer"]
        .as_array()
        .expect("an offer")
        .iter()
        .filter_map(|offered| offered.as_str())
        .collect();
    assert_eq!(offered, [id(&offer).as_str()]);
    for (event, id) in [
        (&events[2], "call_7"),
        (&events[4], "toolu_7"),
        (&events[6], long.as_str()),
    ] {
        assert_eq!(event["call"]["id"].as_str(), Some(id));
        assert_eq!(event["call"]["node_type"].as_str(), Some("lookup"));
    }
    for (event, id) in [
        (&events[3], "call_7"),
        (&events[5], "toolu_7"),
        (&events[7], long.as_str()),
    ] {
        assert_eq!(event["performing"].as_str(), Some(id));
    }

    // The window is its parts, not its rendering.
    let written = &record["context"][id(&window).as_str()];
    assert!(written.get("text").is_none(), "{written}");
    let parts: Vec<&str> = written["parts"]
        .as_array()
        .expect("parts")
        .iter()
        .filter_map(|part| part.as_str())
        .collect();
    assert_eq!(parts, [id(&prompt).as_str(), id(&first).as_str()]);
}

#[cfg(test)]
#[test]
fn version_one_refused() {
    // A record of version 1, as it was written before exchanges existed.
    let written = "version = \"1\"\nbudget = \"5\"\nspent = \"1\"\nsource = \"1\"\n\n\
                   [[argument]]\ninstance = \"n0\"\nparameter = \"input\"\ncontext = \"0\"\n\n\
                   [context.0]\ntype = \"note\"\ntext = \"hello\"\n";
    let refused = fault(Run::<()>::resume(&chain(2), written));
    assert_eq!(
        refused.kind(),
        &RecordFaultKind::Version {
            found: "1".to_owned()
        }
    );
    assert_eq!((refused.line(), refused.column()), (1, 11));
}

#[cfg(test)]
#[test]
fn undeclared_call_refused() {
    let workflow = asking(1);
    let mut source = IdSource::new();
    let mut run = Run::<()>::start(&workflow, Arguments::new(), 10).expect("sound");
    let Step::Activate(_) = run.step() else {
        panic!("the asker is offered")
    };
    let query = note(&mut source, "q");
    let call = Call::new("call_1", "lookup").input("query", query);
    run.exchange(
        Exchange::new(note(&mut source, "window"), note(&mut source, "answer"))
            .calling(call.clone()),
    )
    .expect("held");
    run.call(call).expect("declared");
    let Step::Activate(_) = run.step() else {
        panic!("the call is offered")
    };
    // Taken while the called node type is outstanding - performed by a person,
    // say - so its output is nowhere in the record.
    let record = run.record(&source);

    let mut undeclaring = workflow.clone();
    undeclaring.instances[0].calls.clear();
    match Run::<()>::resume(&undeclaring, &record) {
        Err(ResumeRefusal::CallRefused {
            instance,
            call,
            refusal,
        }) => {
            assert_eq!((instance.as_str(), call.as_str()), ("a0", "call_1"));
            assert_eq!(
                refusal,
                CallRefusal::Undeclared {
                    instance: "a0".to_owned(),
                    node_type: "lookup".to_owned(),
                }
            );
        }
        Err(other) => panic!("expected the call refused, got {other:?}"),
        Ok(_) => panic!("expected the call refused, and it resumed"),
    }

    // Against the workflow unchanged it resumes with the call outstanding,
    // counted once.
    let (mut resumed, _) = Run::<()>::resume(&workflow, &record).expect("resumes");
    assert_eq!(resumed.recorded().spent, 2);
    let Step::Activate(called) = resumed.step() else {
        panic!("the call is offered again")
    };
    assert_eq!((called.instance(), called.call()), ("a0", Some("call_1")));
    assert_eq!(resumed.recorded().spent, 2, "not counted again");
}

#[cfg(test)]
proptest! {
    /// For any run whose instances exchange and call - some calls exchanging in
    /// turn - resumed from every record it can be written as, the resumed run
    /// holds the same exchanges with the same activations in the same order: it
    /// is written again as the same text, byte for byte, and its outstanding
    /// activation holds the exchanges the recorded one held.
    #[test]
    fn exchanges_kept(
        plans in proptest::collection::vec(
            proptest::collection::vec((0..3usize, any::<bool>()), 0..=3),
            1..=2,
        ),
    ) {
        let workflow = asking(plans.len());
        let mut source = IdSource::new();
        let mut run = Run::<()>::start(&workflow, Arguments::new(), 50).expect("sound");
        let mut records = Vec::new();
        let mut windows = Vec::new();

        let take = |run: &Run<'_, ()>, source: &IdSource| {
            let held: Vec<u64> = run.exchanges().iter().map(|e| e.window().id().value()).collect();
            (run.record(source), held)
        };
        records.push(take(&run, &source));
        for (instance, plan) in plans.iter().enumerate() {
            let Step::Activate(activation) = run.step() else {
                return Err(TestCaseError::fail("each instance is offered"));
            };
            prop_assert_eq!(activation.instance(), format!("a{instance}"));
            for (exchange, &(calls, callee_exchanges)) in plan.iter().enumerate() {
                let window = note(&mut source, "window");
                windows.push(window.clone());
                let mut made = Exchange::new(window, note(&mut source, "answer"));
                let calls: Vec<Call> = (0..calls)
                    .map(|call| {
                        let id = format!("a{instance}-{exchange}-{call}");
                        let query = note(&mut source, &id);
                        Call::new(&id, "lookup").input("query", query)
                    })
                    .collect();
                for call in &calls {
                    made = made.calling(call.clone());
                }
                run.exchange(made).expect("held");
                records.push(take(&run, &source));
                for call in calls {
                    run.call(call).expect("declared");
                    records.push(take(&run, &source));
                    let Step::Activate(_) = run.step() else {
                        return Err(TestCaseError::fail("the call is offered"));
                    };
                    records.push(take(&run, &source));
                    if callee_exchanges {
                        run.exchange(Exchange::new(note(&mut source, "w"), note(&mut source, "a")))
                            .expect("the callee's own");
                        records.push(take(&run, &source));
                    }
                    run.produced(note(&mut source, "found")).expect("lookup's output");
                    records.push(take(&run, &source));
                }
            }
            run.produced(note(&mut source, "output")).expect("the instance's output");
            records.push(take(&run, &source));
        }

        for (record, held) in &records {
            let (resumed, again) = Run::<()>::resume(&workflow, record)
                .map_err(|refused| TestCaseError::fail(format!("{refused}\n{record}")))?;
            prop_assert_eq!(&resumed.record(&again), record);
            let resumed_held: Vec<u64> = resumed.exchanges().iter().map(|e| e.window().id().value()).collect();
            prop_assert_eq!(&resumed_held, held);
        }
    }
}
