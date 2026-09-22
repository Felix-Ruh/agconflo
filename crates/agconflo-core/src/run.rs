//! A run of one workflow definition: what it is started with, how its caller
//! drives it, and the one way it ends.
//!
//! The run decides what may activate and what a node is given; performing the
//! activation is the caller's (`DEC_RUN_IS_DRIVEN`). Nothing here opens a file,
//! calls a provider, spawns a thread or awaits anything, which is what keeps a
//! runtime, a script language and a provider client out of this crate until
//! there is a requirement for them.

use std::fmt;
use std::marker::PhantomData;

use crate::defect::WiringDefect;
use crate::scheduler::{Activation, Produced, next_activation};
use crate::workflow::WorkflowDefinition;
use crate::{Context, validate_wiring};

/// The contexts a run supplies to its workflow's entry parameters.
///
/// Addressed by the entry instance *and* the parameter each one fills, never by
/// the parameter name alone (`DEC_ARGUMENTS_PER_ENTRY`). Two instances of one
/// node type necessarily declare the same parameter names, so a name on its own
/// names two parameters - measured handing one context to both, with the run
/// completing and nothing reported (`EVD_RUN_ENTRY_NAME_SHARED`).
///
/// Every argument supplied is kept, including a second one for a pair already
/// supplied. A map keyed by the pair would drop one without a word, and a
/// parameter given two arguments is exactly the "not exactly one source" fault a
/// run refuses to start on (`CREQ_RUN_REFUSES_UNFILLED_SIGNATURE`), so it has to
/// survive being built to be reported at all.
#[derive(Clone, Debug, Default)]
pub struct Arguments {
    supplied: Vec<Argument>,
}

/// One context supplied for one entry instance's parameter.
#[derive(Clone, Debug)]
struct Argument {
    instance: String,
    parameter: String,
    context: Context,
}

impl Arguments {
    /// No arguments at all, which is what a workflow with no entry instance is
    /// started with.
    pub fn new() -> Self {
        Self::default()
    }

    /// The same arguments, also supplying `context` for `parameter` of
    /// `instance`.
    ///
    /// Chainable rather than taking `&mut self`, because a run's arguments are
    /// built once and read afterwards.
    pub fn supply(mut self, instance: &str, parameter: &str, context: Context) -> Self {
        self.supplied.push(Argument {
            instance: instance.to_owned(),
            parameter: parameter.to_owned(),
            context,
        });
        self
    }

    /// The context supplied for that instance's parameter, or `None` when none
    /// was.
    ///
    /// The first, when more than one was supplied. Which one is arbitrary and
    /// deliberately never reached: a run whose entry parameter has two arguments
    /// is refused before anything reads them.
    pub(crate) fn context_for(&self, instance: &str, parameter: &str) -> Option<&Context> {
        self.supplied
            .iter()
            .find(|a| a.instance == instance && a.parameter == parameter)
            .map(|a| &a.context)
    }
}

/// Why a run was not started.
///
/// A refusal is not one of the four ways a run ends (`DEC_RUN_ENDS_ONE_WAY`).
/// It is about what the caller supplied, it is known before any instance has
/// been looked at, and nothing happened - so a caller can tell a workflow it
/// must fix from a run that took place
/// (`DEC_RUN_REFUSED_BEFORE_IT_STARTS`).
///
/// `#[non_exhaustive]` because the classes are not finished, which is the
/// opposite of [`RunEnding`] and for the opposite reason.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum StartRefusal {
    /// The workflow carries wiring defects, all of them
    /// (`CREQ_RUN_REFUSAL_NAMES_EVERY_DEFECT`).
    Wiring(Vec<WiringDefect>),
}

