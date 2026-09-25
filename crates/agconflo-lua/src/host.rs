//! The script host: one activation, performed by running its node type's script
//! in a Lua state made for it alone, with nothing to reach but the activation,
//! the context API and the models its caller mapped, and under its limits.

use std::cell::{Cell, RefCell};
use std::fmt;
use std::rc::Rc;

use std::collections::HashMap;

use agconflo_core::{
    Activation, Call, CallRefusal, Context, ContextType, Exchange, ExchangeRefusal, IdSource,
    NodeType, OutputRefusal, SourceExhausted,
};
use mlua::prelude::*;

use crate::behaviours::Script;
use crate::models::{Asked, ModelFailure, Offered, Part, Roster};

/// What one activation's script may spend.
///
/// Counted in instructions executed, bytes allocated and model calls made rather
/// than in time (`DEC_LIMITS_NOT_TIME`): the same script stops at the same point
/// on every machine, and a slow machine is not a runaway script. Each activation
/// has its own, since nothing else is shared between activations either.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Instructions one activation's script may execute. Counted in steps of a
    /// thousand, so a script is stopped within a thousand instructions of it.
    pub instructions: u64,
    /// Bytes one activation's Lua state may hold, the state's own included.
    pub memory: usize,
    /// Model calls one activation's script may make (`DEC_MODEL_CALLS_COUNTED`).
    /// Awaiting a model costs no instructions, so without this a loop was
    /// measured making 2000 calls in one activation (`EVD_MODEL_CALLS_UNLIMITED`).
    pub model_calls: u32,
}

impl Default for Limits {
    /// Ten million instructions, 64 MiB and one model call: a few tens of
    /// milliseconds of work, several thousand times what an empty state holds,
    /// and a run whose calls are bounded by its step budget. None of the
    /// numbers is a requirement; each is room for a script that assembles text
    /// and asks one question.
    fn default() -> Self {
        Self {
            instructions: 10_000_000,
            memory: 64 << 20,
            model_calls: 1,
        }
    }
}

/// How an activation's script failed.
///
/// Carried as the run's failure, so a caller learns which of these it was as a
/// value (`STKH_TYPED_FAILURE`). A limit is never reported as a script error nor
/// the other way round: every limit reaches a script as a Lua error, and telling
/// them apart is this type's reason to exist.
///
/// `#[non_exhaustive]`: what a script can do wrong grows with what it can do.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScriptFailure {
    /// The script raised an error, or one was raised in something it called
    /// (`CREQ_HOST_ERROR_CARRIED`).
    Raised {
        /// The error as Lua reports it, with the document, the line and the
        /// traceback. An error raised with something other than a string has no
        /// message worth the name: a table renders as its address.
        message: String,
    },
    /// The script returned something other than exactly one context
    /// (`CREQ_HOST_ONE_CONTEXT`).
    NotOneContext {
        /// What it returned: `nothing`, a Lua type's name, or how many values.
        found: String,
    },
    /// The script executed more instructions than its limit
    /// (`CREQ_HOST_INSTRUCTION_LIMIT`).
    InstructionLimit,
    /// The script allocated more memory than its limit
    /// (`CREQ_HOST_MEMORY_LIMIT`).
    MemoryLimit,
    /// The script asked for more model calls than its limit
    /// (`CREQ_HOST_MODEL_CALL_LIMIT`). The call over the limit was not made.
    ModelCallLimit,
    /// A model call failed (`CREQ_HOST_MODEL_FAILURE`), and this is how.
    ModelFailed(ModelFailure),
    /// The script returned a context the run refused
    /// (`CREQ_HOST_OUTPUT_REFUSAL_CARRIED`), and this is the run's refusal.
    OutputRefused(OutputRefusal),
    /// A person answered a step and the identifier source had nothing left to
    /// issue the answer's context under (`CREQ_HOST_TAKES_PERSON_TEXT`). A
    /// script meeting the same source raises an error from the function it
    /// called, which is carried as [`ScriptFailure::Raised`].
    SourceExhausted(SourceExhausted),
    /// A model's answer made a call Agconflo refuses, and none of that answer's
    /// calls was performed (`CREQ_HOST_REFUSES_MALFORMED_CALL`).
    MalformedCall {
        /// The name the model called.
        node_type: String,
        /// Which fault it was.
        fault: ModelCallFault,
    },
    /// The run refused a call the model made (`CREQ_HOST_CALL_REFUSAL_CARRIED`),
    /// and this is the run's refusal.
    CallRefused(CallRefusal),
    /// The run refused an exchange the model made, and this is the run's
    /// refusal. Every context of an exchange is issued by the run's own source
    /// or read from its record, so nothing here is expected to produce one.
    ExchangeRefused(ExchangeRefusal),
    /// An activation resumed from its record sent a window or an offer other
    /// than the one its record holds at that point, and nothing was sent
    /// (`CREQ_HOST_REPLAY_DIVERGED`).
    Diverged {
        /// Which of the activation's recorded exchanges it differs from, the
        /// first being 0.
        exchange: usize,
        /// Whether what differs is the offer rather than the window.
        offer: bool,
    },
}

/// What is wrong with a call a model made (`DEC_MALFORMED_CALL_FAILS`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModelCallFault {
    /// The name is not one of the node types offered.
    NotOffered,
    /// The arguments are not a JSON object.
    NotAnObject,
    /// An argument names a parameter the node type does not declare.
    UndeclaredParameter {
        /// The parameter as the model named it.
        parameter: String,
    },
    /// A required parameter has no argument.
    RequiredMissing {
        /// The parameter.
        parameter: String,
    },
    /// An argument is not a string, so there is no text to make its context of.
    NotAString {
        /// The parameter.
        parameter: String,
    },
}

impl fmt::Display for ModelCallFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotOffered => f.write_str("it was not offered"),
            Self::NotAnObject => f.write_str("its arguments are not an object"),
            Self::UndeclaredParameter { parameter } => {
                write!(f, "it declares no parameter {parameter}")
            }
            Self::RequiredMissing { parameter } => write!(f, "{parameter} is required"),
            Self::NotAString { parameter } => write!(f, "{parameter} is not a string"),
        }
    }
}

