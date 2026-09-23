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
use crate::{Context, ContextId, ContextType, validate_wiring};

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

    /// How many arguments were supplied for that instance's parameter.
    ///
    /// More than one is a fault to report rather than a value to choose between
    /// (`CREQ_RUN_REFUSES_UNFILLED_SIGNATURE`), which is why they are all kept.
    pub(crate) fn count_for(&self, instance: &str, parameter: &str) -> usize {
        self.supplied
            .iter()
            .filter(|a| a.instance == instance && a.parameter == parameter)
            .count()
    }

    /// Every argument supplied, in the order it was supplied: the instance and
    /// parameter it fills, and the context filling it.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str, &Context)> {
        self.supplied
            .iter()
            .map(|a| (a.instance.as_str(), a.parameter.as_str(), &a.context))
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
    /// The workflow's wiring is sound and its entry parameters are not each
    /// filled by exactly one context of their declared type
    /// (`CREQ_RUN_REFUSES_UNFILLED_SIGNATURE`).
    Signature(Vec<SignatureFault>),
}

impl fmt::Display for StartRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (what, count) = match self {
            Self::Wiring(defects) => ("wiring defect", defects.len()),
            Self::Signature(faults) => ("signature fault", faults.len()),
        };
        write!(f, "the workflow carries {count} {what}")?;
        if count != 1 {
            write!(f, "s")?;
        }
        Ok(())
    }
}

impl std::error::Error for StartRefusal {}

/// One thing wrong with what a run was started with, rather than with the
/// workflow itself.
///
/// A value rather than a message, for the same reason a wiring defect is one: an
/// agent correcting its own call reads the place, not the prose. Every variant
/// names the entry instance and the parameter it concerns, because an argument
/// is addressed by that pair and never by a parameter name alone
/// (`DEC_ARGUMENTS_PER_ENTRY`).
///
/// `#[non_exhaustive]`: an entry instance's signature is the part of the model
/// still moving, and the shapes `components/wiring` records as open concern
/// exactly it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SignatureFault {
    /// A required entry parameter that no argument fills. Measured reading as a
    /// stuck run when nothing checked it (`EVD_RUN_MISSING_ARGUMENT_QUIESCES`).
    ParameterUnfilled {
        /// The entry instance whose parameter is unfilled.
        instance: String,
        /// The parameter no argument filled.
        parameter: String,
        /// The context type it is declared for.
        expected: ContextType,
    },
    /// An argument whose context type is not the one the parameter is declared
    /// for. The wiring validator refuses this between two instances and cannot
    /// see it here, because there is no binding to look at.
    ArgumentTypeDisagrees {
        /// The entry instance the argument was addressed to.
        instance: String,
        /// The parameter it fills.
        parameter: String,
        /// The context type the declaration names.
        expected: ContextType,
        /// The context type the argument actually carries.
        supplied: ContextType,
    },
    /// An entry parameter that also carries a binding, so it has two sources
    /// and no ground to prefer either. Sound wiring - `components/wiring`
    /// records it among the shapes still open - and what a loop closing back
    /// onto an entry node draws.
    ParameterAlsoBound {
        /// The entry instance whose parameter is both supplied and wired.
        instance: String,
        /// The parameter with two sources.
        parameter: String,
    },
    /// An entry parameter given more than one argument, which is the same fault
    /// reached from the other side.
    ParameterSuppliedTwice {
        /// The entry instance whose parameter was supplied twice.
        instance: String,
        /// The parameter with two arguments.
        parameter: String,
    },
    /// An argument addressed to an instance or a parameter the workflow has no
    /// entry parameter for. Ignoring it is what makes a typo silent.
    ArgumentMatchesNothing {
        /// The instance the argument named.
        instance: String,
        /// The parameter it named.
        parameter: String,
    },
}

impl fmt::Display for SignatureFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParameterUnfilled {
                instance,
                parameter,
                expected,
            } => write!(
                f,
                "{instance}.{parameter} is declared for {} and no argument fills it",
                expected.as_str()
            ),
            Self::ArgumentTypeDisagrees {
                instance,
                parameter,
                expected,
                supplied,
            } => write!(
                f,
                "{instance}.{parameter} is declared for {} and was given {}",
                expected.as_str(),
                supplied.as_str()
            ),
            Self::ParameterAlsoBound {
                instance,
                parameter,
            } => write!(
                f,
                "{instance}.{parameter} is an entry parameter and is also wired"
            ),
            Self::ParameterSuppliedTwice {
                instance,
                parameter,
            } => write!(f, "{instance}.{parameter} was given more than one argument"),
            Self::ArgumentMatchesNothing {
                instance,
                parameter,
            } => write!(
                f,
                "{instance}.{parameter} is not an entry parameter of this workflow"
            ),
        }
    }
}

impl std::error::Error for SignatureFault {}