impl fmt::Display for StartRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wiring(defects) => {
                write!(f, "the workflow carries {} wiring defect", defects.len())?;
                if defects.len() != 1 {
                    write!(f, "s")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for StartRefusal {}

/// Reporting an outcome when the run had offered no activation.
///
/// There is no instance to file it against, and choosing one would attribute
/// work to a node that never ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NothingOutstanding;

impl fmt::Display for NothingOutstanding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("no activation is outstanding")
    }
}

impl std::error::Error for NothingOutstanding {}

/// The one way a run ended.
///
/// Four, and the set is closed on purpose (`DEC_RUN_ENDS_ONE_WAY`): a caller
/// handling these four has handled everything that can become of a run that
/// started. Deliberately **not** `#[non_exhaustive]`, because saying the set may
/// grow would contradict the decision.
///
/// Generic over the caller's own failure type, which is what makes "which
/// failure occurred" a value rather than a message
/// (`CREQ_RUN_ENDS_ON_FAILURE`). A caller that reports no failure names
/// [`std::convert::Infallible`], which says so.
#[derive(Clone, Debug)]
pub enum RunEnding<F> {
    /// The designated instance produced, and this is what it produced
    /// (`CREQ_RUN_COMPLETES`).
    Completed(Context),
    /// The caller reported an activation as failed
    /// (`CREQ_RUN_ENDS_ON_FAILURE`).
    NodeFailed {
        /// The instance whose activation failed.
        instance: String,
        /// What the caller said went wrong.
        failure: F,
    },
    /// The run had activated as many instances as its budget allows
    /// (`CREQ_RUN_STOPS_AT_BUDGET`).
    BudgetExceeded {
        /// The budget it was held to.
        budget: usize,
    },
    /// Nothing more could activate and the designated instance had produced
    /// nothing (`CREQ_RUN_ENDS_QUIESCENT`).
    Quiescent {
        /// Every instance that produced nothing, in the order the definition
        /// carries them.
        waiting: Vec<String>,
    },
}

/// What a run has for its caller: one activation to perform, or its ending.
#[derive(Clone, Debug)]
pub enum Step<F> {
    /// Perform this activation and report back with [`Run::produced`].
    Activate(Activation),
    /// The run is over, this is how (`DEC_RUN_ENDS_ONE_WAY`).
    Ended(RunEnding<F>),
}

/// One run of one workflow definition.
///
/// Borrows the definition rather than owning it: a run reads it and never
/// changes it, and cloning one would copy the whole graph per run.
pub struct Run<'a, F> {
    definition: &'a WorkflowDefinition,
    arguments: Arguments,
    produced: Produced,
    outstanding: Option<Activation>,
    budget: usize,
    activations: usize,
    failure: PhantomData<fn() -> F>,
}

/// Written out rather than derived: a derived one would demand `F: Debug` of
/// every caller, though the failure type is only ever named here and never
/// held - a run that has failed has ended.
impl<F> fmt::Debug for Run<'_, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Run")
            .field("definition", &self.definition.name)
            .field("produced", &self.produced.len())
            .field(
                "outstanding",
                &self.outstanding.as_ref().map(Activation::instance),
            )
            .field("activations", &self.activations)
            .field("budget", &self.budget)
            .finish()
    }
}

impl<'a, F> Run<'a, F> {
    /// A run of `definition`, or a refusal saying why it cannot be started.
    ///
    /// The wiring is checked first and the run refuses on every defect the
    /// workflow carries. This is where `STKH_WIRING_CHECKED` stops being about
    /// what makes a workflow invalid and starts being about enforcement: the
    /// validator answers when something asks, and a run is the first thing in
    /// this project that can be said to have started, so it is the first place
    /// a refusal can come before anything.
    // @A run of a defective workflow does not start,IMPL_RUN_REFUSES_DEFECTS,impl,[CREQ_RUN_REFUSES_DEFECTS, CREQ_RUN_REFUSAL_NAMES_EVERY_DEFECT]
    pub fn start(
        definition: &'a WorkflowDefinition,
        arguments: Arguments,
        budget: usize,
    ) -> Result<Self, StartRefusal> {
        let defects = validate_wiring(definition);
        if !defects.is_empty() {
            return Err(StartRefusal::Wiring(defects));
        }

        Ok(Self {
            definition,
            arguments,
            produced: Produced::new(),
            outstanding: None,
            budget,
            activations: 0,
            failure: PhantomData,
        })
    }