impl fmt::Display for ScriptFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raised { message } => write!(f, "the script raised an error: {message}"),
            Self::NotOneContext { found } => {
                write!(f, "the script returned {found} rather than one context")
            }
            Self::InstructionLimit => f.write_str("the script exceeded its instruction limit"),
            Self::MemoryLimit => f.write_str("the script exceeded its memory limit"),
            Self::ModelCallLimit => f.write_str("the script exceeded its model call limit"),
            Self::ModelFailed(failure) => write!(f, "a model call failed: {failure}"),
            Self::OutputRefused(refusal) => write!(f, "the run refused the output: {refusal}"),
            Self::SourceExhausted(exhausted) => {
                write!(f, "the person's answer could not be kept: {exhausted}")
            }
            Self::MalformedCall { node_type, fault } => {
                write!(f, "the model's call to {node_type} is refused: {fault}")
            }
            Self::CallRefused(refusal) => write!(f, "the run refused a call: {refusal}"),
            Self::ExchangeRefused(refusal) => {
                write!(f, "the run refused an exchange: {refusal}")
            }
            Self::Diverged { exchange, offer } => write!(
                f,
                "resumed, the script sent another {} than its record holds for exchange {exchange}",
                if *offer { "offer" } else { "window" }
            ),
        }
    }
}

impl std::error::Error for ScriptFailure {}

/// The globals every state has that a script is not given: those that read a
/// file or compile text (`EVD_LUA_DEFAULT_STATE_EXPOSES`), those that catch an
/// error (`EVD_LUA_PCALL_SWALLOWS_LIMITS`), `collectgarbage`, which controls the
/// state's collector, and `print`, which writes to the host process's output.
/// `load` is the one that matters most - it builds a function
/// from text a script assembled, and anything removed would come back through
/// it.
const LEFT_OUT: [&str; 8] = [
    "dofile",
    "loadfile",
    "load",
    "require",
    "pcall",
    "xpcall",
    "collectgarbage",
    "print",
];

/// A Lua state holding the environment a script runs in, and nothing else.
///
/// Built from the libraries named here rather than by taking something out of
/// a default (`DEC_ENVIRONMENT_BY_NAME`): `io`, `os`, `debug` and `package` are
/// never loaded, so what nobody thought of is out by construction. `coroutine`
/// is not loaded here either, and `mlua` loads it anyway once an asynchronous
/// function exists, so it is taken out again after the host functions are made
/// (`EVD_MLUA_ASYNC_LOADS_COROUTINE`). From what is loaded, the base library's file readers, compiler
/// and error catchers go, and so does `math`'s random source, which differed
/// between processes (`EVD_LUA_RANDOM_PER_PROCESS`).
// @An environment built from named parts,IMPL_HOST_SANDBOX,impl,[CREQ_HOST_NOTHING_OUTSIDE, CREQ_HOST_NO_CATCHING]
pub(crate) fn sandbox() -> LuaResult<Lua> {
    let lua = Lua::new_with(
        LuaStdLib::STRING | LuaStdLib::TABLE | LuaStdLib::MATH | LuaStdLib::UTF8,
        LuaOptions::default(),
    )?;
    let globals = lua.globals();
    for name in LEFT_OUT {
        globals.raw_set(name, LuaNil)?;
    }
    let math: LuaTable = globals.raw_get("math")?;
    math.raw_set("random", LuaNil)?;
    math.raw_set("randomseed", LuaNil)?;
    Ok(lua)
}

/// The name a script is compiled under: its document's, marked as a file name so
/// that Lua quotes it as written in every message rather than as a string.
pub(crate) fn chunk_name(document: &str) -> String {
    format!("@{document}")
}

/// A context as a script holds it: something to call methods on and nothing
/// else. Its metatable is closed to the script (`EVD_LUA_USERDATA_PROTECTED`), so
/// what it says cannot be changed from inside.
struct Handed(Context);

impl LuaUserData for Handed {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("render", |_, this, ()| Ok(this.0.render().into_owned()));
        methods.add_method("type", |_, this, ()| {
            Ok(this.0.declared_type().as_str().to_owned())
        });
        methods.add_method("parts", |_, this, ()| {
            Ok(this
                .0
                .parts()
                .iter()
                .cloned()
                .map(Handed)
                .collect::<Vec<_>>())
        });
    }
}

/// What one activation's model calls came to, kept outside the script so that
/// a call it could not complete is reported as what it was, not as the Lua
/// error that stopped the script.
#[derive(Default)]
struct Calls {
    /// Requests counted against the limit, answered from the record included.
    made: Cell<u32>,
    over_limit: Cell<bool>,
    /// How many of the activation's recorded exchanges have been answered from.
    replayed: Cell<usize>,
    failed: RefCell<Option<ScriptFailure>>,
}

/// What the script host asks of the run while it performs an activation - the
/// one way a model's call reaches the run (`DEC_CALL_IS_AN_ACTIVATION`).
pub(crate) enum Asking {
    /// Hold this exchange with the activation, and hand the caller a record.
    Exchange(Exchange),
    /// Perform this call as an activation of the run, and give back its output.
    Call(Call),
}

/// What the run gives back.
pub(crate) enum Given {
    /// The exchange is held.
    Held,
    /// The call's output.
    Output(Context),
    /// The run stopped here - a refusal, a failure, a person's step, its budget
    /// - and the script is to end; its caller knows which.
    Stop,
}

/// Where the script host leaves what it asks and finds what it was given.
///
/// A script's host functions own what they hold, since a model call is awaited
/// and a scoped function cannot be, and the run borrows its workflow, so it
/// cannot be held by them. So the host asks and waits, and whatever polls the
/// script answers from the run it has (`DEC_RUN_IS_DRIVEN`): the yield is to
/// the driver, as the call is a step of the run.
#[derive(Default)]
pub(crate) struct Mailbox {
    pub(crate) asking: RefCell<Option<Asking>>,
    pub(crate) given: RefCell<Option<Given>>,
}

/// Ask `asking` of the run and wait for what it gives.
///
/// Pending until the answer is in the mailbox. No waker is kept: whatever polls
/// the script sees the question, answers it, and polls again.
async fn ask(mailbox: &Rc<Mailbox>, asking: Asking) -> Given {
    *mailbox.asking.borrow_mut() = Some(asking);
    std::future::poll_fn(|_| match mailbox.given.borrow_mut().take() {
        Some(given) => std::task::Poll::Ready(given),
        None => std::task::Poll::Pending,
    })
    .await
}

/// What an activation resumed from its record is answered from: the exchanges
/// the record holds for it, in order, and the output of each of their calls the
/// record holds, by the provider's identifier (`DEC_SCRIPT_REPLAYED_FROM_ITS_RECORD`).
#[derive(Clone, Debug, Default)]
pub(crate) struct Replay {
    pub(crate) exchanges: Vec<Exchange>,
    pub(crate) outputs: HashMap<String, Context>,
}

/// What an activation is performed with besides its script and its limits:
/// the node types its model may call, what its record holds for it, and the
/// mailbox the run answers through.
pub(crate) struct Performing {
    pub(crate) callees: Vec<NodeType>,
    pub(crate) replay: Replay,
    pub(crate) mailbox: Rc<Mailbox>,
}

