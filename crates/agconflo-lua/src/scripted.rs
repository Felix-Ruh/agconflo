//! A run driven from its start to its ending by scripts: each activation the run
//! offers is performed by the script host, and what came of it is reported back
//! - or, for a step a person performs, handed to the caller to answer later.
//!
//! Not a component of its own. `ARCH_BEHAVIOUR` gives the reason, the one
//! `ARCH_RUN` gave for the activation it dropped: nothing about driving a run is
//! true or false taken alone, and every requirement it could carry is already
//! the run's, the behaviour set's or the host's.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use agconflo_core::{
    Activation, Arguments, Context, IdSource, NodeType, NothingOutstanding, ResumeRefusal, Run,
    RunEnding, StartRefusal, Step, WorkflowDefinition,
};

use crate::behaviours::{BehaviourFault, Behaviours};
use crate::host::{self, Limits, ScriptFailure};
use crate::models::Roster;

/// Why a scripted run was not started, not resumed, or not given an answer.
///
/// Asked in order: whatever the run itself refuses a workflow or its arguments
/// for - or, resuming, whatever it refuses a record for - then the scripts, and
/// last, when a person's answer was supplied, whether the run awaits it.
/// The scripts are asked about only once the run would start, because a
/// workflow with a wiring defect may name a node type that does not exist, and
/// whether it has a script is not a question worth answering first.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScriptedRefusal {
    /// The run refused to start: the workflow's wiring or its arguments.
    Start(StartRefusal),
    /// The scripts cannot run, every fault of them
    /// (`CREQ_BEHAVIOURS_EVERY_FAULT`).
    Behaviours(Vec<BehaviourFault>),
    /// The run refused to resume from the record it was given
    /// (`FEAT_RESUME_REFUSES_ANOTHER_RUN`).
    Resume(ResumeRefusal),
    /// A person's text was supplied for an instance whose step the record's
    /// run does not await (`CREQ_HOST_REFUSES_ANSWER_ELSEWHERE`). Nothing was
    /// run and no record handed over.
    NotAwaited {
        /// The instance the text was supplied for.
        answered: String,
        /// The instance whose activation the run offers next, or `None` when
        /// it has ended.
        offered: Option<String>,
    },
}

impl fmt::Display for ScriptedRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start(refusal) => refusal.fmt(f),
            Self::Behaviours(faults) => {
                write!(f, "the scripts carry {} fault", faults.len())?;
                if faults.len() != 1 {
                    f.write_str("s")?;
                }
                Ok(())
            }
            Self::Resume(refusal) => refusal.fmt(f),
            Self::NotAwaited { answered, offered } => {
                write!(f, "the run does not await {answered}")?;
                match offered {
                    Some(offered) => write!(f, "; it offers {offered} next"),
                    None => f.write_str("; it has ended"),
                }
            }
        }
    }
}

impl std::error::Error for ScriptedRefusal {}

/// Where a scripted run stopped: at its ending, or at a step a person performs.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// The run ended, and this is how (`DEC_RUN_ENDS_ONE_WAY`).
    Ended(RunEnding<ScriptFailure>),
    /// The run awaits a person's answer for this activation
    /// (`FEAT_PERSON_STEP_HANDED_OVER`): which instance, what it was given, and
    /// the type of context it produces. The run has not ended and is not stuck
    /// (`FEAT_PERSON_WAIT_NOT_STUCK`); the answer goes to [`answer_scripted`]
    /// with the last record the run handed over.
    Awaiting(Activation),
}

/// Run `definition` to its ending, performing each activation with the script
/// `behaviours` gives its node type - or refuse to start it.
///
/// Stops early, returning [`Outcome::Awaiting`], at the first activation of a
/// node type `behaviours` names as performed by a person: nothing is run for it,
/// and the run is as the last record it handed over describes.
///
/// Nothing is performed before every refusal has been asked: the run's own,
/// then the scripts' (`FEAT_BEHAVIOUR_REFUSED_BEFORE_START`). After that the run
/// decides what activates and how it ends, exactly as for any caller, and a
/// script's failure is carried in the run's ending as a [`ScriptFailure`].
///
/// Model calls go through `roster`, which maps the roles scripts name to the
/// caller's models (`DEC_MODELS_BY_ROLE`); a run whose scripts call no model can
/// be given a roster mapping nothing.
///
/// `keep` is handed the run's record once it starts, again each time it accepts
/// an output, and again each time a model call is answered
/// (`DEC_RECORD_AFTER_EACH_ANSWER`). A caller keeping the latest can give it to
/// [`resume_scripted`] after an interruption and lose at most the script's own
/// work since the last answer, which is run again, and no answer it paid for;
/// one that needs no record ignores it.
///
/// Asynchronous, because a model call is awaited (`DEC_BEHAVIOUR_ASYNC`). The
/// future is not `Send` - a Lua state is not (`EVD_RUN_IS_SEND`) - so it runs on
/// the thread that polls it: a current-thread runtime, or a local set. Dropping
/// it interrupts the run, and nothing after the drop runs.
///
/// `source` issues the identifiers of every context the scripts make, and must
/// be the one `arguments` were made from: two sources repeat each other's
/// identifiers, and the run refuses an output carrying one it already holds.
/// It is lent to the scripts for the length of the run and handed back when it
/// ends, however it ends.
// @Refused before anything runs,IMPL_SCRIPTED_REFUSAL,impl,[CREQ_BEHAVIOURS_REFUSE_MISSING, CREQ_BEHAVIOURS_REFUSE_UNCOMPILABLE, CREQ_BEHAVIOURS_REFUSE_TWICE]
#[allow(clippy::too_many_arguments)]
pub async fn run_scripted(
    definition: &WorkflowDefinition,
    behaviours: &Behaviours,
    roster: &Roster,
    arguments: Arguments,
    source: &mut IdSource,
    budget: usize,
    limits: Limits,
    keep: impl FnMut(String),
) -> Result<Outcome, ScriptedRefusal> {
    let run = Run::start(definition, arguments, budget).map_err(ScriptedRefusal::Start)?;
    drive(
        run, definition, behaviours, roster, source, limits, None, keep,
    )
    .await
}

/// Resume the run `record` is a record of, against `definition`, and run it to
/// its ending as [`run_scripted`] would have - or refuse to.
///
/// No activation whose output the record holds is performed again
/// (`FEAT_RESUME_REPEATS_NO_OUTPUT`): the run is rebuilt from the record, and
/// the first activation performed is the one the recorded run had not
/// finished. The record is refused before any script runs if it does not
/// describe a run of `definition` (`FEAT_RESUME_REFUSES_ANOTHER_RUN`), and the
/// scripts are asked about as for a start.
///
/// Hands back where the run stopped with the identifier source that came back
/// with the run, advanced past everything the resumed run issued, since
/// whatever the caller makes next must continue from it. A record whose run
/// awaits a person stops at once, handing that step back again.
pub async fn resume_scripted(
    definition: &WorkflowDefinition,
    behaviours: &Behaviours,
    roster: &Roster,
    record: &str,
    limits: Limits,
    keep: impl FnMut(String),
) -> Result<(Outcome, IdSource), ScriptedRefusal> {
    let (run, mut source) = Run::resume(definition, record).map_err(ScriptedRefusal::Resume)?;
    let outcome = drive(
        run,
        definition,
        behaviours,
        roster,
        &mut source,
        limits,
        None,
        keep,
    )
    .await?;
    Ok((outcome, source))
}

/// Resume the run `record` is a record of with `text` as the output of the step
/// a person performs for `instance`, and run it on as [`resume_scripted`]
/// would - or refuse to.
///
/// The text becomes a context of the type that step declares, issued by the
/// source resumed with the record (`DEC_PERSON_SUPPLIES_TEXT`), and is kept
/// exactly as given. The run then hands its caller a record, as after any
/// output, and runs on to its ending or to the next step a person performs.
///
/// Refused, with nothing run and no record handed over, unless the record's
/// run next offers an activation of `instance` by a node type a person
/// performs (`CREQ_HOST_REFUSES_ANSWER_ELSEWHERE`) - asked after everything a
/// resume refuses, and after the scripts.
///
/// Answering one record twice gives two runs sharing identifiers, each sound
/// on its own. Keeping to one answer per record is the caller's, since the
/// caller is what keeps records (`DEC_ANSWER_ONCE_BY_THE_KEEPER`).
#[allow(clippy::too_many_arguments)]
pub async fn answer_scripted(
    definition: &WorkflowDefinition,
    behaviours: &Behaviours,
    roster: &Roster,
    record: &str,
    instance: &str,
    text: &str,
    limits: Limits,
    keep: impl FnMut(String),
) -> Result<(Outcome, IdSource), ScriptedRefusal> {
    let (run, mut source) = Run::resume(definition, record).map_err(ScriptedRefusal::Resume)?;
    let outcome = drive(
        run,
        definition,
        behaviours,
        roster,
        &mut source,
        limits,
        Some((instance, text)),
        keep,
    )
    .await?;
    Ok((outcome, source))
}