    /// The next activation to perform, or the run's ending.
    ///
    /// Completion is asked before the budget, or a run that finished on its last
    /// permitted activation would be reported as a runaway. The budget is asked
    /// before an activation is offered, or the run would spend one more than it
    /// was allowed - which for a node that calls a provider is one unbudgeted
    /// call on every run that reaches the limit.
    ///
    /// Asked again before an outcome is reported, it hands back the activation
    /// it is already holding rather than choosing a second one, so the budget is
    /// spent once per activation rather than once per question.
    // @Completion then the budget then quiescence,IMPL_RUN_STEP,impl,[CREQ_RUN_COMPLETES, CREQ_RUN_STOPS_AT_BUDGET, CREQ_RUN_ENDS_QUIESCENT]
    pub fn step(&mut self) -> Step<F> {
        if let Some(result) = self.result() {
            return Step::Ended(RunEnding::Completed(result));
        }

        if let Some(outstanding) = &self.outstanding {
            return Step::Activate(outstanding.clone());
        }

        if self.activations >= self.budget {
            return Step::Ended(RunEnding::BudgetExceeded {
                budget: self.budget,
            });
        }

        match next_activation(self.definition, &self.arguments, &self.produced) {
            Some(activation) => {
                self.outstanding = Some(activation.clone());
                self.activations += 1;
                Step::Activate(activation)
            }
            None => Step::Ended(RunEnding::Quiescent {
                waiting: self.unproduced(),
            }),
        }
    }

    /// Report what the outstanding activation produced.
    pub fn produced(&mut self, context: Context) -> Result<(), NothingOutstanding> {
        let outstanding = self.outstanding.take().ok_or(NothingOutstanding)?;
        self.produced
            .insert(outstanding.instance().to_owned(), context);
        Ok(())
    }

    /// The context the designated instance produced, when it has.
    ///
    /// Completion is a question about the designated output alone rather than
    /// about the graph (`DEC_COMPLETION_IS_DESIGNATED_OUTPUT`), which is why a
    /// run finishes while instances no route to the result passes through sit
    /// idle - measured, and not a defect (`EVD_RUN_COMPLETES_WITH_IDLE`).
    ///
    /// Exactly one output is designated and it names an instance that exists,
    /// both of which the validator has already refused a workflow for, so this
    /// reads the one designation there is.
    fn result(&self) -> Option<Context> {
        let designated = self.definition.designated_outputs.first()?;
        self.produced.get(designated).cloned()
    }

    /// Every instance that has produced nothing, in the order the definition
    /// carries them.
    fn unproduced(&self) -> Vec<String> {
        self.definition
            .instances
            .iter()
            .filter(|node| !self.produced.contains_key(&node.name))
            .map(|node| node.name.clone())
            .collect()
    }
}

#[cfg(test)]
use std::convert::Infallible;

#[cfg(test)]
use crate::wiring::{any_definition, well_formed_definition};
#[cfg(test)]
use crate::workflow::{context_type, definition, instance, node_type};
#[cfg(test)]
use crate::{ContextId, IdSource};
#[cfg(test)]
use proptest::prelude::*;

/// A context of `type_name`, from a source the caller keeps.
#[cfg(test)]
fn ctx(source: &mut IdSource, type_name: &str) -> Context {
    Context::text(source, context_type(type_name), "x").expect("a fresh source issues")
}