/// Perform `activation` by running `script`, or say how it failed.
///
/// A new state for every call (`DEC_STATE_PER_ACTIVATION`): one shared across a
/// run was measured carrying a global and a write to the string library from
/// one node's script into the next's (`EVD_LUA_SHARED_STATE_LEAKS`).
///
/// The script is the body of the behaviour (`DEC_SCRIPT_IS_THE_BODY`) and is
/// given two arguments, read as `local given, host = ...`: the activation's
/// inputs under their parameters' names, with an unbound optional parameter
/// absent rather than empty; and the host functions, which are the context API
/// and nothing else (`DEC_HOST_FUNCTIONS_CONTEXT_API`) - `host.text(type, text)`,
/// `host.compose(type, parts, separator)`, `host.complete(role, prompt, type)`
/// and `host.output`, the type the output is declared as.
///
/// Asynchronous, because a model call is awaited rather than blocked on
/// (`DEC_BEHAVIOUR_ASYNC`). The script runs in a Lua thread made for it, and its
/// instruction limit is set on that thread: a limit set on the state never ran
/// in the coroutine an asynchronous call uses (`EVD_LUA_HOOK_PER_THREAD`).
///
/// Contexts the script makes are issued identifiers from `source`, which must be
/// the source the run's arguments came from: a second source repeats the first
/// one's identifiers, and the run refuses the collision.
// @A script run in a state and thread of its own,IMPL_HOST_PERFORM,impl,[CREQ_HOST_RUNS_THE_SCRIPT, CREQ_HOST_FRESH_STATE, CREQ_HOST_INSTRUCTION_LIMIT]
pub(crate) async fn perform(
    script: &Script,
    activation: &Activation,
    source: &Rc<RefCell<IdSource>>,
    roster: &Roster,
    limits: Limits,
    performing: Performing,
) -> Result<Context, ScriptFailure> {
    let lua = sandbox().map_err(raised)?;
    lua.set_memory_limit(limits.memory).map_err(raised)?;
    let calls = Rc::new(Calls::default());

    let host = host_functions(&lua, activation, source, roster, &calls, limits, performing)
        .map_err(raised)?;
    close_coroutines(&lua).map_err(raised)?;
    let given = lua.create_table().map_err(raised)?;
    for (parameter, context) in activation.inputs() {
        given
            .raw_set(parameter.as_str(), Handed(context.clone()))
            .map_err(raised)?;
    }

    let body = lua
        .load(&script.source)
        .set_name(chunk_name(&script.document))
        .into_function()
        .map_err(raised)?;
    let thread = lua.create_thread(body).map_err(raised)?;
    let over_instructions = limit(&thread, limits).map_err(raised)?;
    let returned: LuaResult<LuaMultiValue> = match thread.into_async((given, host)) {
        Ok(running) => running.await,
        Err(error) => Err(error),
    };

    outcome(returned, &over_instructions, &calls)
}

/// The host table: the context API, the one model call, and the output type.
///
/// Every function owns what it needs rather than borrowing it for a scope,
/// because a model call is awaited and a scoped function cannot be.
/// `host.complete` checks the call limit before calling (`CREQ_HOST_MODEL_CALL_LIMIT`),
/// takes its prompt as a context and nothing else (`CREQ_HOST_PROMPT_IS_A_CONTEXT`),
/// and gives the answer back as a new context of the type the script names, or
/// of its output's declared type when it names none (`CREQ_HOST_MODEL_ANSWER`).
/// Between the two, its model may yield: see [`complete`].
// @A model call through the host,IMPL_HOST_COMPLETE,impl,[CREQ_HOST_MODEL_ANSWER, CREQ_HOST_PROMPT_IS_A_CONTEXT, CREQ_HOST_MODEL_CALL_LIMIT]
fn host_functions(
    lua: &Lua,
    activation: &Activation,
    source: &Rc<RefCell<IdSource>>,
    roster: &Roster,
    calls: &Rc<Calls>,
    limits: Limits,
    performing: Performing,
) -> LuaResult<LuaTable> {
    let host = lua.create_table()?;
    host.raw_set("output", activation.output().as_str())?;

    let issuing = source.clone();
    host.raw_set(
        "text",
        lua.create_function(move |_, (declared, text): (String, String)| {
            let declared = ContextType::new(&declared).map_err(LuaError::external)?;
            Context::text(&mut issuing.borrow_mut(), declared, text)
                .map(Handed)
                .map_err(LuaError::external)
        })?,
    )?;

    let issuing = source.clone();
    host.raw_set(
        "compose",
        lua.create_function(
            move |_,
                  (declared, parts, separator): (
                String,
                Vec<LuaUserDataRef<Handed>>,
                Option<String>,
            )| {
                let declared = ContextType::new(&declared).map_err(LuaError::external)?;
                let parts: Vec<Context> = parts.iter().map(|part| part.0.clone()).collect();
                let separator = separator.unwrap_or_default();
                Context::compose(&mut issuing.borrow_mut(), declared, &parts, &separator)
                    .map(Handed)
                    .map_err(LuaError::external)
            },
        )?,
    )?;

    let completing = Rc::new(Completing {
        source: source.clone(),
        roster: roster.clone(),
        calls: calls.clone(),
        limits,
        output: activation.output().as_str().to_owned(),
        callees: performing.callees,
        replay: performing.replay,
        mailbox: performing.mailbox,
    });
    host.raw_set(
        "complete",
        lua.create_async_function(
            move |_, (role, prompt, declared): (String, LuaAnyUserData, Option<String>)| {
                // Taken before anything is awaited: a borrow of the userdata
                // cannot be held across the call, and a prompt that is not a
                // context is refused before any call is made.
                let prompt = prompt.borrow::<Handed>().map(|handed| handed.0.clone());
                let completing = completing.clone();
                async move {
                    let prompt = prompt?;
                    let declared = declared.unwrap_or_else(|| completing.output.clone());
                    let declared = ContextType::new(&declared).map_err(LuaError::external)?;
                    complete(&completing, &role, prompt, declared)
                        .await
                        .map(Handed)
                }
            },
        )?,
    )?;

    Ok(host)
}

/// Everything one activation's `host.complete` needs, owned.
struct Completing {
    source: Rc<RefCell<IdSource>>,
    roster: Roster,
    calls: Rc<Calls>,
    limits: Limits,
    output: String,
    callees: Vec<NodeType>,
    replay: Replay,
    mailbox: Rc<Mailbox>,
}

impl Completing {
    /// End the script with `failure`, which is what it will be reported as.
    fn fail(&self, failure: ScriptFailure) -> LuaError {
        let message = failure.to_string();
        *self.calls.failed.borrow_mut() = Some(failure);
        LuaError::runtime(message)
    }
}