/// Perform `run`'s activations until it ends or reaches a step a person
/// performs, handing `keep` a record when it starts and after each accepted
/// output - and, through [`serve`], after each answer - or refuse it for its
/// scripts, or for an answer it does not await.
///
/// `answer`, when given, is the instance a person's text was supplied for and
/// the text, taken as the output of the first activation the run offers.
// @A record handed over at the start and after each output,IMPL_SCRIPTED_RECORDS,impl,[CREQ_HOST_HANDS_RECORDS]
#[allow(clippy::too_many_arguments)]
async fn drive(
    mut run: Run<'_, ScriptFailure>,
    definition: &WorkflowDefinition,
    behaviours: &Behaviours,
    roster: &Roster,
    source: &mut IdSource,
    limits: Limits,
    answer: Option<(&str, &str)>,
    mut keep: impl FnMut(String),
) -> Result<Outcome, ScriptedRefusal> {
    let lent = Lent::new(source);

    let faults = behaviours.faults(definition);
    if !faults.is_empty() {
        return Err(ScriptedRefusal::Behaviours(faults));
    }

    if let Some((instance, text)) = answer {
        let reported = answered(&mut run, behaviours, &lent, instance, text)?;
        if let Err(failure) = reported {
            return Ok(Outcome::Ended(fail(run, failure)));
        }
    }

    // The position is read from the loan, which is the source the scripts
    // draw from: what stands in the caller's place meanwhile is a fresh one.
    let mut env = Env {
        definition,
        behaviours,
        roster,
        limits,
        source: lent.source.clone(),
        keep: &mut keep,
    };
    (env.keep)(run.record(&env.source.borrow()));
    loop {
        let activation = match run.step() {
            Step::Ended(ending) => return Ok(Outcome::Ended(ending)),
            Step::Activate(activation) => activation,
        };

        // @A person's step handed to the caller with nothing run for it,IMPL_SCRIPTED_PERSON_STEP,impl,[CREQ_HOST_HANDS_OVER_PERSON_STEP]
        if behaviours.performed_by_person(activation.node_type()) {
            return Ok(Outcome::Awaiting(activation));
        }

        match perform_activation(&mut run, &activation, &mut env).await {
            Performed::Output(output) => {
                if let Err(refusal) = run.produced(output) {
                    return Ok(Outcome::Ended(fail(
                        run,
                        ScriptFailure::OutputRefused(refusal),
                    )));
                }
                (env.keep)(run.record(&env.source.borrow()));
            }
            Performed::Stopped(Stop::Failed(failure)) => {
                return Ok(Outcome::Ended(fail(run, failure)));
            }
            Performed::Stopped(Stop::Awaiting(step)) => return Ok(Outcome::Awaiting(step)),
            Performed::Stopped(Stop::Ended(ending)) => return Ok(Outcome::Ended(ending)),
        }
    }
}

/// What a scripted run is driven with, besides the run itself.
struct Env<'e> {
    definition: &'e WorkflowDefinition,
    behaviours: &'e Behaviours,
    roster: &'e Roster,
    limits: Limits,
    source: Rc<RefCell<IdSource>>,
    keep: &'e mut dyn FnMut(String),
}

/// How performing an activation came out.
enum Performed {
    /// Its output, not yet reported to the run.
    Output(Context),
    /// The run cannot go on from it as it would from an output.
    Stopped(Stop),
}

/// Why performing an activation stopped the run, or where.
enum Stop {
    /// The activation failed, with the run's outstanding activation - the
    /// called one, when a call failed - still to be failed with it.
    Failed(ScriptFailure),
    /// A call reached a node type a person performs: the run awaits them.
    Awaiting(Activation),
    /// A call found the budget spent, and the run ended.
    Ended(RunEnding<ScriptFailure>),
}

/// Perform `activation` by running its node type's script, answering from `run`
/// whatever the script asks of it while it runs.
///
/// The script is polled here, and whenever it waits on the mailbox the question
/// is answered with the run this holds: an exchange held and a record handed
/// over, or a call performed as the run's next activation - the called node
/// type's own script performed by this same function, or its step handed to a
/// person (`DEC_CALL_IS_AN_ACTIVATION`). A script waiting on anything else - a
/// provider - is waited on.
///
/// What the activation's record holds is handed to the script to be answered
/// from (`DEC_SCRIPT_REPLAYED_FROM_ITS_RECORD`), and what its model may call is
/// what its instance declares - nothing, for a call's own activation.
///
/// Boxed, because performing a call's activation is this function again.
// @A script's questions answered from the run it is performed for,IMPL_SCRIPTED_YIELD,impl,[CREQ_HOST_PERFORMS_CALLS, CREQ_HOST_RECORD_AFTER_ANSWER, CREQ_HOST_CALL_REFUSAL_CARRIED, CREQ_HOST_HANDS_OVER_PERSON_STEP]
fn perform_activation<'f>(
    run: &'f mut Run<'_, ScriptFailure>,
    activation: &'f Activation,
    env: &'f mut Env<'_>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Performed> + 'f>> {
    Box::pin(async move {
        // The behaviour set has refused the run unless each type it names has
        // a person or exactly one script, and not both; a person's step never
        // reaches here.
        let script = env
            .behaviours
            .script_for(activation.node_type())
            .expect("every named type a person does not perform has exactly one script");
        let callees = callees_of(env.definition, activation);
        let mut replay = host::Replay {
            exchanges: run.exchanges().to_vec(),
            ..host::Replay::default()
        };
        for exchange in &replay.exchanges {
            for call in exchange.calls() {
                if let Some(output) = run.called(call.id()) {
                    replay.outputs.insert(call.id().to_owned(), output.clone());
                }
            }
        }
        let mailbox = Rc::new(host::Mailbox::default());
        let performing = host::Performing {
            callees,
            replay,
            mailbox: mailbox.clone(),
        };

        let source = env.source.clone();
        let roster = env.roster.clone();
        let mut script = std::pin::pin!(host::perform(
            script, activation, &source, &roster, env.limits, performing
        ));
        let mut stop = None;
        let performed = loop {
            let polled = std::future::poll_fn(|cx| match script.as_mut().poll(cx) {
                std::task::Poll::Ready(done) => std::task::Poll::Ready(Some(done)),
                std::task::Poll::Pending if mailbox.asking.borrow().is_some() => {
                    std::task::Poll::Ready(None)
                }
                std::task::Poll::Pending => std::task::Poll::Pending,
            })
            .await;
            match polled {
                Some(done) => break done,
                None => {
                    let asking = mailbox.asking.borrow_mut().take();
                    let given = match asking {
                        Some(asking) => serve(run, asking, env, &mut stop).await,
                        None => host::Given::Stop,
                    };
                    *mailbox.given.borrow_mut() = Some(given);
                }
            }
        };

        match (stop, performed) {
            (Some(stop), _) => Performed::Stopped(stop),
            (None, Ok(output)) => Performed::Output(output),
            (None, Err(failure)) => Performed::Stopped(Stop::Failed(failure)),
        }
    })
}

/// Answer one thing a script asked of `run`, noting in `stop` why the run cannot
/// go on when that is the answer.
async fn serve(
    run: &mut Run<'_, ScriptFailure>,
    asking: host::Asking,
    env: &mut Env<'_>,
    stop: &mut Option<Stop>,
) -> host::Given {
    let mut stopping = |why: Stop| {
        *stop = Some(why);
        host::Given::Stop
    };
    match asking {
        host::Asking::Exchange(exchange) => match run.exchange(exchange) {
            Ok(()) => {
                (env.keep)(run.record(&env.source.borrow()));
                host::Given::Held
            }
            Err(refusal) => stopping(Stop::Failed(ScriptFailure::ExchangeRefused(refusal))),
        },
        host::Asking::Call(call) => {
            if let Err(refusal) = run.call(call) {
                return stopping(Stop::Failed(ScriptFailure::CallRefused(refusal)));
            }
            let called = match run.step() {
                Step::Ended(ending) => return stopping(Stop::Ended(ending)),
                Step::Activate(called) => called,
            };
            if env.behaviours.performed_by_person(called.node_type()) {
                // Handed over with a record holding the call, which is what the
                // person's answer resumes from.
                (env.keep)(run.record(&env.source.borrow()));
                return stopping(Stop::Awaiting(called));
            }
            match perform_activation(run, &called, env).await {
                Performed::Output(output) => match run.produced(output.clone()) {
                    Ok(()) => {
                        (env.keep)(run.record(&env.source.borrow()));
                        host::Given::Output(output)
                    }
                    Err(refusal) => stopping(Stop::Failed(ScriptFailure::OutputRefused(refusal))),
                },
                Performed::Stopped(why) => stopping(why),
            }
        }
    }
}

/// The node types the model performing `activation` may call: those its
/// instance declares, each once, in the order declared - and none for a call's
/// own activation, which has no instance of its own.
fn callees_of(definition: &WorkflowDefinition, activation: &Activation) -> Vec<NodeType> {
    if activation.call().is_some() {
        return Vec::new();
    }
    let declared = definition
        .instances
        .iter()
        .find(|instance| instance.name == activation.instance())
        .map(|instance| instance.calls.clone())
        .unwrap_or_default();
    let mut callees: Vec<NodeType> = Vec::new();
    for name in declared {
        if callees.iter().any(|callee| callee.name == name) {
            continue;
        }
        if let Some(callee) = definition.node_types.iter().find(|t| t.name == name) {
            callees.push(callee.clone());
        }
    }
    callees
}

/// Report `text` as the output of the step a person performs for `instance`,
/// when that is the activation `run` offers - or refuse it, having reported
/// nothing.
///
/// The outer error is the refusal. The inner one is the step's failure, when
/// its output cannot be made or the run refuses it, and ends the run as a
/// script's failure would.
// @A person's text taken as the awaited step's output,IMPL_SCRIPTED_ANSWER,impl,[CREQ_HOST_TAKES_PERSON_TEXT, CREQ_HOST_REFUSES_ANSWER_ELSEWHERE]
fn answered(
    run: &mut Run<'_, ScriptFailure>,
    behaviours: &Behaviours,
    lent: &Lent<'_>,
    instance: &str,
    text: &str,
) -> Result<Result<(), ScriptFailure>, ScriptedRefusal> {
    let refused = |offered: Option<&str>| ScriptedRefusal::NotAwaited {
        answered: instance.to_owned(),
        offered: offered.map(str::to_owned),
    };
    let activation = match run.step() {
        Step::Ended(_) => return Err(refused(None)),
        Step::Activate(activation) => activation,
    };
    if activation.instance() != instance || !behaviours.performed_by_person(activation.node_type())
    {
        return Err(refused(Some(activation.instance())));
    }

    let made = Context::text(
        &mut lent.source.borrow_mut(),
        activation.output().clone(),
        text,
    );
    Ok(match made {
        Err(exhausted) => Err(ScriptFailure::SourceExhausted(exhausted)),
        Ok(output) => run.produced(output).map_err(ScriptFailure::OutputRefused),
    })
}