/// Drive a run to its ending, producing a fresh context for each activation.
///
/// Returns the ending and, in order, every instance activated with the
/// identifier of what it produced - so that a test can assert on what the run
/// spent as well as on how it ended.
#[cfg(test)]
fn drive(
    workflow: &WorkflowDefinition,
    arguments: Arguments,
    budget: usize,
    source: &mut IdSource,
) -> (RunEnding<Infallible>, Vec<(String, ContextId)>) {
    let mut run =
        Run::<Infallible>::start(workflow, arguments, budget).expect("the wiring is sound");
    let mut activated = Vec::new();
    loop {
        match run.step() {
            Step::Ended(ending) => return (ending, activated),
            Step::Activate(activation) => {
                let produced = ctx(source, "note");
                activated.push((activation.instance().to_owned(), produced.id()));
                run.produced(produced)
                    .expect("an activation had just been offered");
            }
        }
    }
}

/// The instances a run activated, in order.
#[cfg(test)]
fn names(activated: &[(String, ContextId)]) -> Vec<&str> {
    activated.iter().map(|(name, _)| name.as_str()).collect()
}

/// A workflow of `count` instances in a chain, the last of them designated.
#[cfg(test)]
fn chain(count: usize) -> WorkflowDefinition {
    let types = vec![
        node_type("Src", &[], "note"),
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

#[cfg(test)]
#[test]
fn defective_workflow_is_refused() {
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Sink", &[("must", "note")], "note"),
    ];
    let broken = definition(
        types.clone(),
        vec![instance("sink", "Sink", &[("must", "ghost")])],
        &["sink"],
    );

    // Refused, and carrying wiring defects rather than one of the four endings:
    // a caller can tell a workflow it must fix from a run that happened.
    let refusal = Run::<Infallible>::start(&broken, Arguments::new(), 10)
        .expect_err("a defective workflow does not start");
    let StartRefusal::Wiring(defects) = refusal;
    assert!(!defects.is_empty());

    // No activation was offered, and nothing could have been: `start` handed
    // back no run at all, so the type system is what holds this rather than an
    // assertion that could go stale.

    // The control: the same shape with the wire repaired starts.
    let sound = definition(
        types,
        vec![
            instance("a", "Src", &[]),
            instance("sink", "Sink", &[("must", "a")]),
        ],
        &["sink"],
    );
    assert!(Run::<Infallible>::start(&sound, Arguments::new(), 10).is_ok());
}

#[cfg(test)]
#[test]
fn refusal_carries_every_defect() {
    let types = vec![
        node_type("Differ", &[], "diff"),
        node_type("Pair", &[("left", "note"), ("right", "note")], "note"),
        node_type("Sink", &[("input", "note")], "note"),
    ];
    let instances = vec![
        instance("a", "Differ", &[]),
        // Two defects on one instance: a required parameter unbound, and a wire
        // whose ends disagree.
        instance("b", "Pair", &[("left", "a")]),
        instance("c", "Sink", &[("input", "ghost")]),
        instance("d", "Missing", &[]),
    ];
    // And no designated output, which is a fifth defect of a fourth class.
    let broken = definition(types, instances, &[]);

    let refusal = Run::<Infallible>::start(&broken, Arguments::new(), 10)
        .expect_err("a defective workflow does not start");
    let StartRefusal::Wiring(defects) = refusal;

    // Compared against the validator's own report as a whole rather than
    // counted, so a run forwarding the right number of the wrong defects fails,
    // and so that adding a defect class later does not leave this stale.
    assert_eq!(defects, validate_wiring(&broken));
    assert_eq!(defects.len(), 5);
}

#[cfg(test)]
#[test]
fn completes_on_designated_output() {
    let mut source = IdSource::new();
    let workflow = chain(2);

    let (ending, activated) = drive(&workflow, Arguments::new(), 10, &mut source);
    assert_eq!(names(&activated), ["n0", "n1"]);
    match ending {
        RunEnding::Completed(result) => {
            // The context itself, which the case reads content and type from,
            // rather than an identifier a caller would have to hold the run to
            // resolve. Compared by identity as well, so that a different context
            // holding the same bytes would fail.
            assert_eq!(result.render(), "x");
            assert_eq!(result.declared_type().as_str(), "note");
            assert_eq!(result.id(), activated[1].1);
        }
        other => panic!("a workflow whose output produced should complete, not {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn completes_before_offering_more() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Step", &[("input", "note")], "note"),
    ];
    // A branch nothing is waiting for, written after the designated instance so
    // that what is asserted is the run ending rather than the order of a scan.
    let instances = vec![
        instance("a", "Src", &[]),
        instance("sink", "Step", &[("input", "a")]),
        instance("branch", "Src", &[]),
    ];
    let workflow = definition(types, instances, &["sink"]);

    let (ending, activated) = drive(&workflow, Arguments::new(), 10, &mut source);
    assert!(matches!(ending, RunEnding::Completed(_)));
    // The branch is never offered: a run that carried on would arrive at the
    // same result having spent an activation on it.
    assert_eq!(names(&activated), ["a", "sink"]);
}

#[cfg(test)]
#[test]
fn result_is_the_designated_context() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Pair", &[("left", "note"), ("right", "note")], "note"),
    ];
    // Three instances produce before the run completes, so a run returning any
    // produced context other than the designated instance's gives a different
    // answer.
    let instances = vec![
        instance("p", "Src", &[]),
        instance("q", "Src", &[]),
        instance("sink", "Pair", &[("left", "p"), ("right", "q")]),
    ];
    let workflow = definition(types, instances, &["sink"]);

    let (ending, activated) = drive(&workflow, Arguments::new(), 10, &mut source);
    assert_eq!(activated.len(), 3);
    let designated = activated
        .iter()
        .find(|(name, _)| name == "sink")
        .expect("the designated instance activated");
    match ending {
        RunEnding::Completed(result) => {
            assert_eq!(result.id(), designated.1);
            // And it is none of the others.
            for (name, id) in &activated {
                if name != "sink" {
                    assert_ne!(result.id(), *id);
                }
            }
        }
        other => panic!("expected completion, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn cycle_ends_quiescent() {
    let mut source = IdSource::new();
    let types = vec![node_type("Step", &[("input", "note")], "note")];
    let instances = vec![
        instance("c1", "Step", &[("input", "c2")]),
        instance("c2", "Step", &[("input", "c1")]),
    ];
    let workflow = definition(types, instances, &["c1"]);

    let (ending, activated) = drive(&workflow, Arguments::new(), 10, &mut source);
    assert!(activated.is_empty());
    match ending {
        RunEnding::Quiescent { waiting } => assert_eq!(waiting, ["c1", "c2"]),
        other => panic!("a cycle can do nothing, so the run is quiescent, not {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn idle_instance_does_not_make_it_stuck() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Step", &[("input", "note")], "note"),
    ];
    // `stuckone` is bound to its own output and can never become ready. The
    // designated instance does not depend on it, so the run finishes.
    let instances = vec![
        instance("live", "Src", &[]),
        instance("stuckone", "Step", &[("input", "stuckone")]),
    ];
    let workflow = definition(types, instances, &["live"]);

    let (ending, activated) = drive(&workflow, Arguments::new(), 10, &mut source);
    assert_eq!(names(&activated), ["live"]);
    assert!(
        matches!(ending, RunEnding::Completed(_)),
        "an instance that can never run is not a stuck run: {ending:?}"
    );
}

#[cfg(test)]
#[test]
fn quiescent_names_only_unproduced() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Step", &[("input", "note")], "note"),
    ];
    // `e` does its work; the rest are a cycle the designated instance sits in.
    let instances = vec![
        instance("e", "Src", &[]),
        instance("c1", "Step", &[("input", "c2")]),
        instance("c2", "Step", &[("input", "c1")]),
    ];
    let workflow = definition(types, instances, &["c1"]);

    let (ending, activated) = drive(&workflow, Arguments::new(), 10, &mut source);
    assert_eq!(names(&activated), ["e"]);
    match ending {
        RunEnding::Quiescent { waiting } => assert_eq!(waiting, ["c1", "c2"]),
        other => panic!("expected quiescence, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn budget_stops_at_the_limit() {
    let mut source = IdSource::new();
    let workflow = chain(3);

    let (ending, activated) = drive(&workflow, Arguments::new(), 2, &mut source);
    // Counted rather than inferred from the ending: a run that checks after
    // offering ends the same way having spent three.
    assert_eq!(names(&activated), ["n0", "n1"]);
    match ending {
        RunEnding::BudgetExceeded { budget } => assert_eq!(budget, 2),
        other => panic!("expected the budget to stop it, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn zero_budget_activates_nothing() {
    let mut source = IdSource::new();
    let workflow = chain(2);

    let (ending, activated) = drive(&workflow, Arguments::new(), 0, &mut source);
    assert!(
        activated.is_empty(),
        "a budget of nought is a limit, not the absence of one"
    );
    match ending {
        RunEnding::BudgetExceeded { budget } => assert_eq!(budget, 0),
        other => panic!("expected the budget to stop it, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn last_permitted_activation_completes() {
    let mut source = IdSource::new();
    let workflow = chain(2);

    // Exactly as many activations as the budget allows: the run finished, so it
    // completed rather than being reported as a runaway.
    let (ending, activated) = drive(&workflow, Arguments::new(), 2, &mut source);
    assert_eq!(names(&activated), ["n0", "n1"]);
    assert!(
        matches!(ending, RunEnding::Completed(_)),
        "a run that finished on its last permitted activation completed: {ending:?}"
    );
}

#[cfg(test)]
proptest! {
    /// However a run ends, it never offers more activations than its budget
    /// allows.
    #[test]
    fn activations_never_exceed_budget(workflow in any_definition(), budget in 0..6usize) {
        let mut source = IdSource::new();
        if let Ok(mut run) = Run::<Infallible>::start(&workflow, Arguments::new(), budget) {
            let mut activations = 0;
            loop {
                match run.step() {
                    Step::Ended(_) => break,
                    Step::Activate(_) => {
                        activations += 1;
                        prop_assert!(activations <= budget);
                        let produced = ctx(&mut source, "note");
                        run.produced(produced).expect("an activation was offered");
                    }
                }
            }
            prop_assert!(activations <= budget);
        }
    }

    /// A well-formed definition, every entry parameter supplied and a generous
    /// budget, completes - and its result is what its designated instance
    /// produced.
    ///
    /// The counterweight to a file of refusals: a run that refused everything
    /// and offered nothing would satisfy almost every other case here.
    #[test]
    fn well_formed_runs_complete(workflow in well_formed_definition()) {
        let mut source = IdSource::new();

        // Every entry instance's required parameters are the workflow's own.
        let mut arguments = Arguments::new();
        for node in &workflow.instances {
            if !node.entry {
                continue;
            }
            let declared = workflow
                .node_types
                .iter()
                .find(|declared| declared.name == node.node_type)
                .expect("a well-formed definition declares every type it names");
            for parameter in &declared.required {
                let supplied = ctx(&mut source, parameter.context_type.as_str());
                arguments = arguments.supply(&node.name, &parameter.name, supplied);
            }
        }

        let budget = workflow.instances.len();
        let (ending, activated) = drive(&workflow, arguments, budget, &mut source);
        let designated = &workflow.designated_outputs[0];
        let produced = activated
            .iter()
            .find(|(name, _)| name == designated)
            .map(|(_, id)| *id);

        match ending {
            RunEnding::Completed(result) => {
                prop_assert_eq!(Some(result.id()), produced);
            }
            other => prop_assert!(false, "a well-formed workflow completes, got {:?}", other),
        }
    }
}