/// One `host.complete`: the model asked about `prompt` and answering as a
/// context of `declared`, yielding to each call it makes.
///
/// The model is offered the node types the activation's instance declares calls
/// to, as contexts of the prompt's type made from their declarations
/// (`CREQ_HOST_OFFERS_DECLARED`). Every request counts against the model call
/// limit, checked before it is sent (`DEC_EVERY_TURN_COUNTED`). An answer is
/// reported to the run - window, offer, answer and every call - before any of its
/// calls is performed, and the run hands its caller a record holding it
/// (`CREQ_HOST_REPORTS_EXCHANGES`, `CREQ_HOST_RECORD_AFTER_ANSWER`); but first
/// every call of it is checked, and one that is malformed fails the activation
/// with none reported (`CREQ_HOST_REFUSES_MALFORMED_CALL`). The calls are then
/// performed by the run in the order the answer gives them
/// (`CREQ_HOST_PERFORMS_CALLS`), and the model is sent a window composing the
/// last one, the answer, each call's contexts and each call's output
/// (`CREQ_HOST_NEXT_WINDOW`), until it answers without calling.
///
/// An activation resumed from its record answers each request its record holds
/// from the record instead, counted all the same, and each call the record holds
/// an output for from that output (`CREQ_HOST_ANSWERS_FROM_RECORD`) - once the
/// window and the offer are seen to be the ones recorded, by type and content,
/// since the identifiers a replay makes are new. A difference fails the
/// activation before anything is sent (`CREQ_HOST_REPLAY_DIVERGED`). When they
/// agree the activation goes on with the recorded contexts.
// @A model's calls performed and the next window composed,IMPL_HOST_YIELD,impl,[CREQ_HOST_OFFERS_DECLARED, CREQ_HOST_PERFORMS_CALLS, CREQ_HOST_NEXT_WINDOW, CREQ_HOST_REFUSES_MALFORMED_CALL, CREQ_HOST_REPORTS_EXCHANGES, CREQ_HOST_ANSWERS_FROM_RECORD, CREQ_HOST_REPLAY_DIVERGED, CREQ_HOST_MODEL_CALL_LIMIT]
async fn complete(
    completing: &Completing,
    role: &str,
    prompt: Context,
    declared: ContextType,
) -> LuaResult<Context> {
    let issuing = &completing.source;
    let calls = &completing.calls;
    let kind = prompt.declared_type().clone();
    let (offered, offer) = offer_for(issuing, &kind, &completing.callees)?;
    let mut parts = vec![Part::User(prompt.clone())];
    let mut window = prompt;

    loop {
        if calls.made.get() >= completing.limits.model_calls {
            calls.over_limit.set(true);
            return Err(LuaError::runtime("model call limit exceeded"));
        }
        calls.made.set(calls.made.get() + 1);

        let cursor = calls.replayed.get();
        let (answer, made) = if let Some(recorded) = completing.replay.exchanges.get(cursor) {
            calls.replayed.set(cursor + 1);
            if !same(recorded.window(), &window) {
                return Err(completing.fail(ScriptFailure::Diverged {
                    exchange: cursor,
                    offer: false,
                }));
            }
            let recorded_offer = recorded.offer();
            if recorded_offer.len() != offer.len()
                || !recorded_offer.iter().zip(&offer).all(|(a, b)| same(a, b))
            {
                return Err(completing.fail(ScriptFailure::Diverged {
                    exchange: cursor,
                    offer: true,
                }));
            }
            window = recorded.window().clone();
            (recorded.answer().clone(), recorded.calls().to_vec())
        } else {
            let answered = match completing.roster.send(role, &parts, &offered).await {
                Ok(answered) => answered,
                Err(failure) => return Err(completing.fail(ScriptFailure::ModelFailed(failure))),
            };
            let made = match checked(&answered.calls, &completing.callees, issuing) {
                Ok(made) => made?,
                Err((node_type, fault)) => {
                    return Err(completing.fail(ScriptFailure::MalformedCall { node_type, fault }));
                }
            };
            let answer = Context::text(&mut issuing.borrow_mut(), declared.clone(), answered.text)
                .map_err(LuaError::external)?;
            let mut exchange =
                Exchange::new(window.clone(), answer.clone()).offering(offer.clone());
            for call in &made {
                exchange = exchange.calling(call.clone());
            }
            if !matches!(
                ask(&completing.mailbox, Asking::Exchange(exchange)).await,
                Given::Held
            ) {
                return Err(stopped());
            }
            (answer, made)
        };

        if made.is_empty() {
            return Ok(answer);
        }

        let mut outputs = Vec::new();
        for call in &made {
            let output = match completing.replay.outputs.get(call.id()) {
                Some(output) => output.clone(),
                None => match ask(&completing.mailbox, Asking::Call(call.clone())).await {
                    Given::Output(output) => output,
                    _ => return Err(stopped()),
                },
            };
            outputs.push((call.id().to_owned(), output));
        }

        let mut composed: Vec<&Context> = vec![&window, &answer];
        for call in &made {
            composed.extend(call.inputs().iter().map(|(_, given)| given));
        }
        composed.extend(outputs.iter().map(|(_, output)| output));
        let next = Context::compose(&mut issuing.borrow_mut(), kind.clone(), composed, "")
            .map_err(LuaError::external)?;

        parts.push(Part::Answer {
            answer,
            calls: made,
        });
        for (call, output) in outputs {
            parts.push(Part::Result { call, output });
        }
        window = next;
    }
}

/// The error a script ends with when the run stopped the activation; what
/// stopped it is the caller's to know.
fn stopped() -> LuaError {
    LuaError::runtime("the run stopped this activation")
}

/// Whether two contexts are the same by type and content.
fn same(a: &Context, b: &Context) -> bool {
    a.declared_type() == b.declared_type() && a.render() == b.render()
}

/// The offer: for each node type the instance declares a call to, once, a
/// composition of text contexts of `kind` - its name, its description, and each
/// parameter's name, required ones first - and the same contexts as the roster
/// sends them (`DEC_TOOLS_OFFERED_AS_CONTEXTS`).
fn offer_for(
    issuing: &Rc<RefCell<IdSource>>,
    kind: &ContextType,
    callees: &[NodeType],
) -> LuaResult<(Vec<Offered>, Vec<Context>)> {
    let mut source = issuing.borrow_mut();
    let mut text =
        |text: &str| Context::text(&mut source, kind.clone(), text).map_err(LuaError::external);
    let mut offered = Vec::new();
    for callee in callees {
        let name = text(&callee.name)?;
        let description = text(&callee.description)?;
        let mut parameters = Vec::new();
        for (parameter, required) in (callee.required.iter().map(|p| (p, true)))
            .chain(callee.optional.iter().map(|p| (p, false)))
        {
            parameters.push((text(&parameter.name)?, required));
        }
        offered.push(Offered {
            name,
            description,
            parameters,
        });
    }
    let mut offer = Vec::new();
    for tool in &offered {
        let mut parts = vec![&tool.name, &tool.description];
        parts.extend(tool.parameters.iter().map(|(parameter, _)| parameter));
        offer.push(
            Context::compose(&mut source, kind.clone(), parts, "").map_err(LuaError::external)?,
        );
    }
    Ok((offered, offer))
}