/// Every way the arguments fail to fill the workflow's entry parameters exactly
/// once each.
///
/// Every entry parameter is looked at and every argument is looked at, so a
/// caller with three faults learns three - the same reason the validator reports
/// every wiring defect rather than the first.
///
/// The order is the definition's: its entry instances in the order it carries
/// them, each instance's parameters as its node type declares them, and then the
/// arguments that matched no parameter, in the order they were supplied.
// @A run whose signature is not filled does not start,IMPL_RUN_SIGNATURE,impl,[CREQ_RUN_REFUSES_UNFILLED_SIGNATURE]
fn signature_faults(definition: &WorkflowDefinition, arguments: &Arguments) -> Vec<SignatureFault> {
    let mut faults = Vec::new();
    let mut matched: Vec<(&str, &str)> = Vec::new();

    for node in definition.instances.iter().filter(|node| node.entry) {
        let Some(declared) = definition
            .node_types
            .iter()
            .find(|declared| declared.name == node.node_type)
        else {
            continue;
        };

        let listed = (declared.required.iter().map(|p| (p, true)))
            .chain(declared.optional.iter().map(|p| (p, false)));

        for (parameter, required) in listed {
            matched.push((&node.name, &parameter.name));

            // A wire is a second source whatever else is there, and preferring
            // either silently is the answer this refuses.
            if node.bindings.iter().any(|b| b.parameter == parameter.name) {
                faults.push(SignatureFault::ParameterAlsoBound {
                    instance: node.name.clone(),
                    parameter: parameter.name.clone(),
                });
                continue;
            }

            let supplied = arguments.count_for(&node.name, &parameter.name);
            if supplied > 1 {
                faults.push(SignatureFault::ParameterSuppliedTwice {
                    instance: node.name.clone(),
                    parameter: parameter.name.clone(),
                });
                continue;
            }

            match arguments.context_for(&node.name, &parameter.name) {
                Some(context) if context.declared_type() != &parameter.context_type => {
                    faults.push(SignatureFault::ArgumentTypeDisagrees {
                        instance: node.name.clone(),
                        parameter: parameter.name.clone(),
                        expected: parameter.context_type.clone(),
                        supplied: context.declared_type().clone(),
                    });
                }
                Some(_) => {}
                // An optional entry parameter may go unsupplied: refusing it
                // would make a workflow unusable that is not wrong.
                None if required => faults.push(SignatureFault::ParameterUnfilled {
                    instance: node.name.clone(),
                    parameter: parameter.name.clone(),
                    expected: parameter.context_type.clone(),
                }),
                None => {}
            }
        }
    }

    for (instance, parameter, _) in arguments.iter() {
        if !matched.contains(&(instance, parameter)) {
            faults.push(SignatureFault::ArgumentMatchesNothing {
                instance: instance.to_owned(),
                parameter: parameter.to_owned(),
            });
        }
    }

    faults
}

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

/// Why an output reported for an activation was not accepted.
///
/// Refusing an output ends nothing: the activation stays outstanding, counted
/// once, and the caller either reports an output the run accepts or reports the
/// activation failed (`DEC_REFUSED_OUTPUT_OUTSTANDING`). A fifth ending would
/// supersede the decision that there are four, and ending the run as a node's
/// failure would need a failure of the caller's type, which the run cannot make.
///
/// `#[non_exhaustive]` for the reason [`StartRefusal`] is: what a run can find
/// wrong with an output is not finished, where the ways it can end are.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum OutputRefusal {
    /// No activation was outstanding, so there is no instance the output could
    /// be filed against.
    NothingOutstanding,
    /// The output is not of the type the instance's node type declares
    /// (`CREQ_RUN_REFUSES_UNDECLARED_OUTPUT`). Measured accepted and handed on to
    /// a parameter declared for another type (`EVD_RUN_ACCEPTS_UNDECLARED_OUTPUT`).
    UndeclaredType {
        /// The instance the output was reported for.
        instance: String,
        /// The output type its node type declares.
        declared: ContextType,
        /// The type the reported output carries.
        reported: ContextType,
    },
    /// The output carries the identifier of an argument or of an output the run
    /// has already accepted (`CREQ_RUN_REFUSES_HELD_IDENTIFIER`), which would
    /// credit one context to two producers (`EVD_RUN_ACCEPTS_HELD_IDENTIFIER`).
    IdentifierHeld {
        /// The instance the output was reported for.
        instance: String,
        /// The identifier the run already holds.
        id: ContextId,
    },
}

impl fmt::Display for OutputRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NothingOutstanding => NothingOutstanding.fmt(f),
            Self::UndeclaredType {
                instance,
                declared,
                reported,
            } => write!(
                f,
                "{instance} is declared to produce {} and was reported producing {}",
                declared.as_str(),
                reported.as_str()
            ),
            Self::IdentifierHeld { instance, id } => write!(
                f,
                "{instance} was reported producing {id:?}, which the run already holds"
            ),
        }
    }
}