/// The caller's identifier source, lent to the scripts for the length of a run.
///
/// Each host function owns its handle to the source rather than borrowing it
/// for a scope, since a model call is awaited and a scoped function cannot be.
/// Dropping the loan puts the source back where it came from, advanced past
/// every identifier the run issued - on every way out of the run, an early
/// return included.
struct Lent<'a> {
    home: &'a mut IdSource,
    source: Rc<RefCell<IdSource>>,
}

impl<'a> Lent<'a> {
    fn new(home: &'a mut IdSource) -> Self {
        let source = Rc::new(RefCell::new(std::mem::take(home)));
        Self { home, source }
    }
}

impl Drop for Lent<'_> {
    fn drop(&mut self) {
        *self.home = std::mem::take(&mut *self.source.borrow_mut());
    }
}

/// End `run` with `failure` for the activation it has outstanding.
///
/// A refused output is not answered again. The run would take another answer
/// (`DEC_REFUSED_OUTPUT_OUTSTANDING`), but a script run twice on the same inputs
/// has no reason to give a different one, and a refusal is not counted against
/// the budget - so trying again is a loop nothing ends
/// (`CREQ_HOST_OUTPUT_REFUSAL_CARRIED`).
// @A refused output fails its activation,IMPL_SCRIPTED_FAIL,impl,[CREQ_HOST_OUTPUT_REFUSAL_CARRIED]
fn fail(run: Run<'_, ScriptFailure>, failure: ScriptFailure) -> RunEnding<ScriptFailure> {
    match run.fail(failure) {
        Ok(ending) => ending,
        Err(NothingOutstanding) => {
            unreachable!("the activation just performed is outstanding until it is reported")
        }
    }
}

// --- test builders -----------------------------------------------------------
// Used by every module's tests, so they live beside the function they drive.

#[cfg(test)]
use agconflo_core::{ContextType, TypeCatalogue, read_node_types, read_workflow};

/// The definition `flow` describes, over the node types `types` declares - both
/// TOML documents, read the way a caller would read them.
#[cfg(test)]
pub(crate) fn workflow(types: &str, flow: &str) -> WorkflowDefinition {
    let types = read_node_types("types.toml", types).expect("the node types read");
    let catalogue = TypeCatalogue::gather([types]).expect("each type declared once");
    read_workflow("flow.toml", flow, &catalogue)
        .expect("the workflow reads")
        .0
}

/// Limits small enough that a case meeting one does so in milliseconds.
#[cfg(test)]
pub(crate) const SMALL: Limits = Limits {
    instructions: 200_000,
    memory: 2 << 20,
    model_calls: 2,
};

/// Drive `future` to its end on a runtime of its own, on this thread.
///
/// Current-thread because a scripted run is not `Send` (`EVD_RUN_IS_SEND`), and
/// with its drivers enabled because a model call reaches the network - a stub
/// on the loopback interface, in these tests.
#[cfg(test)]
pub(crate) fn block<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a runtime")
        .block_on(future)
}

/// A roster mapping no role, for runs whose scripts call no model.
///
/// Built once per thread and cloned. Building a client reaches no network, and
/// is still not free: on Linux it took about 36 ms, measured in a container with
/// four CPUs, where Windows was far cheaper. Built afresh for every case, the
/// property tests that run a workflow hundreds of times took 19 s and 10 s
/// there, against 0.6 s and 0.5 s with one client, and the first was killed at
/// the 20 s limit in CI.
#[cfg(test)]
pub(crate) fn offline() -> Roster {
    thread_local! {
        static OFFLINE: Roster = Roster::new(genai::Client::builder().build().expect("a client"));
    }
    OFFLINE.with(Roster::clone)
}

/// A context of type `declared` holding `text`, from `source`.
#[cfg(test)]
pub(crate) fn note(source: &mut IdSource, declared: &str, text: &str) -> Context {
    let declared = ContextType::new(declared).expect("a non-empty type name");
    Context::text(source, declared, text).expect("a fresh source issues")
}

/// Run `definition` with `behaviours` under the small limits, its one entry
/// instance `entry` given `argument` for its parameter `input`, and no model.
#[cfg(test)]
pub(crate) fn run_with(
    definition: &WorkflowDefinition,
    behaviours: &Behaviours,
    entry: Option<(&str, &str)>,
) -> Result<Outcome, ScriptedRefusal> {
    run_with_roster(definition, behaviours, &offline(), entry)
}

/// As [`run_with`], with the scripts' model calls going through `roster`.
#[cfg(test)]
pub(crate) fn run_with_roster(
    definition: &WorkflowDefinition,
    behaviours: &Behaviours,
    roster: &Roster,
    entry: Option<(&str, &str)>,
) -> Result<Outcome, ScriptedRefusal> {
    let mut source = IdSource::new();
    let mut arguments = Arguments::new();
    if let Some((instance, text)) = entry {
        let argument = note(&mut source, "note", text);
        arguments = arguments.supply(instance, "input", argument);
    }
    block(run_scripted(
        definition,
        behaviours,
        roster,
        arguments,
        &mut source,
        20,
        SMALL,
        |_| {},
    ))
}

/// What a completed run rendered, or a panic naming how it ended instead.
#[cfg(test)]
pub(crate) fn rendered(ending: Result<Outcome, ScriptedRefusal>) -> String {
    match ending {
        Ok(Outcome::Ended(RunEnding::Completed(result))) => result.render().into_owned(),
        other => panic!("expected the run to complete, got {other:?}"),
    }
}

/// The failure a run ended on and the instance it was reported for, or a panic
/// naming how it ended instead.
#[cfg(test)]
pub(crate) fn failed(ending: Result<Outcome, ScriptedRefusal>) -> (String, ScriptFailure) {
    match ending {
        Ok(Outcome::Ended(RunEnding::NodeFailed { instance, failure })) => (instance, failure),
        other => panic!("expected an activation to fail, got {other:?}"),
    }
}

/// A script that passes its input on, composed with a word of its own.
#[cfg(test)]
pub(crate) fn appending(word: &str) -> String {
    format!(
        "local given, host = ...\nreturn host.compose(host.output, {{given.input, host.text(host.output, '{word}')}}, ' ')"
    )
}