/// Every call of an answer checked against what was offered, and only then made
/// into calls: each argument a text context of the type its parameter is
/// declared for, in the order the model gave them
/// (`DEC_CALL_CARRIES_STRING_VALUES`). The first fault found is the one
/// reported, and no call is made when there is one.
#[allow(clippy::type_complexity)]
fn checked(
    asked: &[Asked],
    callees: &[NodeType],
    issuing: &Rc<RefCell<IdSource>>,
) -> Result<LuaResult<Vec<Call>>, (String, ModelCallFault)> {
    let mut filled = Vec::new();
    for call in asked {
        let fault = |fault| (call.name.clone(), fault);
        let Some(callee) = callees.iter().find(|callee| callee.name == call.name) else {
            return Err(fault(ModelCallFault::NotOffered));
        };
        let Some(arguments) = call.arguments.as_object() else {
            return Err(fault(ModelCallFault::NotAnObject));
        };
        let mut inputs = Vec::new();
        for (parameter, value) in arguments {
            let Some(declared) = callee
                .required
                .iter()
                .chain(&callee.optional)
                .find(|declared| declared.name == *parameter)
            else {
                return Err(fault(ModelCallFault::UndeclaredParameter {
                    parameter: parameter.clone(),
                }));
            };
            let Some(text) = value.as_str() else {
                return Err(fault(ModelCallFault::NotAString {
                    parameter: parameter.clone(),
                }));
            };
            inputs.push((declared.clone(), text.to_owned()));
        }
        if let Some(missing) = callee
            .required
            .iter()
            .find(|required| !arguments.contains_key(&required.name))
        {
            return Err(fault(ModelCallFault::RequiredMissing {
                parameter: missing.name.clone(),
            }));
        }
        filled.push((call.id.clone(), callee.name.clone(), inputs));
    }

    let mut source = issuing.borrow_mut();
    Ok(filled
        .into_iter()
        .map(|(id, node_type, inputs)| {
            let mut made = Call::new(&id, &node_type);
            for (parameter, text) in inputs {
                let given = Context::text(&mut source, parameter.context_type, text)
                    .map_err(LuaError::external)?;
                made = made.input(&parameter.name, given);
            }
            Ok(made)
        })
        .collect())
}

/// Take the coroutine library back out of the script's reach.
///
/// `mlua` loads it into the globals when the first asynchronous function is
/// created, whatever the state was built with, and reads `coroutine.yield` from
/// them there and then into its own poller (`EVD_MLUA_ASYNC_LOADS_COROUTINE`).
/// Left in, a script could resume a coroutine and have any error - a limit's
/// included - handed back as a value, which is catching it under another name
/// (`CREQ_HOST_NO_CATCHING`). Removed after the host functions are made, so
/// the poller keeps what it took.
// @Coroutines closed after the host is built,IMPL_HOST_CLOSE_COROUTINES,impl,[CREQ_HOST_NO_CATCHING]
fn close_coroutines(lua: &Lua) -> LuaResult<()> {
    lua.globals().raw_set("coroutine", LuaNil)
}

/// Hold `thread` to its instruction limit, returning the flag the count sets
/// once it passes the limit.
///
/// A hook every thousand instructions, measured stopping an endless loop
/// (`EVD_LUA_LIMITS_STOP`), and set on the thread the script runs in rather than
/// on the state (`DEC_HOOK_ON_THE_THREAD`). The memory limit is the state's, and
/// held under an asynchronous call. Both reach the script as errors it cannot
/// catch, because nothing that catches one is in its environment
/// (`CREQ_HOST_NO_CATCHING`).
// @Both limits set on every activation,IMPL_HOST_LIMITS,impl,[CREQ_HOST_INSTRUCTION_LIMIT, CREQ_HOST_MEMORY_LIMIT]
fn limit(thread: &LuaThread, limits: Limits) -> LuaResult<Rc<Cell<bool>>> {
    const EVERY: u32 = 1000;
    let over = Rc::new(Cell::new(false));
    let flag = over.clone();
    let spent = Cell::new(0u64);
    thread.set_hook(
        LuaHookTriggers::new().every_nth_instruction(EVERY),
        move |_, _| {
            spent.set(spent.get() + u64::from(EVERY));
            if spent.get() > limits.instructions {
                flag.set(true);
                Err(LuaError::runtime("instruction limit exceeded"))
            } else {
                Ok(LuaVmState::Continue)
            }
        },
    )?;
    Ok(over)
}

/// What the script's run comes to.
///
/// The flags are asked before the error is: every limit, and a failed model
/// call, reach the script as ordinary errors, and their messages are not what
/// says which it was. The instruction limit is asked first, since a script over
/// it may have been anywhere, a model call's aftermath included.
// @A limit is a limit and a failed call is a failed call,IMPL_HOST_OUTCOME,impl,[CREQ_HOST_MODEL_FAILURE, CREQ_HOST_MODEL_CALL_LIMIT, CREQ_HOST_INSTRUCTION_LIMIT]
fn outcome(
    returned: LuaResult<LuaMultiValue>,
    over_instructions: &Cell<bool>,
    calls: &Calls,
) -> Result<Context, ScriptFailure> {
    if over_instructions.get() {
        return Err(ScriptFailure::InstructionLimit);
    }
    if calls.over_limit.get() {
        return Err(ScriptFailure::ModelCallLimit);
    }
    if let Some(failed) = calls.failed.borrow_mut().take() {
        return Err(failed);
    }
    returned.map_err(failure).and_then(one_context)
}

/// The one context `values` holds, or what they held instead.
///
/// Anything else fails, and says what it was: `nil`, a table and two contexts
/// are three different mistakes. A string is not turned into a context, since
/// the type it would be given is a decision the script did not make.
// @Exactly one context or a failure naming what came back,IMPL_HOST_ONE_CONTEXT,impl,[CREQ_HOST_ONE_CONTEXT]
fn one_context(values: LuaMultiValue) -> Result<Context, ScriptFailure> {
    let not_one = |found: String| Err(ScriptFailure::NotOneContext { found });
    let values: Vec<LuaValue> = values.into_iter().collect();
    match values.as_slice() {
        [] => not_one("nothing".to_owned()),
        [LuaValue::UserData(handed)] => match handed.borrow::<Handed>() {
            Ok(handed) => Ok(handed.0.clone()),
            Err(_) => not_one("userdata".to_owned()),
        },
        [one] => not_one(one.type_name().to_owned()),
        several => not_one(format!("{} values", several.len())),
    }
}