impl std::error::Error for OutputRefusal {}

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
    ///
    /// The signature is checked second and only when the wiring is sound, since
    /// an entry instance whose node type is missing has no parameter list to
    /// check arguments against - the faults would be derived from a graph
    /// already known to be broken.
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

        let faults = signature_faults(definition, &arguments);
        if !faults.is_empty() {
            return Err(StartRefusal::Signature(faults));
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

    /// Report that the outstanding activation failed, which ends the run.
    ///
    /// The failure is the caller's own type, carried into the ending as a value
    /// (`CREQ_RUN_ENDS_ON_FAILURE`). What went wrong is the caller's to say; that
    /// it ends the run, and that the ending carries it along with the instance
    /// it was reported for, is the run's.
    ///
    /// Consumes the run, so that no further activation can be offered - the
    /// failure mode is ruled out by the type rather than by a test. A run whose
    /// result is already unreachable should not go on spending activations, and
    /// for a node that calls a provider each one costs.
    ///
    /// Refused when no activation is outstanding: there is no instance to file
    /// the failure against, and choosing one would attribute it to a node that
    /// never ran.
    // @A failed activation ends the run,IMPL_RUN_FAILED,impl,[CREQ_RUN_ENDS_ON_FAILURE]
    pub fn fail(self, failure: F) -> Result<RunEnding<F>, NothingOutstanding> {
        let outstanding = self.outstanding.ok_or(NothingOutstanding)?;
        Ok(RunEnding::NodeFailed {
            instance: outstanding.instance().to_owned(),
            failure,
        })
    }

    /// Report what the outstanding activation produced.
    ///
    /// Refused, and nothing recorded, when the output is not of the type the
    /// instance's node type declares or carries an identifier the run already
    /// holds. Either way the activation stays outstanding and is not
    /// counted again (`DEC_REFUSED_OUTPUT_OUTSTANDING`): asking for the next
    /// step hands it back, so a refusal cannot be stepped past, and the caller
    /// answers again or reports the activation failed.
    // @An output is checked before it is recorded,IMPL_RUN_PRODUCED,impl,[CREQ_RUN_REFUSES_UNDECLARED_OUTPUT, CREQ_RUN_REFUSES_HELD_IDENTIFIER, CREQ_RUN_REFUSED_OUTPUT_OUTSTANDING]
    pub fn produced(&mut self, context: Context) -> Result<(), OutputRefusal> {
        let outstanding = self
            .outstanding
            .as_ref()
            .ok_or(OutputRefusal::NothingOutstanding)?;

        if context.declared_type() != outstanding.output() {
            return Err(OutputRefusal::UndeclaredType {
                instance: outstanding.instance().to_owned(),
                declared: outstanding.output().clone(),
                reported: context.declared_type().clone(),
            });
        }

        if self.holds(context.id()) {
            return Err(OutputRefusal::IdentifierHeld {
                instance: outstanding.instance().to_owned(),
                id: context.id(),
            });
        }

        let instance = outstanding.instance().to_owned();
        self.outstanding = None;
        self.produced.insert(instance, context);
        Ok(())
    }

    /// Whether `id` is the identifier of an argument the run was started with or
    /// of an output it has accepted - which is everything a run holds.
    ///
    /// Asked of the whole run rather than of the outstanding instance's inputs:
    /// a caller holding any context of the run can hand it back, including the
    /// output of an instance not wired to this one. Only the output's own
    /// identifier is asked about. A composition holding a held context by
    /// reference has an identifier of its own, and is the one sanctioned way to
    /// pass an input on.
    // @Everything a run holds,IMPL_RUN_HOLDS,impl,[CREQ_RUN_REFUSES_HELD_IDENTIFIER]
    fn holds(&self, id: ContextId) -> bool {
        self.arguments
            .iter()
            .any(|(_, _, context)| context.id() == id)
            || self.produced.values().any(|context| context.id() == id)
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
use crate::IdSource;
#[cfg(test)]
use crate::wiring::{any_definition, well_formed_definition};
#[cfg(test)]
use crate::workflow::{context_type, definition, instance, node_type};
#[cfg(test)]
use proptest::prelude::*;

/// A context of `type_name`, from a source the caller keeps.
#[cfg(test)]
fn ctx(source: &mut IdSource, type_name: &str) -> Context {
    Context::text(source, context_type(type_name), "x").expect("a fresh source issues")
}

/// Drive a run to its ending, producing a fresh context of the declared output
/// type for each activation.
///
/// The declared type rather than a fixed one: until the run checked outputs, a
/// fixed `note` here let `well_formed_runs_complete` pass on workflows whose
/// types declare other outputs, by feeding every consumer a mistyped context.
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
                let produced = ctx(source, activation.output().as_str());
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
    let StartRefusal::Wiring(defects) = refusal else {
        panic!("a broken wire is a wiring defect, not a signature fault: {refusal:?}")
    };
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
    let StartRefusal::Wiring(defects) = refusal else {
        panic!("every one of these is a wiring defect: {refusal:?}")
    };

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
                    Step::Activate(activation) => {
                        activations += 1;
                        prop_assert!(activations <= budget);
                        let produced = ctx(&mut source, activation.output().as_str());
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

/// The signature faults a refusal carries, or a panic naming what came instead.
#[cfg(test)]
fn signature(refusal: StartRefusal) -> Vec<SignatureFault> {
    match refusal {
        StartRefusal::Signature(faults) => faults,
        other => panic!("expected a signature refusal, got {other:?}"),
    }
}

/// One entry instance of a type requiring `seed` of `declared`, designated.
#[cfg(test)]
fn one_entry(declared: &str) -> WorkflowDefinition {
    let types = vec![node_type("Entry", &[("seed", declared)], declared)];
    let instances = vec![instance("e", "Entry", &[]).into_entry()];
    definition(types, instances, &["e"])
}

#[cfg(test)]
#[test]
fn missing_argument_is_refused() {
    let workflow = one_entry("note");

    // Which answer came back is the assertion, not merely that the run did not
    // complete: measured, this shape reads as a stuck run naming every waiting
    // instance (EVD_RUN_MISSING_ARGUMENT_QUIESCES), and a weaker assertion
    // passes against that.
    let refusal = Run::<Infallible>::start(&workflow, Arguments::new(), 10)
        .expect_err("a required entry parameter with no argument is refused");
    assert_eq!(
        signature(refusal),
        vec![SignatureFault::ParameterUnfilled {
            instance: "e".to_owned(),
            parameter: "seed".to_owned(),
            expected: context_type("note"),
        }]
    );
}

#[cfg(test)]
#[test]
fn argument_of_wrong_type_is_refused() {
    let mut source = IdSource::new();
    let workflow = one_entry("note");

    // No binding exists to compare, so the wiring validator cannot see this and
    // the run is the only thing that can.
    assert!(validate_wiring(&workflow).is_empty());

    let arguments = Arguments::new().supply("e", "seed", ctx(&mut source, "diff"));
    let refusal = Run::<Infallible>::start(&workflow, arguments, 10)
        .expect_err("an argument of the wrong context type is refused");
    assert_eq!(
        signature(refusal),
        vec![SignatureFault::ArgumentTypeDisagrees {
            instance: "e".to_owned(),
            parameter: "seed".to_owned(),
            expected: context_type("note"),
            supplied: context_type("diff"),
        }]
    );
}

#[cfg(test)]
#[test]
fn entry_parameter_also_bound_is_refused() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Entry", &[("seed", "note")], "note"),
    ];
    let instances = vec![
        instance("a", "Src", &[]),
        instance("e", "Entry", &[("seed", "a")]).into_entry(),
    ];
    let workflow = definition(types, instances, &["e"]);

    // The wiring is sound: a binding into an entry node is checked like any
    // other, which is what makes this the run's question.
    assert!(validate_wiring(&workflow).is_empty());

    let expected = vec![SignatureFault::ParameterAlsoBound {
        instance: "e".to_owned(),
        parameter: "seed".to_owned(),
    }];

    // Both arrangements, because a reader that silently prefers the argument
    // passes the first and one that silently prefers the wire passes the second.
    let with_argument = Arguments::new().supply("e", "seed", ctx(&mut source, "note"));
    let refusal = Run::<Infallible>::start(&workflow, with_argument, 10)
        .expect_err("a parameter with two sources is refused");
    assert_eq!(signature(refusal), expected);

    let refusal = Run::<Infallible>::start(&workflow, Arguments::new(), 10)
        .expect_err("a wired entry parameter is refused with no argument too");
    assert_eq!(signature(refusal), expected);
}

#[cfg(test)]
#[test]
fn argument_for_no_parameter_is_refused() {
    let mut source = IdSource::new();
    let workflow = one_entry("note");
    let good = ctx(&mut source, "note");

    // A typo in the instance name. Ignoring it leaves the real parameter
    // unfilled while the caller believes it supplied one.
    let arguments = Arguments::new().supply("e", "seed", good.clone()).supply(
        "ee",
        "seed",
        ctx(&mut source, "note"),
    );
    let refusal = Run::<Infallible>::start(&workflow, arguments, 10)
        .expect_err("an argument naming no instance is refused");
    assert_eq!(
        signature(refusal),
        vec![SignatureFault::ArgumentMatchesNothing {
            instance: "ee".to_owned(),
            parameter: "seed".to_owned(),
        }]
    );

    // And a typo in the parameter name.
    let arguments =
        Arguments::new()
            .supply("e", "seed", good)
            .supply("e", "sead", ctx(&mut source, "note"));
    let refusal = Run::<Infallible>::start(&workflow, arguments, 10)
        .expect_err("an argument naming no parameter is refused");
    assert_eq!(
        signature(refusal),
        vec![SignatureFault::ArgumentMatchesNothing {
            instance: "e".to_owned(),
            parameter: "sead".to_owned(),
        }]
    );

    // A parameter given two arguments is the same question from the other side.
    let arguments = Arguments::new()
        .supply("e", "seed", ctx(&mut source, "note"))
        .supply("e", "seed", ctx(&mut source, "note"));
    let refusal = Run::<Infallible>::start(&workflow, arguments, 10)
        .expect_err("a parameter supplied twice is refused");
    assert_eq!(
        signature(refusal),
        vec![SignatureFault::ParameterSuppliedTwice {
            instance: "e".to_owned(),
            parameter: "seed".to_owned(),
        }]
    );

    // Both typos at once are two faults, not the first of them. A caller that
    // learns its faults one round trip at a time is the cost this exists to
    // avoid, and nothing else here has more than one fault to report.
    let arguments = Arguments::new()
        .supply("e", "seed", ctx(&mut source, "note"))
        .supply("ee", "seed", ctx(&mut source, "note"))
        .supply("e", "sead", ctx(&mut source, "note"));
    let refusal = Run::<Infallible>::start(&workflow, arguments, 10)
        .expect_err("two arguments matching nothing are refused");
    assert_eq!(
        signature(refusal),
        vec![
            SignatureFault::ArgumentMatchesNothing {
                instance: "ee".to_owned(),
                parameter: "seed".to_owned(),
            },
            SignatureFault::ArgumentMatchesNothing {
                instance: "e".to_owned(),
                parameter: "sead".to_owned(),
            },
        ]
    );
}

#[cfg(test)]
#[test]
fn signature_filled_exactly_starts() {
    let mut source = IdSource::new();
    // Two entry instances whose node types each declare a parameter called the
    // same thing, for two different context types - the shape measured handing
    // one context to both (EVD_RUN_ENTRY_NAME_SHARED). They are two parameters.
    // `p` and `p2` are two instances of one node type, so they necessarily
    // declare the same parameter names and are two parameters all the same.
    let types = vec![
        node_type("Alpha", &[("seed", "note")], "note"),
        node_type("Beta", &[("seed", "diff")], "note").with_optional(&[("hint", "note")]),
        node_type(
            "Join",
            &[("left", "note"), ("right", "note"), ("extra", "note")],
            "note",
        ),
    ];
    let instances = vec![
        instance("p", "Alpha", &[]).into_entry(),
        instance("q", "Beta", &[]).into_entry(),
        instance("p2", "Alpha", &[]).into_entry(),
        instance(
            "j",
            "Join",
            &[("left", "p"), ("right", "q"), ("extra", "p2")],
        ),
    ];
    let workflow = definition(types, instances, &["j"]);

    // One argument per required entry parameter, each of the declared type, and
    // `q`'s optional parameter deliberately unsupplied: refusing that would make
    // a workflow unusable that is not wrong.
    let alpha_seed = ctx(&mut source, "note");
    let beta_seed = ctx(&mut source, "diff");
    let (alpha_id, beta_id) = (alpha_seed.id(), beta_seed.id());
    let p2_seed = ctx(&mut source, "note");
    let p2_id = p2_seed.id();
    let arguments = Arguments::new()
        .supply("p", "seed", alpha_seed)
        .supply("q", "seed", beta_seed)
        .supply("p2", "seed", p2_seed);

    let mut run =
        Run::<Infallible>::start(&workflow, arguments, 10).expect("the signature is filled");

    // Each entry instance is given its own argument rather than one shared by
    // name, which is the whole point of addressing them per instance.
    let mut seen = Vec::new();
    loop {
        match run.step() {
            Step::Ended(ending) => {
                assert!(matches!(ending, RunEnding::Completed(_)), "{ending:?}");
                break;
            }
            Step::Activate(activation) => {
                // Same node type, same parameter name, its own argument.
                if activation.instance() == "p" {
                    assert_eq!(activation.inputs()[0].1.id(), alpha_id);
                }
                if activation.instance() == "p2" {
                    assert_eq!(activation.inputs()[0].1.id(), p2_id);
                }
                if activation.instance() == "q" {
                    assert_eq!(
                        activation.inputs().len(),
                        1,
                        "the optional one is unsupplied"
                    );
                    assert_eq!(activation.inputs()[0].1.id(), beta_id);
                }
                seen.push(activation.instance().to_owned());
                let produced = ctx(&mut source, "note");
                run.produced(produced).expect("an activation was offered");
            }
        }
    }
    assert_eq!(seen, ["p", "q", "p2", "j"]);
}

/// A caller's own failure type. Named variants rather than a string, because
/// asserting which failure occurred is what every error-path case here owes
/// (`STKH_TYPED_FAILURE`), and prose cannot be asserted on.
#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
enum NodeTrouble {
    Refused(&'static str),
    TimedOut,
}

/// A workflow of two instances that are both ready from the start, the first
/// designated.
#[cfg(test)]
fn two_ready() -> WorkflowDefinition {
    let types = vec![node_type("Src", &[], "note")];
    let instances = vec![instance("a", "Src", &[]), instance("b", "Src", &[])];
    definition(types, instances, &["a"])
}

#[cfg(test)]
#[test]
fn failed_activation_ends_the_run() {
    let mut source = IdSource::new();
    let workflow = two_ready();

    let mut run = Run::<NodeTrouble>::start(&workflow, Arguments::new(), 10).expect("sound");
    let Step::Activate(activation) = run.step() else {
        panic!("an instance with nothing to wait for is offered")
    };
    assert_eq!(activation.instance(), "a");

    let ending = run
        .fail(NodeTrouble::Refused("the node said no"))
        .expect("an activation was outstanding");

    // The instance is named, and the failure is carried as the caller's own
    // value: matched here rather than read out of a message, which is what a
    // rendered failure would force and what this exists to rule out.
    match ending {
        RunEnding::NodeFailed { instance, failure } => {
            assert_eq!(instance, "a");
            assert_eq!(failure, NodeTrouble::Refused("the node said no"));
        }
        other => panic!("a reported failure ends the run as a failure, not {other:?}"),
    }

    // That the run offers nothing further is held by the type rather than by an
    // assertion: `fail` consumes the run, so carrying on does not compile. A
    // test could only check that this caller did not carry on.
    crate::compile_fail::assert_refused(
        "run_continues_after_failing",
        "fn after(mut run: agconflo_core::Run<'static, ()>) { let _ = run.fail(()); let _ = run.step(); }",
        "E0382",
        "moved value",
    );

    // `b` was ready the whole time, which is what makes the case bite: without
    // another instance waiting, a run that carried on would be indistinguishable
    // from one that stopped. The same workflow driven without a failure goes on
    // to activate it.
    let (_, activated) = drive(&workflow, Arguments::new(), 10, &mut source);
    assert_eq!(names(&activated), ["a"]);

    // And with `b` designated instead, it is reached - so it really was ready.
    let types = vec![node_type("Src", &[], "note")];
    let instances = vec![instance("a", "Src", &[]), instance("b", "Src", &[])];
    let b_designated = definition(types, instances, &["b"]);
    let (_, activated) = drive(&b_designated, Arguments::new(), 10, &mut source);
    assert_eq!(names(&activated), ["a", "b"]);
}

#[cfg(test)]
#[test]
fn failure_with_no_activation_is_refused() {
    let mut source = IdSource::new();
    let workflow = two_ready();

    // Nothing has been offered yet.
    let run = Run::<NodeTrouble>::start(&workflow, Arguments::new(), 10).expect("sound");
    assert_eq!(
        run.fail(NodeTrouble::TimedOut).unwrap_err(),
        NothingOutstanding
    );

    // And nothing is outstanding once an outcome has been reported for it.
    let mut run = Run::<NodeTrouble>::start(&workflow, Arguments::new(), 10).expect("sound");
    let Step::Activate(_) = run.step() else {
        panic!("an instance with nothing to wait for is offered")
    };
    run.produced(ctx(&mut source, "note"))
        .expect("an activation was outstanding");
    assert_eq!(
        run.fail(NodeTrouble::TimedOut).unwrap_err(),
        NothingOutstanding
    );
}

/// The activation a run offers next, or a panic naming what came instead.
#[cfg(test)]
fn offered<F: fmt::Debug>(run: &mut Run<'_, F>) -> Activation {
    match run.step() {
        Step::Activate(activation) => activation,
        Step::Ended(ending) => panic!("expected an activation, the run ended: {ending:?}"),
    }
}

#[cfg(test)]
#[test]
fn undeclared_output_is_refused() {
    let mut source = IdSource::new();
    let workflow = chain(2);
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");

    assert_eq!(offered(&mut run).instance(), "n0");
    // The measured defect (EVD_RUN_ACCEPTS_UNDECLARED_OUTPUT): `n0`'s type
    // declares `note`, and it is answered with a `diff`.
    let refusal = run
        .produced(ctx(&mut source, "diff"))
        .expect_err("an output of an undeclared type is refused");
    // Matched as a whole, so a refusal naming the wrong instance or either type
    // wrongly fails, and so a refusal of the other kind cannot stand in for it.
    assert_eq!(
        refusal,
        OutputRefusal::UndeclaredType {
            instance: "n0".to_owned(),
            declared: context_type("note"),
            reported: context_type("diff"),
        }
    );

    // Refused before it was recorded: had it been recorded, `n1` - bound to
    // `n0` - would be ready and offered now.
    assert_eq!(offered(&mut run).instance(), "n0");
}

#[cfg(test)]
#[test]
fn designated_undeclared_output_is_refused() {
    let mut source = IdSource::new();
    // Nothing consumes the designated instance's output, so a run comparing
    // outputs with the parameters they reach checks nothing here.
    let workflow = definition(
        vec![node_type("Src", &[], "note")],
        vec![instance("only", "Src", &[])],
        &["only"],
    );
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");

    offered(&mut run);
    let refusal = run
        .produced(ctx(&mut source, "diff"))
        .expect_err("the result's type is checked too");
    assert_eq!(
        refusal,
        OutputRefusal::UndeclaredType {
            instance: "only".to_owned(),
            declared: context_type("note"),
            reported: context_type("diff"),
        }
    );
    // And the run did not complete with the mistyped result.
    assert_eq!(offered(&mut run).instance(), "only");
}

#[cfg(test)]
#[test]
fn output_of_declared_type_is_accepted() {
    let mut source = IdSource::new();
    // `Summ` changes the type: it takes a `note` and produces a `summary`, so a
    // run comparing an output with its inputs refuses it for being right.
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Summ", &[("input", "note")], "summary"),
    ];
    let workflow = definition(
        types,
        vec![
            instance("a", "Src", &[]),
            instance("s", "Summ", &[("input", "a")]),
        ],
        &["s"],
    );

    // Answered with a plain context of the declared type.
    let (ending, _) = drive(&workflow, Arguments::new(), 10, &mut source);
    assert!(matches!(ending, RunEnding::Completed(_)), "{ending:?}");

    // And with a composition of the declared type whose part is a `note`: a
    // composition's type is the one it was declared with, whatever its parts'.
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");
    offered(&mut run);
    run.produced(ctx(&mut source, "note"))
        .expect("of the declared type");
    let activation = offered(&mut run);
    let composed = Context::compose(
        &mut source,
        context_type("summary"),
        [&activation.inputs()[0].1],
        "",
    )
    .expect("a fresh source issues");
    run.produced(composed)
        .expect("a composition of the declared type is accepted");
    assert!(matches!(run.step(), Step::Ended(RunEnding::Completed(_))));
}

#[cfg(test)]
#[test]
fn passed_through_argument_is_refused() {
    let mut source = IdSource::new();
    // Declared `note` in and `note` out, so only the identifier is wrong.
    let workflow = one_entry("note");
    let argument = ctx(&mut source, "note");
    let held = argument.id();
    let arguments = Arguments::new().supply("e", "seed", argument);
    let mut run = Run::<Infallible>::start(&workflow, arguments, 10).expect("sound");

    let activation = offered(&mut run);
    let refusal = run
        .produced(activation.inputs()[0].1.clone())
        .expect_err("an argument handed back is refused");
    assert_eq!(
        refusal,
        OutputRefusal::IdentifierHeld {
            instance: "e".to_owned(),
            id: held,
        }
    );
}

#[cfg(test)]
#[test]
fn passed_through_output_is_refused() {
    let mut source = IdSource::new();
    // `a` and `b` are not wired to each other, so a run asking only about the
    // instance's own inputs finds nothing to compare.
    let workflow = definition(
        vec![node_type("Src", &[], "note")],
        vec![instance("a", "Src", &[]), instance("b", "Src", &[])],
        &["b"],
    );
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");

    assert_eq!(offered(&mut run).instance(), "a");
    let from_a = ctx(&mut source, "note");
    run.produced(from_a.clone()).expect("a new context");

    assert_eq!(offered(&mut run).instance(), "b");
    let refusal = run
        .produced(from_a.clone())
        .expect_err("another instance's output handed back is refused");
    assert_eq!(
        refusal,
        OutputRefusal::IdentifierHeld {
            instance: "b".to_owned(),
            id: from_a.id(),
        }
    );
}

#[cfg(test)]
#[test]
fn output_composing_its_input_is_accepted() {
    let mut source = IdSource::new();
    let workflow = chain(2);
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");

    offered(&mut run);
    run.produced(ctx(&mut source, "note"))
        .expect("a new context");

    let activation = offered(&mut run);
    let input = activation.inputs()[0].1.clone();
    let composed = Context::compose(&mut source, context_type("note"), [&input], "")
        .expect("a fresh source issues");
    let composed_id = composed.id();
    run.produced(composed)
        .expect("a composition holding the input is new to the run");

    // The result is the composition, holding the input as itself.
    match run.step() {
        Step::Ended(RunEnding::Completed(result)) => {
            assert_eq!(result.id(), composed_id);
            assert_ne!(result.id(), input.id());
            assert_eq!(result.parts()[0].id(), input.id());
        }
        other => panic!("expected completion, got {other:?}"),
    }
}

/// A chain of `count` instances whose first is an entry instance taking one
/// `note` argument, the last designated.
#[cfg(test)]
fn entry_chain(count: usize) -> WorkflowDefinition {
    let types = vec![
        node_type("Entry", &[("seed", "note")], "note"),
        node_type("Step", &[("input", "note")], "note"),
    ];
    let mut instances = vec![instance("n0", "Entry", &[]).into_entry()];
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

/// What a caller answers one activation with.
#[cfg(test)]
#[derive(Clone, Debug)]
enum Answer {
    /// A new context built from nothing the run holds.
    Fresh,
    /// A composition of the held context at this index, modulo how many there
    /// are.
    Compose(usize),
    /// The held context at this index handed back unchanged, and a new context
    /// once that has been refused.
    HandBack(usize),
}

#[cfg(test)]
proptest! {
    /// However a caller answers, an output is refused exactly when it was handed
    /// back, and nothing the run was started with or accepted shares an
    /// identifier with anything else it holds.
    #[test]
    fn accepted_outputs_are_all_new(
        answers in prop::collection::vec(
            prop_oneof![
                Just(Answer::Fresh),
                any::<usize>().prop_map(Answer::Compose),
                any::<usize>().prop_map(Answer::HandBack),
            ],
            1..7,
        ),
    ) {
        let mut source = IdSource::new();
        let workflow = entry_chain(answers.len());
        let argument = ctx(&mut source, "note");
        let mut held = vec![argument.clone()];
        let arguments = Arguments::new().supply("n0", "seed", argument);
        let mut run = Run::<Infallible>::start(&workflow, arguments, answers.len())
            .expect("sound");

        for answer in &answers {
            let activation = offered(&mut run);
            let pick = |index: usize| held[index % held.len()].clone();
            let accepted = match answer {
                Answer::Fresh => ctx(&mut source, "note"),
                Answer::Compose(index) => {
                    let part = pick(*index);
                    Context::compose(&mut source, context_type("note"), [&part], "")
                        .expect("a fresh source issues")
                }
                Answer::HandBack(index) => {
                    let back = pick(*index);
                    prop_assert_eq!(
                        run.produced(back.clone()),
                        Err(OutputRefusal::IdentifierHeld {
                            instance: activation.instance().to_owned(),
                            id: back.id(),
                        })
                    );
                    ctx(&mut source, "note")
                }
            };
            prop_assert_eq!(run.produced(accepted.clone()), Ok(()));
            held.push(accepted);
        }

        prop_assert!(matches!(run.step(), Step::Ended(RunEnding::Completed(_))));
        let ids: std::collections::HashSet<_> = held.iter().map(Context::id).collect();
        prop_assert_eq!(ids.len(), held.len());
    }
}

#[cfg(test)]
#[test]
fn refused_output_keeps_the_activation() {
    let mut source = IdSource::new();
    let workflow = chain(2);
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");

    assert_eq!(offered(&mut run).instance(), "n0");
    run.produced(ctx(&mut source, "diff"))
        .expect_err("an undeclared type is refused");

    // Not ended, and the same instance again rather than its consumer.
    assert_eq!(offered(&mut run).instance(), "n0");

    // An accepted answer lets the run go on to completion.
    run.produced(ctx(&mut source, "note"))
        .expect("of the declared type");
    assert_eq!(offered(&mut run).instance(), "n1");
    run.produced(ctx(&mut source, "note"))
        .expect("of the declared type");
    assert!(matches!(run.step(), Step::Ended(RunEnding::Completed(_))));
}

#[cfg(test)]
#[test]
fn refusal_does_not_spend_the_budget() {
    let mut source = IdSource::new();
    let workflow = chain(2);
    // Exactly as many activations as there are instances.
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 2).expect("sound");

    offered(&mut run);
    run.produced(ctx(&mut source, "diff"))
        .expect_err("an undeclared type is refused");
    offered(&mut run);
    run.produced(ctx(&mut source, "note"))
        .expect("of the declared type");
    assert_eq!(offered(&mut run).instance(), "n1");
    run.produced(ctx(&mut source, "note"))
        .expect("of the declared type");

    // Counted a second time, the refused activation would have left no budget
    // for `n1`, and the run would have ended on it instead.
    match run.step() {
        Step::Ended(RunEnding::Completed(_)) => {}
        other => panic!("a refusal spends nothing, so the run completes: {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn refused_output_can_be_failed() {
    let mut source = IdSource::new();
    let workflow = chain(2);
    let mut run = Run::<NodeTrouble>::start(&workflow, Arguments::new(), 10).expect("sound");

    offered(&mut run);
    run.produced(ctx(&mut source, "diff"))
        .expect_err("an undeclared type is refused");

    // Still outstanding, so the caller can give up on it.
    match run.fail(NodeTrouble::Refused("no acceptable output")) {
        Ok(RunEnding::NodeFailed { instance, failure }) => {
            assert_eq!(instance, "n0");
            assert_eq!(failure, NodeTrouble::Refused("no acceptable output"));
        }
        other => panic!("a refused activation can be failed, got {other:?}"),
    }
}