/// Node types for a chain: `seed` takes the run's argument, `step` passes its
/// input on. Both are `note` in and `note` out.
#[cfg(test)]
pub(crate) const CHAIN_TYPES: &str = "\
[types.seed]
required = { input = \"note\" }
output = \"note\"

[types.step]
required = { input = \"note\" }
output = \"note\"
";

/// `first`, an entry `seed`, followed by `second` and `third`, each a `step`
/// bound to the one before; `third` is designated.
#[cfg(test)]
pub(crate) const CHAIN: &str = "\
name = \"chain\"
output = \"third\"

[instances.first]
node_type = \"seed\"
entry = true

[instances.second]
node_type = \"step\"
bindings = { input = \"first\" }

[instances.third]
node_type = \"step\"
bindings = { input = \"second\" }
";

#[cfg(test)]
#[test]
fn arguments_from_another_source_fail() {
    let definition = workflow(CHAIN_TYPES, CHAIN);

    // Two scripts, two shapes. The first makes its output directly, so the
    // output itself repeats the argument's identifier. The second composes its
    // input with a word, so the word - a part - repeats it and the output does
    // not; that one the run once accepted, measured
    // (`EVD_RUN_PART_SHARES_IDENTIFIER`).
    let direct = "local given, host = ...\nreturn host.text(host.output, 'made')";
    for (script, shared_in_a_part) in [(direct, false), (appending("seeded").as_str(), true)] {
        let behaviours = Behaviours::new().define("seed", "seed.lua", script).define(
            "step",
            "step.lua",
            &appending("stepped"),
        );

        // The argument is made from one source and the run given a fresh other,
        // so the first context a script makes repeats the argument's identifier.
        let argument = note(&mut IdSource::new(), "note", "hello");
        let held = argument.id();
        let arguments = Arguments::new().supply("first", "input", argument);
        let ending = block(run_scripted(
            &definition,
            &behaviours,
            &offline(),
            arguments,
            &mut IdSource::new(),
            20,
            SMALL,
            |_| {},
        ));

        let (instance, failure) = failed(ending);
        assert_eq!(instance, "first");
        let refusal = if shared_in_a_part {
            agconflo_core::OutputRefusal::IdentifierShared {
                instance: "first".to_owned(),
                id: held,
            }
        } else {
            agconflo_core::OutputRefusal::IdentifierHeld {
                instance: "first".to_owned(),
                id: held,
            }
        };
        assert_eq!(failure, ScriptFailure::OutputRefused(refusal), "{script}");
    }
}

#[cfg(test)]
#[test]
fn changed_script_changes_the_result() {
    let definition = workflow(CHAIN_TYPES, CHAIN);
    let with = |word: &str| {
        Behaviours::new()
            .define("seed", "seed.lua", &appending("seeded"))
            .define("step", "step.lua", &appending(word))
    };

    // One build, one workflow, two scripts for `step`: two results, differing
    // exactly as the scripts do.
    let one = rendered(run_with(
        &definition,
        &with("one"),
        Some(("first", "hello")),
    ));
    let two = rendered(run_with(
        &definition,
        &with("two"),
        Some(("first", "hello")),
    ));
    assert_eq!(one, "hello seeded one one");
    assert_eq!(two, "hello seeded two two");
}

#[cfg(test)]
proptest::proptest! {
    /// For any chain whose scripts each append a word of their own, the run
    /// completes and its result is the argument followed by every word in the
    /// order the chain ran.
    #[test]
    fn workflows_run_to_completion(
        argument in "[a-z]{1,8}",
        words in proptest::collection::vec("[a-z0-9]{1,8}", 1..6),
    ) {
        let mut types = String::new();
        let mut flow = format!("name = \"chain\"\noutput = \"n{}\"\n", words.len() - 1);
        let mut behaviours = Behaviours::new();
        for (index, word) in words.iter().enumerate() {
            types += &format!(
                "[types.t{index}]\nrequired = {{ input = \"note\" }}\noutput = \"note\"\n"
            );
            flow += &format!("\n[instances.n{index}]\nnode_type = \"t{index}\"\n");
            if index == 0 {
                flow += "entry = true\n";
            } else {
                flow += &format!("bindings = {{ input = \"n{}\" }}\n", index - 1);
            }
            behaviours = behaviours.define(&format!("t{index}"), &format!("t{index}.lua"), &appending(word));
        }

        let definition = workflow(&types, &flow);
        let result = rendered(run_with(&definition, &behaviours, Some(("n0", &argument))));
        proptest::prop_assert_eq!(result, format!("{argument} {}", words.join(" ")));
    }
}

#[cfg(test)]
#[test]
fn same_workflow_two_providers() {
    let stub = crate::models::Stub::answering(200, "an answer");
    let calling = "local given, host = ...\nlocal answer = host.complete('drafting', given.input)\nreturn host.compose(host.output, {given.input, answer}, ' / ')";
    let behaviours = Behaviours::new()
        .define("seed", "seed.lua", calling)
        .define("step", "step.lua", &appending("stepped"));
    let definition = workflow(CHAIN_TYPES, CHAIN);

    // Nothing in the workflow or its scripts changes between the two runs;
    // only the caller's roster does.
    let mut paths = Vec::new();
    for model in ["openai::gpt-draft", "anthropic::claude-draft"] {
        let roster = Roster::new(crate::models::client_for(&stub.base)).map("drafting", model);
        let result = rendered(run_with_roster(
            &definition,
            &behaviours,
            &roster,
            Some(("first", "hello")),
        ));
        assert_eq!(result, "hello / an answer stepped stepped");
        paths.push(stub.requests().last().expect("a request").0.clone());
    }
    assert_eq!(paths, ["/v1/chat/completions", "/v1/messages"]);
}

#[cfg(test)]
#[test]
fn source_handed_back_advanced() {
    let definition = workflow(CHAIN_TYPES, CHAIN);
    let behaviours = Behaviours::new()
        .define("seed", "seed.lua", &appending("seeded"))
        .define("step", "step.lua", &appending("stepped"));
    let mut source = IdSource::new();
    let argument = note(&mut source, "note", "hello");
    let arguments = Arguments::new().supply("first", "input", argument);

    let ending = block(run_scripted(
        &definition,
        &behaviours,
        &offline(),
        arguments,
        &mut source,
        20,
        SMALL,
        |_| {},
    ));
    let Outcome::Ended(RunEnding::Completed(result)) = ending.expect("starts") else {
        panic!("expected completion")
    };

    // What the caller's source issues next is new to everything the run made:
    // handed back reset, it would repeat the argument's identifier and the
    // run's own.
    let after = note(&mut source, "note", "later");
    let mut issued: Vec<_> = result.lineage().iter().map(|c| c.id()).collect();
    issued.push(result.id());
    assert!(
        !issued.contains(&after.id()),
        "{:?} was issued in the run",
        after.id()
    );
}

#[cfg(test)]
#[test]
fn records_handed_over() {
    let definition = workflow(CHAIN_TYPES, CHAIN);
    let broken = "local given, host = ...\nerror('broken')";
    for (step, expected) in [(appending("stepped"), 4), (broken.to_owned(), 2)] {
        let behaviours = Behaviours::new()
            .define("seed", "seed.lua", &appending("seeded"))
            .define("step", "step.lua", &step);
        let mut source = IdSource::new();
        let argument = note(&mut source, "note", "hello");
        let mut records = Vec::new();
        let ending = block(run_scripted(
            &definition,
            &behaviours,
            &offline(),
            Arguments::new().supply("first", "input", argument),
            &mut source,
            20,
            SMALL,
            |record| records.push(record),
        ));
        assert!(ending.is_ok(), "{ending:?}");

        // One at the start and one after each accepted output, the one after
        // the i-th holding i outputs and no later one; and each a record of a
        // run of this workflow - which one claiming the fresh source that
        // stands in the caller's place during a run would not be, every
        // identifier it holds being past it.
        assert_eq!(records.len(), expected, "{step}");
        for (outputs, record) in records.iter().enumerate() {
            assert_eq!(outputs_in(record), outputs, "{record}");
            let resumed = Run::<ScriptFailure>::resume(&definition, record);
            assert!(resumed.is_ok(), "{:?}\n{record}", resumed.err());
        }
    }
}

#[cfg(test)]
#[test]
fn interrupted_run_resumes() {
    // The measured shape (EVD_INTERRUPTED_RUN_REPEATS_CALLS): three nodes each
    // calling a model, the provider answering two calls and holding the third.
    let calling = "local given, host = ...\nlocal answer = host.complete('drafting', given.input)\nreturn host.compose(host.output, {given.input, answer}, ' ')";
    let behaviours = Behaviours::new()
        .define("seed", "seed.lua", calling)
        .define("step", "step.lua", calling);
    let definition = workflow(CHAIN_TYPES, CHAIN);
    let roster = |stub: &crate::models::Stub| {
        Roster::new(crate::models::client_for(&stub.base)).map("drafting", "openai::m")
    };

    let holding = crate::models::Stub::holding_after(2, "an answer");
    let first = roster(&holding);
    let mut source = IdSource::new();
    let argument = note(&mut source, "note", "hello");
    let mut records = Vec::new();
    let interrupted = block(async {
        tokio::select! {
            ending = run_scripted(
                &definition,
                &behaviours,
                &first,
                Arguments::new().supply("first", "input", argument),
                &mut source,
                20,
                SMALL,
                |record| records.push(record),
            ) => Some(ending),
            () = async {
                while holding.requests().len() < 3 {
                    tokio::task::yield_now().await;
                }
            } => None,
        }
    });
    assert!(
        interrupted.is_none(),
        "the run was dropped during its third call"
    );
    let last = records.last().expect("records were handed over");
    assert_eq!(outputs_in(last), 2, "two outputs recorded:\n{last}");

    // Resumed from the last record against a provider that answers: one call,
    // for the activation that was interrupted.
    let answering = crate::models::Stub::answering(200, "an answer");
    let (ending, _) = block(resume_scripted(
        &definition,
        &behaviours,
        &roster(&answering),
        last,
        SMALL,
        |_| {},
    ))
    .expect("resumes");
    assert_eq!(answering.requests().len(), 1);
    let Outcome::Ended(RunEnding::Completed(resumed)) = ending else {
        panic!("expected completion, got {ending:?}")
    };

    // The result an uninterrupted run gives, identifiers included: the first
    // half's contexts are the ones it recorded.
    let whole = crate::models::Stub::answering(200, "an answer");
    let mut source = IdSource::new();
    let argument = note(&mut source, "note", "hello");
    let ending = block(run_scripted(
        &definition,
        &behaviours,
        &roster(&whole),
        Arguments::new().supply("first", "input", argument),
        &mut source,
        20,
        SMALL,
        |_| {},
    ));
    let Ok(Outcome::Ended(RunEnding::Completed(uninterrupted))) = ending else {
        panic!("expected completion, got {ending:?}")
    };
    assert_eq!(whole.requests().len(), 3);
    assert_eq!(resumed.render(), uninterrupted.render());
    assert_eq!(resumed.id(), uninterrupted.id());
    let ids = |context: &agconflo_core::Context| {
        let mut ids: Vec<String> = context
            .lineage()
            .iter()
            .map(|c| format!("{:?}", c.id()))
            .collect();
        ids.sort();
        ids
    };
    assert_eq!(ids(&resumed), ids(&uninterrupted));
}

/// Node types for a reviewed draft: `seed` drafts from the run's argument,
/// `review` gives a verdict on the draft, and `close` takes both.
#[cfg(test)]
const REVIEWED_TYPES: &str = "\
[types.seed]
required = { input = \"note\" }
output = \"note\"

[types.review]
required = { input = \"note\" }
output = \"verdict\"

[types.close]
required = { draft = \"note\", verdict = \"verdict\" }
output = \"note\"
";

/// `first`, an entry `seed`; `second`, a `review` of it; `third`, a `close`
/// bound to both, and designated.
#[cfg(test)]
const REVIEWED: &str = "\
name = \"reviewed\"
output = \"third\"

[instances.first]
node_type = \"seed\"
entry = true

[instances.second]
node_type = \"review\"
bindings = { input = \"first\" }

[instances.third]
node_type = \"close\"
bindings = { draft = \"first\", verdict = \"second\" }
";

/// A script composing a draft and its verdict.
#[cfg(test)]
const CLOSING: &str = "local given, host = ...\nreturn host.compose(host.output, {given.draft, given.verdict}, ' / ')";

/// A script that fails if it runs at all.
#[cfg(test)]
const RAN: &str = "error('ran')";

/// Behaviours for [`REVIEWED`]: `review` performed by a person, `close` by
/// `closing`.
#[cfg(test)]
fn reviewed(closing: &str) -> Behaviours {
    Behaviours::new()
        .define("seed", "seed.lua", &appending("seeded"))
        .person("review")
        .define("close", "close.lua", closing)
}

/// [`REVIEWED`] run from `hello` with `behaviours` and `budget` until it stops,
/// with every record it handed over.
#[cfg(test)]
fn reviewed_until_it_stops(
    behaviours: &Behaviours,
    budget: usize,
) -> (Result<Outcome, ScriptedRefusal>, Vec<String>) {
    let definition = workflow(REVIEWED_TYPES, REVIEWED);
    let mut source = IdSource::new();
    let argument = note(&mut source, "note", "hello");
    let mut records = Vec::new();
    let outcome = block(run_scripted(
        &definition,
        behaviours,
        &offline(),
        Arguments::new().supply("first", "input", argument),
        &mut source,
        budget,
        SMALL,
        |record| records.push(record),
    ));
    (outcome, records)
}

/// How many outputs a record's text holds: its events that are outputs, read
/// from the text rather than from a resumed run.
#[cfg(test)]
fn outputs_in(record: &str) -> usize {
    record
        .lines()
        .filter(|line| line.starts_with("output = "))
        .count()
}

/// The identifiers of an activation's inputs, by parameter.
#[cfg(test)]
fn input_ids(activation: &Activation) -> Vec<(String, agconflo_core::ContextId)> {
    activation
        .inputs()
        .iter()
        .map(|(parameter, context)| (parameter.clone(), context.id()))
        .collect()
}

#[cfg(test)]
#[test]
fn person_step_handed_over() {
    // `close` fails if it runs, so a host running past the person's step ends
    // the run on its failure.
    let (outcome, records) = reviewed_until_it_stops(&reviewed(RAN), 3);
    let Ok(Outcome::Awaiting(activation)) = outcome else {
        panic!("expected the person's step handed over, got {outcome:?}")
    };
    assert_eq!(activation.instance(), "second");
    assert_eq!(activation.output().as_str(), "verdict");
    let [(parameter, input)] = activation.inputs() else {
        panic!("one input, got {:?}", activation.inputs())
    };
    assert_eq!(
        (parameter.as_str(), &*input.render()),
        ("input", "hello seeded")
    );

    // At the start and after `first`'s output, and none as the step was handed
    // over. The input is the context the last record holds, by identifier:
    // the step a run resumed from it offers carries the same one.
    assert_eq!(records.len(), 2);
    let definition = workflow(REVIEWED_TYPES, REVIEWED);
    let (mut resumed, _) =
        Run::<ScriptFailure>::resume(&definition, &records[1]).expect("the record resumes");
    let Step::Activate(offered) = resumed.step() else {
        panic!("the resumed run offers the person's step")
    };
    assert_eq!(input_ids(&offered), input_ids(&activation));
}

#[cfg(test)]
proptest::proptest! {
    /// For any text, the person's answer is kept exactly, as a context of the
    /// type the step declares, and the run completes on a budget exactly its
    /// length.
    #[test]
    fn person_text_becomes_output(
        text in proptest::prop_oneof![
            proptest::strategy::Just(String::new()),
            proptest::strategy::Just(" \r\n looks right \r \n\t".to_owned()),
            proptest::arbitrary::any::<String>(),
        ],
    ) {
        let (outcome, records) = reviewed_until_it_stops(&reviewed(CLOSING), 3);
        proptest::prop_assert!(matches!(outcome, Ok(Outcome::Awaiting(_))), "{:?}", outcome);
        let definition = workflow(REVIEWED_TYPES, REVIEWED);

        let mut after = Vec::new();
        let answered = block(answer_scripted(
            &definition,
            &reviewed(CLOSING),
            &offline(),
            records.last().expect("records were handed over"),
            "second",
            &text,
            SMALL,
            |record| after.push(record),
        ));
        let Ok((Outcome::Ended(RunEnding::Completed(result)), _)) = answered else {
            let message = format!("expected completion, got {answered:?}");
            return Err(proptest::test_runner::TestCaseError::fail(message));
        };

        // The result is the draft and the verdict; the verdict is the text.
        let [draft, verdict] = result.parts() else {
            return Err(proptest::test_runner::TestCaseError::fail("two parts"));
        };
        proptest::prop_assert_eq!(&*draft.render(), "hello seeded");
        proptest::prop_assert_eq!(verdict.declared_type().as_str(), "verdict");
        proptest::prop_assert_eq!(&*verdict.render(), text.as_str());

        // After the answer and after `third`'s output; the first holds the
        // answer and resumes.
        proptest::prop_assert_eq!(after.len(), 2);
        proptest::prop_assert_eq!(outputs_in(&after[0]), 2);
        let resumed = Run::<ScriptFailure>::resume(&definition, &after[0]);
        proptest::prop_assert!(resumed.is_ok(), "{:?}", resumed.err());
    }
}

#[cfg(test)]
#[test]
fn exhausted_source_fails_the_step() {
    let (_, records) = reviewed_until_it_stops(&reviewed(CLOSING), 3);
    let parked = records.last().expect("records were handed over");
    let position = parked
        .lines()
        .find(|line| line.starts_with("source = "))
        .expect("a record carries its source");
    let exhausted = parked.replace(position, "source = \"exhausted\"");

    // The record is sound - an exhausted source has issued every identifier it
    // holds - and the person's answer cannot be issued under anything.
    let definition = workflow(REVIEWED_TYPES, REVIEWED);
    let answered = block(answer_scripted(
        &definition,
        &reviewed(CLOSING),
        &offline(),
        &exhausted,
        "second",
        "looks right",
        SMALL,
        |_| {},
    ));
    let Ok((Outcome::Ended(RunEnding::NodeFailed { instance, failure }), _)) = answered else {
        panic!("expected the step to fail, got {answered:?}")
    };
    assert_eq!(instance, "second");
    assert_eq!(
        failure,
        ScriptFailure::SourceExhausted(agconflo_core::SourceExhausted)
    );
}

#[cfg(test)]
#[test]
fn answer_elsewhere_refused() {
    let definition = workflow(REVIEWED_TYPES, REVIEWED);
    let (_, records) = reviewed_until_it_stops(&reviewed(CLOSING), 3);
    let (started, parked) = (&records[0], &records[1]);
    let mut completed = Vec::new();
    block(answer_scripted(
        &definition,
        &reviewed(CLOSING),
        &offline(),
        parked,
        "second",
        "looks right",
        SMALL,
        |record| completed.push(record),
    ))
    .expect("the answer is taken");
    let completed = completed.last().expect("records were handed over");

    // Every script fails if it runs, so a host that ran one would end the run
    // on that failure rather than refuse.
    let failing = Behaviours::new()
        .define("seed", "seed.lua", RAN)
        .person("review")
        .define("close", "close.lua", RAN);
    let refused = |record: &str, instance: &str, offered: Option<&str>| {
        let mut handed = 0;
        let answered = block(answer_scripted(
            &definition,
            &failing,
            &offline(),
            record,
            instance,
            "looks right",
            SMALL,
            |_| handed += 1,
        ));
        let expected = ScriptedRefusal::NotAwaited {
            answered: instance.to_owned(),
            offered: offered.map(str::to_owned),
        };
        assert_eq!(answered.map(|_| ()), Err(expected), "{instance}");
        assert_eq!(handed, 0, "no record is handed over");
    };

    // The run offers a script's step first: answered for the person's
    // instance, and for the script's own.
    refused(started, "second", Some("first"));
    refused(started, "first", Some("first"));
    // It awaits the person, answered for another instance and for none it has.
    refused(parked, "third", Some("second"));
    refused(parked, "nobody", Some("second"));
    // It has ended.
    refused(completed, "second", None);

    // What a resume refuses, and what the scripts are refused for, come first.
    let answered = block(answer_scripted(
        &definition,
        &failing,
        &offline(),
        "not a record",
        "second",
        "looks right",
        SMALL,
        |_| {},
    ));
    assert!(
        matches!(answered, Err(ScriptedRefusal::Resume(_))),
        "{answered:?}"
    );
    // The scripts' faults, whether or not the answer is one the run awaits.
    let faulty = failing.clone().define("review", "review.lua", RAN);
    for instance in ["second", "third"] {
        let answered = block(answer_scripted(
            &definition,
            &faulty,
            &offline(),
            parked,
            instance,
            "looks right",
            SMALL,
            |_| {},
        ));
        assert!(
            matches!(answered, Err(ScriptedRefusal::Behaviours(_))),
            "{instance}: {answered:?}"
        );
    }
}

#[cfg(test)]
#[test]
fn awaiting_is_not_quiescent() {
    // The person's step is the designated one, and nothing else is left.
    let flow = REVIEWED.replace("output = \"third\"", "output = \"second\"");
    let definition = workflow(REVIEWED_TYPES, &flow);
    let mut source = IdSource::new();
    let argument = note(&mut source, "note", "hello");
    let mut records = Vec::new();
    let outcome = block(run_scripted(
        &definition,
        &reviewed(RAN),
        &offline(),
        Arguments::new().supply("first", "input", argument),
        &mut source,
        3,
        SMALL,
        |record| records.push(record),
    ));
    let Ok(Outcome::Awaiting(activation)) = outcome else {
        panic!("expected the person's step handed over, got {outcome:?}")
    };
    assert_eq!(activation.instance(), "second");

    // Resumed from its record, as after a restart: the same step again.
    let (again, _) = block(resume_scripted(
        &definition,
        &reviewed(RAN),
        &offline(),
        records.last().expect("records were handed over"),
        SMALL,
        |_| {},
    ))
    .expect("resumes");
    let Outcome::Awaiting(again) = again else {
        panic!("expected the person's step handed over again, got {again:?}")
    };
    assert_eq!(again.instance(), "second");
    assert_eq!(input_ids(&again), input_ids(&activation));

    // The control: a person's step no run can reach, in a cycle written in
    // bindings. That run is stuck, and says so.
    let types = "[types.ask]\nrequired = { input = \"note\" }\noutput = \"note\"\n";
    let cycle = "\
name = \"cycle\"
output = \"y\"

[instances.x]
node_type = \"ask\"
bindings = { input = \"y\" }

[instances.y]
node_type = \"ask\"
bindings = { input = \"x\" }
";
    let behaviours = Behaviours::new().person("ask");
    match run_with(&workflow(types, cycle), &behaviours, None) {
        Ok(Outcome::Ended(RunEnding::Quiescent { waiting })) => assert_eq!(waiting, ["x", "y"]),
        other => panic!("a cycle can do nothing, so the run is quiescent, not {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn person_between_model_calls() {
    let types = "\
[types.ask]
required = { input = \"note\" }
output = \"note\"

[types.review]
required = { input = \"note\" }
output = \"note\"

[types.close]
required = { draft = \"note\", verdict = \"note\" }
output = \"note\"
";
    let asking = "local given, host = ...\nlocal answer = host.complete('drafting', given.input)\nreturn host.compose(host.output, {given.input, answer}, ' ')";
    let closing = "local given, host = ...\nlocal answer = host.complete('drafting', given.verdict)\nreturn host.compose(host.output, {given.draft, given.verdict, answer}, ' / ')";
    let behaviours = Behaviours::new()
        .define("ask", "ask.lua", asking)
        .person("review")
        .define("close", "close.lua", closing);
    let flow = REVIEWED.replace("\"seed\"", "\"ask\"");
    let definition = workflow(types, &flow);
    let roster = |stub: &crate::models::Stub| {
        Roster::new(crate::models::client_for(&stub.base)).map("drafting", "openai::m")
    };

    let first = crate::models::Stub::answering(200, "an answer");
    let before = roster(&first);
    let mut source = IdSource::new();
    let argument = note(&mut source, "note", "hello");
    let mut records = Vec::new();
    let outcome = block(run_scripted(
        &definition,
        &behaviours,
        &before,
        Arguments::new().supply("first", "input", argument),
        &mut source,
        3,
        SMALL,
        |record| records.push(record),
    ));
    assert!(matches!(outcome, Ok(Outcome::Awaiting(_))), "{outcome:?}");
    assert_eq!(first.requests().len(), 1);
    let parked = records.last().expect("records were handed over");

    // The first half's output, by identifier, as the parked record holds it.
    let (mut resumed, _) =
        Run::<ScriptFailure>::resume(&definition, parked).expect("the record resumes");
    let Step::Activate(offered) = resumed.step() else {
        panic!("the resumed run offers the person's step")
    };
    let drafted = offered.inputs()[0].1.id();

    // Answered from the record, in a run built from its text alone.
    let second = crate::models::Stub::answering(200, "a second answer");
    let after = roster(&second);
    let (outcome, _) = block(answer_scripted(
        &definition,
        &behaviours,
        &after,
        parked,
        "second",
        "looks right",
        SMALL,
        |_| {},
    ))
    .expect("the answer is taken");
    assert_eq!(second.requests().len(), 1, "one call, for the third step");
    let Outcome::Ended(RunEnding::Completed(result)) = outcome else {
        panic!("expected completion, got {outcome:?}")
    };
    assert_eq!(
        result.render(),
        "hello an answer / looks right / a second answer"
    );
    assert!(
        result
            .lineage()
            .iter()
            .any(|context| context.id() == drafted),
        "the result holds the first half's output under its recorded identifier"
    );
}

// --- a model yielding ------------------------------------------------------------

#[cfg(test)]
use crate::models::{Reply, Stub, client_for, sent_messages, sent_tools};

/// Node types for a model yielding: `ask`, which asks; `lookup`, described, one
/// required parameter and one optional; and `search`, which nothing declares a
/// call to.
#[cfg(test)]
pub(crate) const YIELD_TYPES: &str = "\
[types.ask]
output = \"note\"

[types.lookup]
description = \"Looks a codeword up.\"
required = { query = \"note\" }
optional = { hint = \"note\" }
output = \"note\"

[types.search]
output = \"note\"
";

/// One instance, `asker`, designated, declaring its calls to `lookup` twice.
#[cfg(test)]
pub(crate) const YIELDING: &str = "\
name = \"yielding\"
output = \"asker\"

[instances.asker]
node_type = \"ask\"
calls = [\"lookup\", \"lookup\"]
";

/// The asker's script: sets a global of its own, asks the model `question`, and
/// outputs a composition of the answer.
#[cfg(test)]
fn asking(question: &str) -> String {
    format!(
        "local given, host = ...\nleaked = 'the caller'\nlocal answer = host.complete('asking', host.text('note', '{question}'))\nreturn host.compose(host.output, {{answer}}, '')"
    )
}

/// lookup's script: what it was asked, and what its state holds under the
/// caller's global name.
#[cfg(test)]
const LOOKUP: &str = "local given, host = ...\nreturn host.text(host.output, 'means ' .. given.query:render() .. ' / ' .. tostring(leaked))";

/// The asker and lookup, performed by their scripts.
#[cfg(test)]
fn yielding_behaviours(question: &str, lookup: &str) -> Behaviours {
    Behaviours::new()
        .define("ask", "ask.lua", &asking(question))
        .define("lookup", "lookup.lua", lookup)
}

/// Room for a few calls.
#[cfg(test)]
const CALLING: Limits = Limits {
    model_calls: 8,
    ..SMALL
};

/// A roster sending the role `asking` to `stub`, in `model`'s format.
#[cfg(test)]
fn roster_at(stub: &Stub, model: &str) -> Roster {
    Roster::new(client_for(&stub.base)).map("asking", model)
}

/// Run `definition` against `stub` with nothing supplied, keeping every record.
#[cfg(test)]
fn run_keeping(
    definition: &WorkflowDefinition,
    behaviours: &Behaviours,
    roster: &Roster,
    limits: Limits,
) -> (Result<Outcome, ScriptedRefusal>, Vec<String>) {
    let mut records = Vec::new();
    let mut source = IdSource::new();
    let outcome = block(run_scripted(
        definition,
        behaviours,
        roster,
        Arguments::new(),
        &mut source,
        20,
        limits,
        |record| records.push(record),
    ));
    (outcome, records)
}

/// A reply calling `lookup` about `query`, under `id`.
#[cfg(test)]
fn looking_up(id: &str, query: &str) -> Reply {
    Reply::text("").call(id, "lookup", &format!("{{\"query\": \"{query}\"}}"))
}

/// A record's text as TOML.
#[cfg(test)]
fn record_toml(record: &str) -> toml_edit::DocumentMut {
    record.parse().expect("a record is TOML")
}

/// A record's events, and the kind of each: `exchange`, `call` or `output`.
#[cfg(test)]
fn record_events(record: &str) -> Vec<(String, toml_edit::Table)> {
    let parsed = record_toml(record);
    let Some(events) = parsed
        .get("event")
        .and_then(toml_edit::Item::as_array_of_tables)
    else {
        return Vec::new();
    };
    events
        .iter()
        .map(|event| {
            let kind = ["exchange", "call", "output"]
                .into_iter()
                .find(|kind| event.contains_key(kind))
                .expect("one kind each");
            (kind.to_owned(), event.clone())
        })
        .collect()
}

/// The kinds of a record's events, in order.
#[cfg(test)]
fn kinds(record: &str) -> Vec<String> {
    record_events(record)
        .into_iter()
        .map(|(kind, _)| kind)
        .collect()
}

/// The text a record's text context `id` holds.
#[cfg(test)]
fn text_of(record: &str, id: &str) -> String {
    record_toml(record)["context"][id]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("context {id} holds text"))
        .to_owned()
}

#[cfg(test)]
#[test]
fn model_yields() {
    let definition = workflow(YIELD_TYPES, YIELDING);
    for model in ["openai::m", "anthropic::m"] {
        let stub = Stub::replying(vec![
            looking_up("call_7", "amber-7"),
            Reply::text("amber-7: the harbour is closed"),
        ]);
        let (outcome, records) = run_keeping(
            &definition,
            &yielding_behaviours("What is amber-7?", LOOKUP),
            &roster_at(&stub, model),
            CALLING,
        );
        assert_eq!(
            rendered(outcome),
            "amber-7: the harbour is closed",
            "{model}"
        );

        let requests = stub.requests();
        assert_eq!(requests.len(), 2, "{model}");
        let continued = sent_messages(&requests[1].1);
        assert_eq!(
            continued.last().expect("messages").results,
            [("call_7".to_owned(), "means amber-7 / nil".to_owned())],
            "{model}: the model is asked again with the call's output as its result"
        );
        assert_eq!(
            kinds(records.last().expect("records")),
            ["exchange", "call", "output", "exchange", "output"],
            "{model}"
        );
    }
}

#[cfg(test)]
#[test]
fn offer_is_the_declared_calls() {
    // lookup asks a model itself, so its own request can be looked at.
    let asking_lookup = "local given, host = ...\nlocal found = host.complete('asking', given.query)\nreturn host.text(host.output, 'means ' .. found:render())";
    let stub = Stub::replying(vec![
        looking_up("call_7", "amber-7"),
        Reply::text("the harbour"),
        Reply::text("done"),
    ]);
    let (outcome, records) = run_keeping(
        &workflow(YIELD_TYPES, YIELDING),
        &yielding_behaviours("What is amber-7?", asking_lookup),
        &roster_at(&stub, "openai::m"),
        CALLING,
    );
    assert_eq!(rendered(outcome), "done");

    let requests = stub.requests();
    assert_eq!(requests.len(), 3);
    let lookup_offered = Some(vec![(
        "lookup".to_owned(),
        Some("Looks a codeword up.".to_owned()),
        vec![
            ("query".to_owned(), "string".to_owned()),
            ("hint".to_owned(), "string".to_owned()),
        ],
        vec!["query".to_owned()],
    )]);
    // Declared twice, offered once; `search` is in the catalogue and not
    // declared, so not offered.
    assert_eq!(sent_tools(&requests[0].1), lookup_offered);
    assert_eq!(
        sent_tools(&requests[1].1),
        None,
        "lookup's own call offers nothing"
    );
    assert_eq!(sent_tools(&requests[2].1), lookup_offered);

    // Held by the run as contexts: the exchange's one offered node type is a
    // composition of its name, description and parameters' names.
    let last = records.last().expect("records");
    let (_, first) = &record_events(last)[0];
    let offer = first["exchange"]["offer"].as_array().expect("an offer");
    assert_eq!(offer.len(), 1);
    let offered = offer
        .get(0)
        .and_then(|id| id.as_str())
        .expect("an identifier");
    let parts: Vec<String> = record_toml(last)["context"][offered]["parts"]
        .as_array()
        .expect("a composition")
        .iter()
        .map(|part| text_of(last, part.as_str().expect("an identifier")))
        .collect();
    assert_eq!(parts, ["lookup", "Looks a codeword up.", "query", "hint"]);

    // A node declaring no calls offers nothing.
    let plain = Stub::replying(vec![Reply::text("plain")]);
    let (outcome, _) = run_keeping(
        &workflow(
            YIELD_TYPES,
            "name = \"plain\"\noutput = \"asker\"\n\n[instances.asker]\nnode_type = \"ask\"\n",
        ),
        &yielding_behaviours("q", LOOKUP),
        &roster_at(&plain, "openai::m"),
        CALLING,
    );
    assert_eq!(rendered(outcome), "plain");
    assert_eq!(sent_tools(&plain.requests()[0].1), None);
}

#[cfg(test)]
#[test]
fn calls_performed_in_order() {
    let stub = Stub::replying(vec![
        Reply::text("")
            .call("c1", "lookup", "{\"query\": \"first\"}")
            .call("c2", "lookup", "{\"query\": \"second\"}"),
        Reply::text("done"),
    ]);
    let (outcome, _) = run_keeping(
        &workflow(YIELD_TYPES, YIELDING),
        &yielding_behaviours("q", LOOKUP),
        &roster_at(&stub, "anthropic::m"),
        CALLING,
    );
    assert_eq!(rendered(outcome), "done");
    // Asked again once, after both, with both results in the answer's order;
    // and each lookup's state held nothing of the asker's global.
    let requests = stub.requests();
    assert_eq!(requests.len(), 2);
    let results: Vec<(String, String)> = sent_messages(&requests[1].1)
        .into_iter()
        .flat_map(|message| message.results)
        .collect();
    assert_eq!(
        results,
        [
            ("c1".to_owned(), "means first / nil".to_owned()),
            ("c2".to_owned(), "means second / nil".to_owned()),
        ]
    );
}

#[cfg(test)]
#[test]
fn next_window_composes_the_last() {
    let stub = Stub::replying(vec![looking_up("call_7", "amber-7"), Reply::text("done")]);
    let (outcome, records) = run_keeping(
        &workflow(YIELD_TYPES, YIELDING),
        &yielding_behaviours("What is amber-7?", LOOKUP),
        &roster_at(&stub, "openai::m"),
        CALLING,
    );
    assert_eq!(rendered(outcome), "done");
    let last = records.last().expect("records");
    let events = record_events(last);
    let id = |item: &toml_edit::Item| item.as_str().expect("an identifier").to_owned();
    let first = &events[0].1["exchange"];
    let call = &events[1].1["call"];
    let output = &events[2].1;
    let second = &events[3].1["exchange"];

    let window = id(&second["window"]);
    let written = &record_toml(last)["context"][window.as_str()];
    let parts: Vec<String> = written["parts"]
        .as_array()
        .expect("a composition")
        .iter()
        .map(|part| part.as_str().expect("an identifier").to_owned())
        .collect();
    assert_eq!(
        parts,
        [
            id(&first["window"]),
            id(&first["answer"]),
            id(&call["inputs"]["query"]),
            id(&output["output"]),
        ],
        "the last window, the answer, the call's contexts and its output, by reference"
    );
    assert_eq!(written["separator"].as_str(), Some(""));
    assert_eq!(written["type"].as_str(), Some("note"), "the prompt's type");

    // Its rendering is the text the request carried, in order.
    let rendering: String = parts.iter().map(|part| text_of(last, part)).collect();
    let mut carried = String::new();
    for message in sent_messages(&stub.requests()[1].1) {
        carried.extend(message.texts);
        for (_, _, arguments) in message.calls {
            carried.push_str(arguments["query"].as_str().unwrap_or(""));
        }
        carried.extend(message.results.into_iter().map(|(_, text)| text));
    }
    assert_eq!(carried, rendering);
}

#[cfg(test)]
#[test]
fn malformed_call_fails() {
    use crate::host::ModelCallFault;
    let never = "error('lookup must not run')";
    let cases = [
        ("search", "{\"query\": \"x\"}", ModelCallFault::NotOffered),
        ("lookup", "\"amber-7\"", ModelCallFault::NotAnObject),
        (
            "lookup",
            "{\"query\": \"x\", \"extra\": \"y\"}",
            ModelCallFault::UndeclaredParameter {
                parameter: "extra".to_owned(),
            },
        ),
        (
            "lookup",
            "{\"hint\": \"x\"}",
            ModelCallFault::RequiredMissing {
                parameter: "query".to_owned(),
            },
        ),
        (
            "lookup",
            "{\"query\": 7}",
            ModelCallFault::NotAString {
                parameter: "query".to_owned(),
            },
        ),
    ];
    for (name, arguments, fault) in cases {
        let stub = Stub::replying(vec![
            looking_up("call_1", "fine").call("call_2", name, arguments),
            Reply::text("never asked"),
        ]);
        let (outcome, records) = run_keeping(
            &workflow(YIELD_TYPES, YIELDING),
            &yielding_behaviours("q", never),
            &roster_at(&stub, "openai::m"),
            CALLING,
        );
        assert_eq!(
            failed(outcome),
            (
                "asker".to_owned(),
                ScriptFailure::MalformedCall {
                    node_type: name.to_owned(),
                    fault: fault.clone(),
                }
            ),
            "{arguments}"
        );
        // Nothing reported, performed or spent for the well-formed call before
        // it - no record was handed over after the one at the start, since
        // nothing happened to hold - and the model not asked again.
        assert_eq!(stub.requests().len(), 1, "{arguments}");
        assert_eq!(records.len(), 1, "{arguments}: {records:?}");
        assert!(kinds(&records[0]).is_empty());
    }
}

#[cfg(test)]
#[test]
fn refused_call_fails_with_the_refusal() {
    // The host checks every call before the run sees it, so a call the run
    // refuses is made on purpose: the host's offer comes from a workflow that
    // declares the call, and the run's from one that does not.
    let declaring = workflow(YIELD_TYPES, YIELDING);
    let undeclaring = workflow(
        YIELD_TYPES,
        "name = \"yielding\"\noutput = \"asker\"\n\n[instances.asker]\nnode_type = \"ask\"\n",
    );
    let mut run = Run::<ScriptFailure>::start(&undeclaring, Arguments::new(), 20).expect("starts");
    let Step::Activate(activation) = run.step() else {
        panic!("the asker is offered")
    };
    let stub = Stub::replying(vec![looking_up("call_1", "amber-7")]);
    let behaviours = yielding_behaviours("q", LOOKUP);
    let roster = roster_at(&stub, "openai::m");
    let mut keep = |_: String| {};
    let mut env = Env {
        definition: &declaring,
        behaviours: &behaviours,
        roster: &roster,
        limits: CALLING,
        source: Rc::new(RefCell::new(IdSource::new())),
        keep: &mut keep,
    };
    match block(perform_activation(&mut run, &activation, &mut env)) {
        Performed::Stopped(Stop::Failed(ScriptFailure::CallRefused(refusal))) => assert_eq!(
            refusal,
            agconflo_core::CallRefusal::Undeclared {
                instance: "asker".to_owned(),
                node_type: "lookup".to_owned(),
            }
        ),
        Performed::Output(output) => panic!("expected the call refused, got {output:?}"),
        Performed::Stopped(_) => panic!("expected the call refused, and it stopped otherwise"),
    }
    assert_eq!(stub.requests().len(), 1, "not asked again");
}

#[cfg(test)]
#[test]
fn exchanges_reported_before_calls() {
    let asking_lookup = "local given, host = ...\nlocal found = host.complete('asking', given.query)\nreturn host.text(host.output, 'means ' .. found:render())";
    let stub = Stub::replying(vec![
        looking_up("call_7", "amber-7"),
        Reply::text("the harbour"),
        Reply::text("done"),
    ]);
    let (outcome, records) = run_keeping(
        &workflow(YIELD_TYPES, YIELDING),
        &yielding_behaviours("q", asking_lookup),
        &roster_at(&stub, "openai::m"),
        CALLING,
    );
    assert_eq!(rendered(outcome), "done");
    // The record handed over when lookup starts - the one after the answer that
    // called it - already holds that exchange, with its call.
    let before = &records[1];
    assert_eq!(kinds(before), ["exchange"]);
    let (_, exchange) = &record_events(before)[0];
    let calls = exchange["exchange"]["calls"].as_array().expect("calls");
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls
            .get(0)
            .and_then(|call| call.as_inline_table())
            .and_then(|call| call.get("id"))
            .and_then(|id| id.as_str()),
        Some("call_7")
    );
    // lookup's own answer, which called nothing, and the continuation are held
    // too - lookup's under the call it performs.
    let last = records.last().expect("records");
    assert_eq!(
        kinds(last),
        [
            "exchange", "call", "exchange", "output", "exchange", "output"
        ]
    );
    let (_, own) = &record_events(last)[2];
    assert_eq!(own["performing"].as_str(), Some("call_7"));
}

#[cfg(test)]
#[test]
fn record_after_each_answer() {
    let stub = Stub::replying(vec![looking_up("call_7", "amber-7"), Reply::text("done")]);
    let (outcome, records) = run_keeping(
        &workflow(YIELD_TYPES, YIELDING),
        &yielding_behaviours("q", LOOKUP),
        &roster_at(&stub, "openai::m"),
        CALLING,
    );
    assert_eq!(rendered(outcome), "done");
    let counts: Vec<Vec<String>> = records.iter().map(|record| kinds(record)).collect();
    assert_eq!(
        counts,
        [
            vec![],
            vec!["exchange".to_owned()],
            vec![
                "exchange".to_owned(),
                "call".to_owned(),
                "output".to_owned()
            ],
            vec![
                "exchange".to_owned(),
                "call".to_owned(),
                "output".to_owned(),
                "exchange".to_owned()
            ],
            vec![
                "exchange".to_owned(),
                "call".to_owned(),
                "output".to_owned(),
                "exchange".to_owned(),
                "output".to_owned()
            ],
        ],
        "at the start, after each answer and after each output"
    );
    // Each record handed over after an answer holds that answer.
    let answer = |record: &str| {
        let (_, exchange) = record_events(record)
            .into_iter()
            .rev()
            .find(|(kind, _)| kind == "exchange")
            .expect("an exchange");
        let id = exchange["exchange"]["answer"]
            .as_str()
            .expect("an identifier")
            .to_owned();
        text_of(record, &id)
    };
    assert_eq!(answer(&records[1]), "");
    assert_eq!(answer(&records[3]), "done");
}

#[cfg(test)]
proptest::proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(12))]

    /// For a run whose model yields in `rounds` answers of `calls` calls each,
    /// resumed from every record it hands over against a provider holding only
    /// the replies the record had not had, the resumed run ends with the same
    /// output having asked the provider for exactly those: no window the record
    /// answered is sent again. When the called node type asks a model of its
    /// own, some of those records are taken while it is being performed, its
    /// answer held and the caller's continuation not yet sent. And resumed with
    /// a model call limit equal to the requests the record answered, the next
    /// new request fails at the limit - asked of the caller alone, since each
    /// activation has a limit of its own.
    #[test]
    fn resumed_mid_call_asks_nothing_twice(
        rounds in 1..=2usize,
        calls in 1..=2usize,
        anthropic in proptest::prelude::any::<bool>(),
        callee_asks in proptest::prelude::any::<bool>(),
    ) {
        let model = if anthropic { "anthropic::m" } else { "openai::m" };
        let definition = workflow(YIELD_TYPES, YIELDING);
        let asking_lookup = "local given, host = ...\nlocal found = host.complete('asking', given.query)\nreturn host.text(host.output, 'means ' .. found:render())";
        let behaviours = yielding_behaviours("q", if callee_asks { asking_lookup } else { LOOKUP });
        let mut replies = Vec::new();
        for round in 0..rounds {
            let mut reply = Reply::text("");
            for call in 0..calls {
                let id = format!("c{round}{call}");
                reply = reply.call(&id, "lookup", &format!("{{\"query\": \"q{round}{call}\"}}"));
            }
            replies.push(reply);
            if callee_asks {
                for call in 0..calls {
                    replies.push(Reply::text(&format!("found {round}{call}")));
                }
            }
        }
        replies.push(Reply::text("done"));

        let stub = Stub::replying(replies.clone());
        let (outcome, records) = run_keeping(&definition, &behaviours, &roster_at(&stub, model), CALLING);
        let result = rendered(outcome);

        let finished = kinds(records.last().expect("records"));
        for record in &records {
            let answered = kinds(record).iter().filter(|kind| *kind == "exchange").count();
            let remaining = Stub::replying(replies[answered..].to_vec());
            let mut again = vec![record.clone()];
            let resumed = block(resume_scripted(
                &definition, &behaviours, &roster_at(&remaining, model), record, CALLING,
                |record| again.push(record),
            ))
            .map(|(outcome, _)| outcome);
            proptest::prop_assert_eq!(rendered(resumed), result.clone());
            proptest::prop_assert_eq!(remaining.requests().len(), replies.len() - answered, "{}", record);
            // No recorded call performed again: the resumed run ends having
            // held what the uninterrupted one held, and no more.
            proptest::prop_assert_eq!(&kinds(again.last().expect("records")), &finished);

            if !callee_asks && answered < replies.len() {
                let limit = Limits { model_calls: u32::try_from(answered).expect("small"), ..CALLING };
                let unasked = Stub::replying(Vec::new());
                let limited = block(resume_scripted(
                    &definition, &behaviours, &roster_at(&unasked, model), record, limit, |_| {},
                ))
                .map(|(outcome, _)| outcome);
                proptest::prop_assert_eq!(failed(limited).1, ScriptFailure::ModelCallLimit);
                proptest::prop_assert_eq!(unasked.requests().len(), 0);
            }
        }
    }
}

#[cfg(test)]
#[test]
fn replay_that_diverges_fails() {
    let definition = workflow(YIELD_TYPES, YIELDING);
    let stub = Stub::replying(vec![looking_up("call_7", "amber-7"), Reply::text("done")]);
    let (outcome, records) = run_keeping(
        &definition,
        &yielding_behaviours("What is amber-7?", LOOKUP),
        &roster_at(&stub, "openai::m"),
        CALLING,
    );
    assert_eq!(rendered(outcome), "done");
    // Taken after the first answer.
    let record = &records[1];
    assert_eq!(kinds(record), ["exchange"]);
    let resumed = |definition: &WorkflowDefinition, question: &str| {
        let unasked = Stub::replying(vec![Reply::text("done")]);
        let outcome = block(resume_scripted(
            definition,
            &yielding_behaviours(question, LOOKUP),
            &roster_at(&unasked, "openai::m"),
            record,
            CALLING,
            |_| {},
        ))
        .map(|(outcome, _)| outcome);
        (outcome, unasked.requests().len())
    };

    // The script edited to ask something else.
    let (outcome, sent) = resumed(&definition, "What is amber-8?");
    assert_eq!(
        failed(outcome).1,
        ScriptFailure::Diverged {
            exchange: 0,
            offer: false
        }
    );
    assert_eq!(sent, 0, "nothing sent");

    // lookup's description edited, which changes the offer.
    let edited = workflow(
        &YIELD_TYPES.replace("Looks a codeword up.", "Looks it up."),
        YIELDING,
    );
    let (outcome, sent) = resumed(&edited, "What is amber-7?");
    assert_eq!(
        failed(outcome).1,
        ScriptFailure::Diverged {
            exchange: 0,
            offer: true
        }
    );
    assert_eq!(sent, 0);

    // Unchanged, the replayed prompt has new identifiers and the same type and
    // content, and the run goes on to its one remaining request.
    let (outcome, sent) = resumed(&definition, "What is amber-7?");
    assert_eq!(rendered(outcome), "done");
    assert_eq!(sent, 1);
}

#[cfg(test)]
#[test]
fn continuation_counts_against_the_limit() {
    let stub = Stub::replying(vec![looking_up("call_7", "amber-7"), Reply::text("done")]);
    let (outcome, records) = run_keeping(
        &workflow(YIELD_TYPES, YIELDING),
        &yielding_behaviours("q", LOOKUP),
        &roster_at(&stub, "openai::m"),
        Limits {
            model_calls: 1,
            ..SMALL
        },
    );
    assert_eq!(
        failed(outcome),
        ("asker".to_owned(), ScriptFailure::ModelCallLimit)
    );
    assert_eq!(stub.requests().len(), 1, "the continuation is not sent");
    // lookup ran and its output is held.
    assert_eq!(outputs_in(records.last().expect("records")), 1);
}

#[cfg(test)]
#[test]
fn person_as_callee() {
    let definition = workflow(YIELD_TYPES, YIELDING);
    let behaviours = Behaviours::new()
        .define("ask", "ask.lua", &asking("What is amber-7?"))
        .person("lookup");
    let before = Stub::replying(vec![looking_up("call_7", "amber-7")]);
    let (outcome, records) = run_keeping(
        &definition,
        &behaviours,
        &roster_at(&before, "openai::m"),
        CALLING,
    );
    let Ok(Outcome::Awaiting(step)) = outcome else {
        panic!("the call is handed to a person: {outcome:?}")
    };
    assert_eq!(
        (step.instance(), step.node_type(), step.call()),
        ("asker", "lookup", Some("call_7"))
    );
    let [(parameter, given)] = step.inputs() else {
        panic!("one input: {:?}", step.inputs())
    };
    assert_eq!((parameter.as_str(), &*given.render()), ("query", "amber-7"));
    assert_eq!(before.requests().len(), 1);

    // Answered from the last record, in a run of its own: the asker's script
    // runs again, its first request answered from the record, and one more is
    // sent, carrying the person's text as the call's result.
    let after = Stub::replying(vec![Reply::text("done")]);
    let (outcome, _) = block(answer_scripted(
        &definition,
        &behaviours,
        &roster_at(&after, "openai::m"),
        records.last().expect("a record to answer from"),
        "asker",
        "the harbour is closed",
        CALLING,
        |_| {},
    ))
    .expect("answered");
    assert_eq!(rendered(Ok(outcome)), "done");
    let requests = after.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        sent_messages(&requests[0].1)
            .last()
            .expect("messages")
            .results,
        [("call_7".to_owned(), "the harbour is closed".to_owned())]
    );
}

#[cfg(test)]
#[test]
fn interrupted_call_resumes() {
    // The measured shape (EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS) the other way
    // round: two model calls in one script, the run dropped while the second
    // waits.
    let definition = workflow(
        YIELD_TYPES,
        "name = \"plain\"\noutput = \"asker\"\n\n[instances.asker]\nnode_type = \"ask\"\n",
    );
    let script = "local given, host = ...\nlocal a = host.complete('asking', host.text('note', 'one'))\nlocal b = host.complete('asking', host.text('note', 'two'))\nreturn host.compose(host.output, {a, b}, ' ')";
    let behaviours = Behaviours::new().define("ask", "ask.lua", script);

    let holding = Stub::holding_after(1, "first");
    let roster = roster_at(&holding, "openai::m");
    let mut source = IdSource::new();
    let mut records = Vec::new();
    let interrupted = block(async {
        tokio::select! {
            ending = run_scripted(
                &definition, &behaviours, &roster, Arguments::new(), &mut source, 20, CALLING,
                |record| records.push(record),
            ) => Some(ending),
            () = async {
                while holding.requests().len() < 2 {
                    tokio::task::yield_now().await;
                }
            } => None,
        }
    });
    assert!(interrupted.is_none(), "dropped during the second call");
    let last = records.last().expect("records");
    assert_eq!(kinds(last), ["exchange"], "the first answer is recorded");

    let answering = Stub::answering(200, "second");
    let (outcome, _) = block(resume_scripted(
        &definition,
        &behaviours,
        &roster_at(&answering, "openai::m"),
        last,
        CALLING,
        |_| {},
    ))
    .expect("resumes");
    assert_eq!(rendered(Ok(outcome)), "first second");
    // Only the second question is asked; the first answer is bought once.
    let requests = answering.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(sent_messages(&requests[0].1)[0].texts, ["two"]);
}