/// The failure a Lua error stands for, once the flags have been asked.
///
/// A memory error is the memory limit, whatever raised it. Anything else is the
/// script's own error, kept whole: the document, the line and the traceback are
/// what its author reads.
// @A limit is a limit and an error is an error,IMPL_HOST_FAILURE,impl,[CREQ_HOST_ERROR_CARRIED, CREQ_HOST_MEMORY_LIMIT]
fn failure(error: LuaError) -> ScriptFailure {
    match error {
        LuaError::MemoryError(_) => ScriptFailure::MemoryLimit,
        other => raised(other),
    }
}

/// A Lua error reported as the script's own.
fn raised(error: LuaError) -> ScriptFailure {
    ScriptFailure::Raised {
        message: error.to_string(),
    }
}
#[cfg(test)]
use crate::Behaviours;
#[cfg(test)]
use crate::scripted::{
    CHAIN, CHAIN_TYPES, SMALL, appending, failed, note, rendered, run_with, workflow,
};

/// Two instances ready from the start: `a`, which runs first and whose type is
/// `first`, and `b`, designated, whose type is `second`. A script failing in `a`
/// ends the run before `b` is performed.
#[cfg(test)]
const PAIR_TYPES: &str = r#"
[types.first]
output = "note"

[types.second]
output = "note"

[types.pass]
required = { input = "note" }
output = "note"
"#;

#[cfg(test)]
const PAIR: &str = r#"
name = "pair"
output = "b"

[instances.a]
node_type = "first"

[instances.b]
node_type = "second"
"#;

/// Run the pair with `script` as `a`'s behaviour, and return how `a` failed.
///
/// That the failure is `a`'s is the evidence nothing after it was performed:
/// `b` was ready the whole time, and a run that went on would have ended on it.
#[cfg(test)]
fn first_fails_with(script: &str) -> ScriptFailure {
    let behaviours = Behaviours::new()
        .define("first", "first.lua", script)
        .define(
            "second",
            "second.lua",
            "local given, host = ...\nreturn host.text(host.output, 'b')",
        );
    let (instance, failure) = failed(run_with(&workflow(PAIR_TYPES, PAIR), &behaviours, None));
    assert_eq!(instance, "a", "the run ended on the failing instance");
    failure
}

/// Run `script` as the behaviour of an instance whose output the designated
/// instance passes on, and return what the run rendered.
#[cfg(test)]
fn first_renders(script: &str) -> String {
    let flow = r#"
name = "pair"
output = "b"

[instances.a]
node_type = "first"

[instances.b]
node_type = "pass"
bindings = { input = "a" }
"#;
    let behaviours = Behaviours::new()
        .define("first", "first.lua", script)
        .define(
            "pass",
            "pass.lua",
            "local given, host = ...\nreturn host.compose(host.output, {given.input}, '')",
        );
    rendered(run_with(&workflow(PAIR_TYPES, flow), &behaviours, None))
}

#[cfg(test)]
#[test]
fn inputs_by_parameter_name() {
    // Declared `second` then `first`: neither alphabetical nor the order the
    // script names them in.
    let types = r#"
[types.make]
required = { input = "note" }
output = "note"

[types.join]
required = { second = "note", first = "note" }
output = "note"
"#;
    let flow = r#"
name = "join"
output = "j"

[instances.one]
node_type = "make"
entry = true

[instances.two]
node_type = "make"
entry = true

[instances.j]
node_type = "join"
bindings = { first = "one", second = "two" }
"#;
    let behaviours = Behaviours::new()
        .define(
            "make",
            "make.lua",
            "local given, host = ...\nreturn host.compose(host.output, {given.input}, '')",
        )
        .define(
            "join",
            "join.lua",
            "local given, host = ...\nreturn host.compose(host.output, {given.first, given.second}, '+')",
        );

    let mut source = IdSource::new();
    let arguments = agconflo_core::Arguments::new()
        .supply("one", "input", note(&mut source, "note", "ONE"))
        .supply("two", "input", note(&mut source, "note", "TWO"));
    let ending = crate::scripted::block(crate::run_scripted(
        &workflow(types, flow),
        &behaviours,
        &crate::scripted::offline(),
        arguments,
        &mut source,
        10,
        SMALL,
        |_| {},
    ));
    assert_eq!(rendered(ending), "ONE+TWO");
}

#[cfg(test)]
#[test]
fn unbound_optional_is_absent() {
    let types = r#"
[types.hinted]
optional = { hint = "note" }
output = "note"
"#;
    let flow = r#"
name = "hinted"
output = "h"

[instances.h]
node_type = "hinted"
"#;
    let behaviours = Behaviours::new().define(
        "hinted",
        "hinted.lua",
        "local given, host = ...\nreturn host.text(host.output, given.hint == nil and 'absent' or 'present')",
    );
    assert_eq!(
        rendered(run_with(&workflow(types, flow), &behaviours, None)),
        "absent"
    );
}

#[cfg(test)]
#[test]
fn each_type_runs_its_own_script() {
    let types = format!(
        "{CHAIN_TYPES}\n[types.other]\nrequired = {{ input = \"note\" }}\noutput = \"note\"\n"
    );
    let flow = CHAIN.replace(
        "[instances.third]\nnode_type = \"step\"",
        "[instances.third]\nnode_type = \"other\"",
    );
    assert_ne!(flow, CHAIN, "the third instance's type was changed");
    let behaviours = Behaviours::new()
        .define("seed", "seed.lua", &appending("x"))
        .define("step", "step.lua", &appending("y"))
        .define("other", "other.lua", &appending("z"));

    assert_eq!(
        rendered(run_with(
            &workflow(&types, &flow),
            &behaviours,
            Some(("first", "hello"))
        )),
        "hello x y z"
    );
}

#[cfg(test)]
#[test]
fn output_type_is_given() {
    let types = r#"
[types.summarise]
output = "summary"

[types.judge]
required = { input = "summary" }
output = "verdict"
"#;
    let flow = r#"
name = "typed"
output = "j"

[instances.s]
node_type = "summarise"

[instances.j]
node_type = "judge"
bindings = { input = "s" }
"#;
    // The same script for both, writing no type of its own.
    let script = "local given, host = ...\nreturn host.text(host.output, host.output)";
    let behaviours = Behaviours::new()
        .define("summarise", "s.lua", script)
        .define("judge", "j.lua", script);
    assert_eq!(
        rendered(run_with(&workflow(types, flow), &behaviours, None)),
        "verdict"
    );
}

#[cfg(test)]
#[test]
fn not_one_context_fails() {
    let cases = [
        ("return nil", "nil"),
        ("return", "nothing"),
        (
            "local given, host = ...\nreturn host.text(host.output, 'x'), host.text(host.output, 'y')",
            "2 values",
        ),
        ("return 'plain'", "string"),
        ("return {}", "table"),
    ];
    for (script, found) in cases {
        assert_eq!(
            first_fails_with(script),
            ScriptFailure::NotOneContext {
                found: found.to_owned()
            },
            "{script}"
        );
    }
}

#[cfg(test)]
#[test]
fn refused_output_fails_with_the_refusal() {
    // A type the node type does not declare.
    assert_eq!(
        first_fails_with("local given, host = ...\nreturn host.text('banana', 'x')"),
        ScriptFailure::OutputRefused(OutputRefusal::UndeclaredType {
            instance: "a".to_owned(),
            declared: ContextType::new("note").expect("a name"),
            reported: ContextType::new("banana").expect("a name"),
        })
    );

    // The input handed back: the run holds its identifier.
    let behaviours = Behaviours::new()
        .define("seed", "seed.lua", "local given = ...\nreturn given.input")
        .define("step", "step.lua", &appending("stepped"));
    let (instance, failure) = failed(run_with(
        &workflow(CHAIN_TYPES, CHAIN),
        &behaviours,
        Some(("first", "hello")),
    ));
    assert_eq!(instance, "first");
    assert!(
        matches!(
            failure,
            ScriptFailure::OutputRefused(OutputRefusal::IdentifierHeld { ref instance, .. })
                if instance == "first"
        ),
        "{failure:?}"
    );
}

#[cfg(test)]
#[test]
fn nothing_survives_an_activation() {
    // `second` and `third` are two instances of one type, `step`, so a state kept
    // per type is caught as well as one kept per run.
    let leaky = r#"
local given, host = ...
local seen = tostring(secret) .. '/' .. tostring(string.secret)
secret = 'left'
string.secret = 'left'
return host.compose(host.output, {given.input, host.text(host.output, seen)}, ' ')
"#;
    let behaviours = Behaviours::new()
        .define("seed", "seed.lua", leaky)
        .define("step", "step.lua", leaky);
    assert_eq!(
        rendered(run_with(
            &workflow(CHAIN_TYPES, CHAIN),
            &behaviours,
            Some(("first", "hello"))
        )),
        "hello nil/nil nil/nil nil/nil"
    );
}

#[cfg(test)]
#[test]
fn nothing_reads_outside() {
    let listing = r#"
local given, host = ...
local seen = {}
for _, name in ipairs({'io', 'os', 'require', 'package', 'dofile', 'loadfile', 'load', 'loadstring', 'debug'}) do
  if _G[name] ~= nil then seen[#seen + 1] = name end
end
if math.random ~= nil then seen[#seen + 1] = 'math.random' end
if math.randomseed ~= nil then seen[#seen + 1] = 'math.randomseed' end
return host.text(host.output, 'reachable:' .. table.concat(seen, ','))
"#;
    assert_eq!(first_renders(listing), "reachable:");

    // And reaching for one fails as the script's error, having opened nothing.
    match first_fails_with("local f = io.open('anything')") {
        ScriptFailure::Raised { message } => assert!(message.contains("'io'"), "{message}"),
        other => panic!("expected a script error, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn nothing_catches_an_error() {
    assert_eq!(
        first_renders(
            "local given, host = ...\nreturn host.text(host.output, tostring(pcall) .. tostring(xpcall) .. tostring(coroutine))"
        ),
        "nilnilnil"
    );

    // Each wraps its trouble in whatever catcher it can find, and returns
    // normally if one caught it. With none to find, the limit is what ends it.
    let guarded = |trouble: &str| {
        format!(
            r#"
local given, host = ...
local function trouble() {trouble} end
if pcall then pcall(trouble) return host.text(host.output, 'survived') end
if xpcall then xpcall(trouble, function(e) return e end) return host.text(host.output, 'survived') end
if coroutine then coroutine.resume(coroutine.create(trouble)) return host.text(host.output, 'survived') end
trouble()
"#
        )
    };
    assert_eq!(
        first_fails_with(&guarded("while true do end")),
        ScriptFailure::InstructionLimit
    );
    assert_eq!(
        first_fails_with(&guarded("return string.rep('x', 1 << 30)")),
        ScriptFailure::MemoryLimit
    );
}

#[cfg(test)]
#[test]
fn error_carries_its_message() {
    let script = "local given, host = ...\nlocal unused = 1\nerror('first line\\nsecond line')";
    match first_fails_with(script) {
        ScriptFailure::Raised { message } => {
            assert!(
                message.contains("first.lua:3:"),
                "document and line: {message}"
            );
            assert!(
                message.contains("first line\nsecond line"),
                "both lines: {message}"
            );
        }
        other => panic!("a script's error is not a limit: {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn non_string_error_is_raised() {
    for script in ["error({code = 7})", "error(nil)"] {
        assert!(
            matches!(first_fails_with(script), ScriptFailure::Raised { .. }),
            "{script}"
        );
    }
}

#[cfg(test)]
#[test]
fn runaway_script_is_stopped() {
    assert_eq!(
        first_fails_with("while true do end"),
        ScriptFailure::InstructionLimit
    );
}

/// A script spending `iterations` of an empty loop, then passing its input on.
#[cfg(test)]
fn spending(iterations: u64) -> String {
    format!(
        "local given, host = ...\nfor i = 1, {iterations} do end\nreturn host.compose(host.output, {{given.input}}, '')"
    )
}

#[cfg(test)]
#[test]
fn instruction_limit_is_per_activation() {
    // Calibrated rather than assumed: `spend` iterations fit one activation's
    // limit and twice as many do not, so the three activations of the second
    // run spend more than any limit counted across the run could allow.
    let spend = SMALL.instructions * 2 / 3;
    let chained = |seed: &str| {
        let behaviours = Behaviours::new().define("seed", "seed.lua", seed).define(
            "step",
            "step.lua",
            &spending(spend),
        );
        run_with(
            &workflow(CHAIN_TYPES, CHAIN),
            &behaviours,
            Some(("first", "hello")),
        )
    };
    let (instance, failure) = failed(chained(&spending(spend * 2)));
    assert_eq!(
        (instance.as_str(), failure),
        ("first", ScriptFailure::InstructionLimit)
    );

    assert_eq!(rendered(chained(&spending(spend))), "hello");
}

#[cfg(test)]
#[test]
fn memory_bomb_is_stopped() {
    assert_eq!(
        first_fails_with("return string.rep('x', 1 << 30)"),
        ScriptFailure::MemoryLimit
    );
}

/// A script holding a string of `bytes` while it makes its output.
#[cfg(test)]
fn holding(bytes: usize) -> String {
    format!(
        "local given, host = ...\nlocal held = string.rep('x', {bytes})\nreturn host.compose(host.output, {{given.input}}, '')"
    )
}

#[cfg(test)]
#[test]
fn memory_limit_is_per_activation() {
    // Calibrated as the instruction case is: `hold` fits one activation and
    // twice as much does not, so three activations exceed any shared limit.
    let hold = SMALL.memory * 2 / 3;
    let chained = |seed: &str| {
        let behaviours = Behaviours::new().define("seed", "seed.lua", seed).define(
            "step",
            "step.lua",
            &holding(hold),
        );
        run_with(
            &workflow(CHAIN_TYPES, CHAIN),
            &behaviours,
            Some(("first", "hello")),
        )
    };
    let (instance, failure) = failed(chained(&holding(hold * 2)));
    assert_eq!(
        (instance.as_str(), failure),
        ("first", ScriptFailure::MemoryLimit)
    );

    assert_eq!(rendered(chained(&holding(hold))), "hello");
}

/// Run the pair with `script` as `a`'s behaviour and `roster` for its calls, and
/// return how `a` failed.
#[cfg(test)]
fn first_fails_calling(script: &str, roster: &crate::Roster) -> ScriptFailure {
    let behaviours = Behaviours::new()
        .define("first", "first.lua", script)
        .define(
            "second",
            "second.lua",
            "local given, host = ...\nreturn host.text(host.output, 'b')",
        );
    let (instance, failure) = failed(crate::scripted::run_with_roster(
        &workflow(PAIR_TYPES, PAIR),
        &behaviours,
        roster,
        None,
    ));
    assert_eq!(instance, "a", "the run ended on the failing instance");
    failure
}

/// A roster mapping `drafting` to an OpenAI model at `stub`.
#[cfg(test)]
fn drafting(stub: &crate::models::Stub) -> crate::Roster {
    crate::Roster::new(crate::models::client_for(&stub.base)).map("drafting", "openai::m")
}

#[cfg(test)]
#[test]
fn model_answer_is_a_context() {
    let answer = "  an answer\nwith its whitespace \n";
    let stub = crate::models::Stub::answering(200, answer);
    let script = r#"
local given, host = ...
local named = host.complete('drafting', host.text(host.output, 'first'), 'draft')
local plain = host.complete('drafting', host.text(host.output, 'second'))
return host.compose(host.output, {host.text(host.output, named:type()), named, plain, host.text(host.output, plain:type())}, '|')
"#;
    let behaviours = Behaviours::new().define("first", "first.lua", script);
    let flow = r#"
name = "one"
output = "a"

[instances.a]
node_type = "first"
"#;
    let result = rendered(crate::scripted::run_with_roster(
        &workflow(PAIR_TYPES, flow),
        &behaviours,
        &drafting(&stub),
        None,
    ));
    // Both answers exactly as sent, the first of the type named and the second
    // of the output's declared type - and the output made of them accepted.
    assert_eq!(result, format!("draft|{answer}|{answer}|note"));
    assert_eq!(stub.requests().len(), 2);
}

#[cfg(test)]
#[test]
fn prompt_must_be_a_context() {
    let stub = crate::models::Stub::answering(200, "unused");
    let failure = first_fails_calling(
        "local given, host = ...\nreturn host.complete('drafting', 'a plain string')",
        &drafting(&stub),
    );
    assert!(
        matches!(failure, ScriptFailure::Raised { .. }),
        "{failure:?}"
    );
    assert!(stub.requests().is_empty(), "no call was made");
}

#[cfg(test)]
#[test]
fn model_call_limit_holds() {
    let stub = crate::models::Stub::answering(200, "ok");
    let asking = |times: u32| {
        format!(
            "local given, host = ...\nlocal last\nfor i = 1, {times} do last = host.complete('drafting', host.text(host.output, 'q')) end\nreturn host.compose(host.output, {{given.input, last}}, ' ')"
        )
    };

    // Three calls against a limit of two: the third is refused before it is
    // made, as the limit rather than as a script error.
    let behaviours = Behaviours::new()
        .define("seed", "seed.lua", &asking(SMALL.model_calls + 1))
        .define("step", "step.lua", &asking(1));
    let (instance, failure) = failed(crate::scripted::run_with_roster(
        &workflow(CHAIN_TYPES, CHAIN),
        &behaviours,
        &drafting(&stub),
        Some(("first", "hello")),
    ));
    assert_eq!(
        (instance.as_str(), failure),
        ("first", ScriptFailure::ModelCallLimit)
    );
    assert_eq!(stub.requests().len(), 2);

    // Two calls in each of three activations: each activation's own count.
    let behaviours = Behaviours::new()
        .define("seed", "seed.lua", &asking(SMALL.model_calls))
        .define("step", "step.lua", &asking(SMALL.model_calls));
    let result = rendered(crate::scripted::run_with_roster(
        &workflow(CHAIN_TYPES, CHAIN),
        &behaviours,
        &drafting(&stub),
        Some(("first", "hello")),
    ));
    assert_eq!(result, "hello ok ok ok");
    assert_eq!(stub.requests().len(), 2 + 6);
}

#[cfg(test)]
#[test]
fn model_failure_ends_the_activation() {
    let stub = crate::models::Stub::answering(503, "unused");
    let failure = first_fails_calling(
        "local given, host = ...\nreturn host.complete('drafting', host.text(host.output, 'q'))",
        &drafting(&stub),
    );
    match failure {
        ScriptFailure::ModelFailed(crate::ModelFailure::Provider { role, status, .. }) => {
            assert_eq!(role, "drafting");
            assert_eq!(status, Some(503));
        }
        other => panic!("a failed call is a model failure, not {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn instruction_limit_holds_across_a_model_call() {
    let stub = crate::models::Stub::answering(200, "ok");
    let failure = first_fails_calling(
        "local given, host = ...\nhost.complete('drafting', host.text(host.output, 'q'))\nwhile true do end",
        &drafting(&stub),
    );
    assert_eq!(failure, ScriptFailure::InstructionLimit);
    assert_eq!(
        stub.requests().len(),
        1,
        "the call was made before the loop"
    );
}

#[cfg(test)]
#[test]
fn memory_limit_holds_across_a_model_call() {
    let stub = crate::models::Stub::answering(200, "ok");
    let failure = first_fails_calling(
        "local given, host = ...\nhost.complete('drafting', host.text(host.output, 'q'))\nreturn string.rep('x', 1 << 30)",
        &drafting(&stub),
    );
    assert_eq!(failure, ScriptFailure::MemoryLimit);
    assert_eq!(stub.requests().len(), 1);
}
