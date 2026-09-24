//! A run of one workflow definition: what it is started with, how its caller
//! drives it, and the one way it ends.
//!
//! The run decides what may activate and what a node is given; performing the
//! activation is the caller's (`DEC_RUN_IS_DRIVEN`). Nothing here opens a file,
//! calls a provider, spawns a thread or awaits anything, which is what keeps a
//! runtime, a script language and a provider client out of this crate until
//! there is a requirement for them.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;

use crate::defect::WiringDefect;
use crate::scheduler::{Activation, Produced, next_activation};
use crate::workflow::{NodeType, WorkflowDefinition};
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
    /// Different contexts among the arguments, or among what they were composed
    /// from, share these identifiers, each named once in the order found
    /// (`CREQ_RUN_REFUSES_SHARED_ARGUMENT_IDENTIFIER`). Not a signature fault:
    /// a shared identifier belongs to no one entry parameter.
    SharedIdentifiers(Vec<ContextId>),
}

impl fmt::Display for StartRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (what, count) = match self {
            Self::Wiring(defects) => ("wiring defect", defects.len()),
            Self::Signature(faults) => ("signature fault", faults.len()),
            Self::SharedIdentifiers(ids) => ("shared identifier", ids.len()),
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

/// Everything the run's arguments hold, or every identifier that different
/// contexts among them share.
///
/// Each argument is brought in as an output would be, against what the
/// arguments before it brought, so one context supplied twice is one context,
/// and two contexts under one identifier are found wherever in the arguments
/// they sit. Every shared identifier is collected, named once each, so that a
/// caller whose arguments came from two sources learns all of it at once.
// @Arguments hold one context per identifier,IMPL_RUN_HELD_ARGUMENTS,impl,[CREQ_RUN_REFUSES_SHARED_ARGUMENT_IDENTIFIER]
fn held_arguments(arguments: &Arguments) -> Result<HashMap<ContextId, Context>, Vec<ContextId>> {
    let mut held = HashMap::new();
    let mut shared: Vec<ContextId> = Vec::new();
    for (_, _, argument) in arguments.iter() {
        let (brought, found) = brought_in(&[&held], argument);
        held.extend(brought);
        for id in found {
            if !shared.contains(&id) {
                shared.push(id);
            }
        }
    }
    if shared.is_empty() {
        Ok(held)
    } else {
        Err(shared)
    }
}

/// What `context` would bring into a run holding `known` - the run's own
/// contexts and those of each activation in progress: every context reachable
/// from it, itself included, that none of them holds - and every identifier
/// under which it reaches a context other than the one they, or an earlier step
/// of the same walk, have under that identifier.
///
/// A held context reached again is the very context held, and the walk stops
/// there: its own parts were brought in when it was. So a walk costs what the
/// context brings in rather than its whole ancestry, and passing an input on by
/// composing it (`DEC_COMPOSITION_BY_REFERENCE`) brings in nothing but the
/// composition. "The very context" is the value's identity, since by identifier
/// a second context and the held one are indistinguishable
/// (`DEC_IDENTIFIER_NAMES_ONE_CONTEXT`).
///
/// A stack of its own rather than recursion, for the reason the lineage walker
/// has one.
// @A second context under a held identifier is found,IMPL_RUN_BROUGHT_IN,impl,[CREQ_RUN_REFUSES_SHARED_ARGUMENT_IDENTIFIER, CREQ_RUN_REFUSES_SHARED_OUTPUT_IDENTIFIER]
fn brought_in(
    known: &[&HashMap<ContextId, Context>],
    context: &Context,
) -> (HashMap<ContextId, Context>, Vec<ContextId>) {
    let mut brought: HashMap<ContextId, Context> = HashMap::new();
    let mut shared = Vec::new();
    let mut pending = vec![context];
    while let Some(reached) = pending.pop() {
        match known
            .iter()
            .find_map(|held| held.get(&reached.id()))
            .or_else(|| brought.get(&reached.id()))
        {
            Some(known) if known.is(reached) => {}
            Some(_) => {
                if !shared.contains(&reached.id()) {
                    shared.push(reached.id());
                }
            }
            None => {
                brought.insert(reached.id(), reached.clone());
                pending.extend(reached.parts());
            }
        }
    }
    (brought, shared)
}

/// Every way `call` misses the declaration of the node type it calls: each input
/// in the order given, then each required parameter it leaves unfilled.
fn call_faults(declared: &NodeType, call: &Call) -> Vec<CallFault> {
    let mut faults = Vec::new();
    let mut filled = HashSet::new();
    for (parameter, given) in &call.inputs {
        if !filled.insert(parameter.as_str()) {
            faults.push(CallFault::FilledTwice {
                parameter: parameter.clone(),
            });
            continue;
        }
        let declaration = declared
            .required
            .iter()
            .chain(&declared.optional)
            .find(|declaration| declaration.name == *parameter);
        match declaration {
            None => faults.push(CallFault::UndeclaredParameter {
                parameter: parameter.clone(),
            }),
            Some(declaration) if declaration.context_type != *given.declared_type() => {
                faults.push(CallFault::WrongType {
                    parameter: parameter.clone(),
                    declared: declaration.context_type.clone(),
                    given: given.declared_type().clone(),
                });
            }
            Some(_) => {}
        }
    }
    for required in &declared.required {
        if !filled.contains(required.name.as_str()) {
            faults.push(CallFault::RequiredUnfilled {
                parameter: required.name.clone(),
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
    /// The output is not of the type the activation's node type declares
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
    /// The output carries an identifier the run already holds, or that the
    /// activation waiting on this one's call holds
    /// (`CREQ_RUN_REFUSES_HELD_IDENTIFIER`), which would credit one context to
    /// two producers (`EVD_RUN_ACCEPTS_HELD_IDENTIFIER`).
    IdentifierHeld {
        /// The instance the output was reported for.
        instance: String,
        /// The identifier the run already holds.
        id: ContextId,
    },
    /// A context the output was composed from carries an identifier the run,
    /// or the output itself, holds for a different context
    /// (`CREQ_RUN_REFUSES_SHARED_OUTPUT_IDENTIFIER`). Measured accepted, and the
    /// result's lineage then lost a context (`EVD_RUN_PART_SHARES_IDENTIFIER`).
    IdentifierShared {
        /// The instance the output was reported for.
        instance: String,
        /// The first identifier found shared.
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
            Self::IdentifierShared { instance, id } => write!(
                f,
                "{instance} was reported producing a context composed of a second context under {id:?}"
            ),
        }
    }
}

impl std::error::Error for OutputRefusal {}

/// One call a model made during an activation, as the activation's performer
/// reports it: the provider's identifier for it, the node type called, and a
/// context for each parameter the call fills (`DEC_CALL_CARRIES_STRING_VALUES`).
///
/// Built with [`Call::new`] and [`Call::input`], and checked only when it is
/// reported: a call has to be able to hold every shape a model can send, or what
/// [`Run::call`] refuses could not be put to it.
#[derive(Clone, Debug)]
pub struct Call {
    id: String,
    node_type: String,
    inputs: Vec<(String, Context)>,
}

impl Call {
    /// A call, with the provider's identifier `id`, to `node_type`, filling no
    /// parameter yet.
    pub fn new(id: &str, node_type: &str) -> Self {
        Self {
            id: id.to_owned(),
            node_type: node_type.to_owned(),
            inputs: Vec::new(),
        }
    }

    /// The same call, also giving `context` for `parameter`.
    pub fn input(mut self, parameter: &str, context: Context) -> Self {
        self.inputs.push((parameter.to_owned(), context));
        self
    }

    /// The provider's identifier for this call, as it issued it.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The node type called.
    pub fn node_type(&self) -> &str {
        &self.node_type
    }

    /// Each parameter the call fills, with its context, in the order given.
    pub fn inputs(&self) -> &[(String, Context)] {
        &self.inputs
    }
}

/// What an activation's performer sent out and got back: the offer and the
/// window a model call was made with, the answer, and the identifiers of the
/// calls the answer made (`DEC_RECORD_HOLDS_EXCHANGES`).
///
/// The core knows nothing of models. An exchange is contexts an activation sent
/// and a context that came back, which is what makes it the run's to hold and
/// its record's to write.
#[derive(Clone, Debug)]
pub struct Exchange {
    offer: Vec<Context>,
    window: Context,
    answer: Context,
    calls: Vec<String>,
}

impl Exchange {
    /// An exchange that sent `window`, offering nothing, and got `answer` back,
    /// making no call.
    pub fn new(window: Context, answer: Context) -> Self {
        Self {
            offer: Vec::new(),
            window,
            answer,
            calls: Vec::new(),
        }
    }

    /// The same exchange, having offered `offer` as well.
    pub fn offering(mut self, offer: Vec<Context>) -> Self {
        self.offer = offer;
        self
    }

    /// The same exchange, its answer also making the call identified by `id`.
    pub fn calling(mut self, id: &str) -> Self {
        self.calls.push(id.to_owned());
        self
    }

    /// What was offered with the window, one context per thing offered.
    pub fn offer(&self) -> &[Context] {
        &self.offer
    }

    /// The window sent.
    pub fn window(&self) -> &Context {
        &self.window
    }

    /// The answer that came back.
    pub fn answer(&self) -> &Context {
        &self.answer
    }

    /// The identifiers of the calls the answer made, in the order it made them.
    pub fn calls(&self) -> &[String] {
        &self.calls
    }

    /// Every context the exchange holds: the offer, then the window, then the
    /// answer.
    fn contexts(&self) -> impl Iterator<Item = &Context> {
        self.offer.iter().chain([&self.window, &self.answer])
    }
}

/// Why a reported call was not accepted.
///
/// A refused call is no activation: nothing is offered, the budget is not
/// spent, and the activation that made it stays outstanding.
///
/// `#[non_exhaustive]` for the reason [`OutputRefusal`] is.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CallRefusal {
    /// No activation was outstanding, so there is no instance whose calls the
    /// call could be checked against.
    NothingOutstanding,
    /// The outstanding activation's instance does not declare a call to that
    /// node type (`CREQ_RUN_REFUSES_UNDECLARED_CALL`).
    Undeclared {
        /// The instance whose activation made the call.
        instance: String,
        /// The node type called.
        node_type: String,
    },
    /// The outstanding activation is itself a call's, and a called node type has
    /// no instance of its own to declare calls, so calls go one deep
    /// (`CREQ_RUN_REFUSES_UNDECLARED_CALL`).
    CalledFromACall {
        /// The instance the calling activation is for.
        instance: String,
        /// The node type called.
        node_type: String,
    },
    /// The call does not fill the node type's parameters as it declares them,
    /// each fault named (`CREQ_RUN_REFUSES_UNFILLED_CALL`).
    Unfilled {
        /// The instance whose activation made the call.
        instance: String,
        /// The node type called.
        node_type: String,
        /// Every way the call's inputs miss the declaration, in the order found:
        /// the inputs as given, then the required parameters left unfilled.
        faults: Vec<CallFault>,
    },
    /// A context the call gives shares its identifier with a different context
    /// the run holds (`CREQ_RUN_REFUSES_SHARED_CALL_IDENTIFIER`).
    IdentifierShared {
        /// The instance whose activation made the call.
        instance: String,
        /// The first identifier found shared.
        id: ContextId,
    },
}

/// One way a call's inputs miss the declaration of the node type called.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CallFault {
    /// A parameter the node type declares neither as required nor as optional.
    UndeclaredParameter {
        /// The parameter as the call names it.
        parameter: String,
    },
    /// A parameter the call fills more than once.
    FilledTwice {
        /// The parameter.
        parameter: String,
    },
    /// A required parameter the call gives no context for.
    RequiredUnfilled {
        /// The parameter.
        parameter: String,
    },
    /// A parameter given a context of another type than it is declared for.
    WrongType {
        /// The parameter.
        parameter: String,
        /// The type it is declared for.
        declared: ContextType,
        /// The type of the context given.
        given: ContextType,
    },
}

impl CallFault {
    /// The parameter this fault concerns.
    pub fn parameter(&self) -> &str {
        match self {
            Self::UndeclaredParameter { parameter }
            | Self::FilledTwice { parameter }
            | Self::RequiredUnfilled { parameter }
            | Self::WrongType { parameter, .. } => parameter,
        }
    }
}

impl fmt::Display for CallRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NothingOutstanding => NothingOutstanding.fmt(f),
            Self::Undeclared {
                instance,
                node_type,
            } => write!(f, "{instance} does not declare a call to {node_type}"),
            Self::CalledFromACall {
                instance,
                node_type,
            } => write!(
                f,
                "a call to {node_type} was made by a called node type's activation for {instance}, and calls go one deep"
            ),
            Self::Unfilled {
                instance,
                node_type,
                faults,
            } => {
                let parameters: Vec<&str> = faults.iter().map(CallFault::parameter).collect();
                write!(
                    f,
                    "{instance}'s call to {node_type} does not fill it as declared: {}",
                    parameters.join(", ")
                )
            }
            Self::IdentifierShared { instance, id } => write!(
                f,
                "{instance}'s call gives a second context under {id:?}, which the run holds"
            ),
        }
    }
}

impl std::error::Error for CallRefusal {}

/// Why a reported exchange was not accepted. Nothing of it is held, and the
/// activation stays outstanding.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExchangeRefusal {
    /// No activation was outstanding to hold it with.
    NothingOutstanding,
    /// A context of the exchange shares its identifier with a different context
    /// the run holds (`CREQ_RUN_REFUSES_SHARED_CALL_IDENTIFIER`).
    IdentifierShared {
        /// The instance the outstanding activation is for.
        instance: String,
        /// The first identifier found shared.
        id: ContextId,
    },
}

impl fmt::Display for ExchangeRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NothingOutstanding => NothingOutstanding.fmt(f),
            Self::IdentifierShared { instance, id } => write!(
                f,
                "an exchange of {instance} holds a second context under {id:?}, which the run holds"
            ),
        }
    }
}

impl std::error::Error for ExchangeRefusal {}

/// An activation being performed, with what it has brought into the run so far.
///
/// What an activation brings in while it is performed - its exchanges, the
/// contexts its calls give - is held with it (`CREQ_RUN_HOLDS_EXCHANGES`) and
/// becomes the run's when its output is accepted. Until then its own output may
/// be one of them - a model's answer returned as it came - where a context the
/// run or another activation in progress holds may not, since that would credit
/// one context to two producers (`CREQ_RUN_REFUSES_HELD_IDENTIFIER`).
#[derive(Clone, Debug)]
struct InProgress {
    activation: Activation,
    exchanges: Vec<Exchange>,
    contexts: HashMap<ContextId, Context>,
}

impl InProgress {
    fn new(activation: Activation) -> Self {
        Self {
            activation,
            exchanges: Vec::new(),
            contexts: HashMap::new(),
        }
    }
}

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
    /// The run had made as many activations as its budget allows
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

/// A run's state as its record writes it: everything it was started with and
/// everything it has accepted since, and nothing that can be derived from those
/// (`EVD_RUN_STATE_DERIVABLE`).
pub(crate) struct Recorded<'r> {
    /// The budget the run is held to.
    pub(crate) budget: usize,
    /// The activations it has spent, one more than its outputs while one is
    /// outstanding.
    pub(crate) spent: usize,
    /// What it was started with.
    pub(crate) arguments: &'r Arguments,
    /// Each accepted activation with its output, in the order accepted.
    pub(crate) accepted: &'r [(Activation, Context)],
    /// Every context it holds, each under its identifier.
    pub(crate) held: &'r HashMap<ContextId, Context>,
}

/// One run of one workflow definition.
///
/// Borrows the definition rather than owning it: a run reads it and never
/// changes it, and cloning one would copy the whole graph per run.
pub struct Run<'a, F> {
    definition: &'a WorkflowDefinition,
    arguments: Arguments,
    produced: Produced,
    /// Every context the run holds - its arguments, the outputs it has
    /// accepted, and everything any of them was composed from - each under its
    /// identifier, which names it alone (`DEC_IDENTIFIER_NAMES_ONE_CONTEXT`).
    held: HashMap<ContextId, Context>,
    /// Every activation whose output was accepted, with that output, in the
    /// order they were accepted - what a record needs of a run's history that
    /// `produced` does not keep (`DEC_RECORD_IS_OUTPUTS`).
    accepted: Vec<(Activation, Context)>,
    /// The exchanges each accepted activation made, beside it in `accepted`.
    settled_exchanges: Vec<Vec<Exchange>>,
    /// The activation being performed, with what it has brought in so far.
    outstanding: Option<InProgress>,
    /// The activation whose call is being performed, waiting beneath it.
    suspended: Option<InProgress>,
    /// A call accepted and not yet offered, which the next step offers.
    called: Option<Activation>,
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
                &self
                    .outstanding
                    .as_ref()
                    .map(|outstanding| outstanding.activation.instance()),
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
    ///
    /// The arguments' identifiers are checked third, against each other and
    /// against everything they were composed from: they are made before the run
    /// exists, and so are the first place a second identifier source enters.
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

        let held = held_arguments(&arguments).map_err(StartRefusal::SharedIdentifiers)?;

        Ok(Self {
            definition,
            arguments,
            produced: Produced::new(),
            held,
            accepted: Vec::new(),
            settled_exchanges: Vec::new(),
            outstanding: None,
            suspended: None,
            called: None,
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
    ///
    /// A call accepted since the last step is offered before any instance the
    /// scheduler would choose, and counted as every activation is: the model
    /// that made it is waiting on it (`DEC_CALL_IS_AN_ACTIVATION`). With the
    /// budget spent it is not offered, and the run ends as over budget.
    // @Completion then the budget then a call then quiescence,IMPL_RUN_STEP,impl,[CREQ_RUN_COMPLETES, CREQ_RUN_STOPS_AT_BUDGET, CREQ_RUN_ENDS_QUIESCENT, CREQ_RUN_CALL_OFFERED]
    pub fn step(&mut self) -> Step<F> {
        if let Some(result) = self.result() {
            return Step::Ended(RunEnding::Completed(result));
        }

        if let Some(outstanding) = &self.outstanding {
            return Step::Activate(outstanding.activation.clone());
        }

        if self.activations >= self.budget {
            return Step::Ended(RunEnding::BudgetExceeded {
                budget: self.budget,
            });
        }

        if let Some(called) = self.called.take() {
            self.outstanding = Some(InProgress::new(called.clone()));
            self.activations += 1;
            return Step::Activate(called);
        }

        match next_activation(self.definition, &self.arguments, &self.produced) {
            Some(activation) => {
                self.outstanding = Some(InProgress::new(activation.clone()));
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
            instance: outstanding.activation.instance().to_owned(),
            failure,
        })
    }

    /// Report what the outstanding activation produced.
    ///
    /// Refused, and nothing recorded, when the output is not of the type the
    /// activation's node type declares, carries an identifier the run already
    /// holds, or was composed from a second context under an identifier the run
    /// or the output holds for another. Each way the activation stays
    /// outstanding and is not counted again (`DEC_REFUSED_OUTPUT_OUTSTANDING`): asking for the next
    /// step hands it back, so a refusal cannot be stepped past, and the caller
    /// answers again or reports the activation failed.
    ///
    /// An output may be a context its own activation brought in - a model's
    /// answer returned as it came - and not one another activation in progress
    /// holds: a called node type handing back the context it was called with
    /// would credit it to two producers.
    ///
    /// Accepted for a call's activation, the output is the called node type's
    /// and no instance's: the calling activation is outstanding again, and the
    /// output goes to it alone, filed under no instance for a binding to read or
    /// for the run to complete on (`CREQ_RUN_CALL_OUTPUT_TO_CALLER`).
    // @An output is checked before it is recorded,IMPL_RUN_PRODUCED,impl,[CREQ_RUN_REFUSES_UNDECLARED_OUTPUT, CREQ_RUN_REFUSES_HELD_IDENTIFIER, CREQ_RUN_REFUSES_SHARED_OUTPUT_IDENTIFIER, CREQ_RUN_REFUSED_OUTPUT_OUTSTANDING, CREQ_RUN_CALL_OUTPUT_TO_CALLER]
    pub fn produced(&mut self, context: Context) -> Result<(), OutputRefusal> {
        let outstanding = self
            .outstanding
            .as_ref()
            .ok_or(OutputRefusal::NothingOutstanding)?;
        let activation = &outstanding.activation;

        if context.declared_type() != activation.output() {
            return Err(OutputRefusal::UndeclaredType {
                instance: activation.instance().to_owned(),
                declared: activation.output().clone(),
                reported: context.declared_type().clone(),
            });
        }

        let held_by_the_caller = self
            .suspended
            .as_ref()
            .is_some_and(|caller| caller.contexts.contains_key(&context.id()));
        if self.holds(context.id()) || held_by_the_caller {
            return Err(OutputRefusal::IdentifierHeld {
                instance: activation.instance().to_owned(),
                id: context.id(),
            });
        }

        let (brought, shared) = brought_in(&self.known(), &context);
        if let Some(&id) = shared.first() {
            return Err(OutputRefusal::IdentifierShared {
                instance: activation.instance().to_owned(),
                id,
            });
        }

        let Some(done) = self.outstanding.take() else {
            unreachable!("an outstanding activation was just read");
        };
        self.held.extend(done.contexts);
        self.held.extend(brought);
        if done.activation.call().is_none() {
            self.produced
                .insert(done.activation.instance().to_owned(), context.clone());
        }
        self.accepted.push((done.activation, context));
        self.settled_exchanges.push(done.exchanges);
        self.outstanding = self.suspended.take();
        Ok(())
    }

    /// Report a call the outstanding activation's model made, to be performed as
    /// an activation of the node type called (`DEC_CALL_IS_AN_ACTIVATION`).
    ///
    /// Accepted, the calling activation waits beneath the call, and the next
    /// step offers the node type's activation for the calling instance, given
    /// the call's contexts in the order the node type declares its parameters;
    /// its output comes back through [`Run::produced`], after which the calling
    /// activation is outstanding again.
    ///
    /// Refused, with nothing offered or spent and the activation outstanding as
    /// it was, when nothing is outstanding, when the outstanding activation is
    /// itself a call's, when its instance does not declare a call to that node
    /// type, when the call does not fill the node type's parameters as declared,
    /// naming every such parameter, or when a context it gives is a second one
    /// under an identifier the run holds. The checks are the performer's too
    /// (`CREQ_HOST_REFUSES_MALFORMED_CALL`); the run makes them whoever reports.
    // @A call checked and offered as the next activation,IMPL_RUN_CALL,impl,[CREQ_RUN_CALL_OFFERED, CREQ_RUN_REFUSES_UNDECLARED_CALL, CREQ_RUN_REFUSES_UNFILLED_CALL, CREQ_RUN_REFUSES_SHARED_CALL_IDENTIFIER]
    pub fn call(&mut self, call: Call) -> Result<(), CallRefusal> {
        let Some(outstanding) = &self.outstanding else {
            return Err(CallRefusal::NothingOutstanding);
        };
        let instance = outstanding.activation.instance().to_owned();
        let node_type = call.node_type.clone();

        if outstanding.activation.call().is_some() {
            return Err(CallRefusal::CalledFromACall {
                instance,
                node_type,
            });
        }

        let declares = self
            .definition
            .instances
            .iter()
            .find(|node| node.name == instance)
            .is_some_and(|node| node.calls.contains(&node_type));
        let declared = self
            .definition
            .node_types
            .iter()
            .find(|declared| declared.name == node_type);
        let (true, Some(declared)) = (declares, declared) else {
            return Err(CallRefusal::Undeclared {
                instance,
                node_type,
            });
        };

        let faults = call_faults(declared, &call);
        if !faults.is_empty() {
            return Err(CallRefusal::Unfilled {
                instance,
                node_type,
                faults,
            });
        }

        let mut brought_all: HashMap<ContextId, Context> = HashMap::new();
        for (_, given) in &call.inputs {
            let mut known = self.known();
            known.push(&brought_all);
            let (brought, shared) = brought_in(&known, given);
            if let Some(&id) = shared.first() {
                return Err(CallRefusal::IdentifierShared { instance, id });
            }
            brought_all.extend(brought);
        }

        let listed = declared.required.iter().chain(&declared.optional);
        let inputs = listed
            .filter_map(|parameter| {
                call.inputs
                    .iter()
                    .find(|(given, _)| *given == parameter.name)
                    .cloned()
            })
            .collect();
        let activation = Activation {
            instance,
            node_type,
            call: Some(call.id),
            inputs,
            output: declared.output.clone(),
        };

        let Some(mut caller) = self.outstanding.take() else {
            unreachable!("an outstanding activation was just read");
        };
        caller.contexts.extend(brought_all);
        self.suspended = Some(caller);
        self.called = Some(activation);
        Ok(())
    }

    /// Report an exchange the outstanding activation made, to be held with it
    /// (`CREQ_RUN_HOLDS_EXCHANGES`) and, once its output is accepted, with the
    /// run.
    ///
    /// Refused, holding nothing of it, when nothing is outstanding or when a
    /// context of it is a second one under an identifier the run holds. A
    /// context held already - an earlier window a later one composes - is the
    /// very one held, and brings nothing in.
    // @An exchange held with its activation,IMPL_RUN_EXCHANGE,impl,[CREQ_RUN_HOLDS_EXCHANGES, CREQ_RUN_REFUSES_SHARED_CALL_IDENTIFIER]
    pub fn exchange(&mut self, exchange: Exchange) -> Result<(), ExchangeRefusal> {
        let Some(outstanding) = &self.outstanding else {
            return Err(ExchangeRefusal::NothingOutstanding);
        };
        let instance = outstanding.activation.instance().to_owned();

        let mut brought_all: HashMap<ContextId, Context> = HashMap::new();
        for context in exchange.contexts() {
            let mut known = self.known();
            known.push(&brought_all);
            let (brought, shared) = brought_in(&known, context);
            if let Some(&id) = shared.first() {
                return Err(ExchangeRefusal::IdentifierShared { instance, id });
            }
            brought_all.extend(brought);
        }

        let Some(outstanding) = self.outstanding.as_mut() else {
            unreachable!("an outstanding activation was just read");
        };
        outstanding.contexts.extend(brought_all);
        outstanding.exchanges.push(exchange);
        Ok(())
    }

    /// The exchanges the outstanding activation has made, in the order they
    /// were reported, or none when nothing is outstanding.
    pub fn exchanges(&self) -> &[Exchange] {
        self.outstanding
            .as_ref()
            .map_or(&[], |outstanding| &outstanding.exchanges)
    }

    /// Every set of contexts an identifier is checked against: the run's own,
    /// and those of each activation in progress.
    fn known(&self) -> Vec<&HashMap<ContextId, Context>> {
        let mut known = vec![&self.held];
        known.extend(self.outstanding.as_ref().map(|now| &now.contexts));
        known.extend(self.suspended.as_ref().map(|caller| &caller.contexts));
        known
    }

    /// What a record of this run holds, read without changing it
    /// (`CREQ_RECORD_HOLDS_THE_RUN`).
    pub(crate) fn recorded(&self) -> Recorded<'_> {
        Recorded {
            budget: self.budget,
            spent: self.activations,
            arguments: &self.arguments,
            accepted: &self.accepted,
            held: &self.held,
        }
    }

    /// Whether the run holds a context under `id`: an argument, an accepted
    /// output, or anything either was composed from.
    ///
    /// Asked of the whole run rather than of the outstanding instance's inputs:
    /// a caller holding any context of the run can hand it back, including the
    /// output of an instance not wired to this one, or a part of its own input.
    /// A composition holding a held context by reference has an identifier of
    /// its own, and is the one sanctioned way to pass an input on.
    // @Everything a run holds,IMPL_RUN_HOLDS,impl,[CREQ_RUN_REFUSES_HELD_IDENTIFIER]
    fn holds(&self, id: ContextId) -> bool {
        self.held.contains_key(&id)
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

    // A part the argument was composed from is held too, though it is neither
    // an argument nor an output: handed back, it would be credited to `e`.
    let inner = ctx(&mut source, "note");
    let argument = Context::compose(&mut source, context_type("note"), [&inner], "")
        .expect("a fresh source issues");
    let arguments = Arguments::new().supply("e", "seed", argument);
    let mut run = Run::<Infallible>::start(&workflow, arguments, 10).expect("sound");
    offered(&mut run);
    assert_eq!(
        run.produced(inner.clone()),
        Err(OutputRefusal::IdentifierHeld {
            instance: "e".to_owned(),
            id: inner.id(),
        })
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

/// Four entry instances of one type, each taking one `note`, all feeding a
/// join that is designated.
#[cfg(test)]
fn four_entries() -> WorkflowDefinition {
    let types = vec![
        node_type("Entry", &[("seed", "note")], "note"),
        node_type(
            "Join",
            &[("a", "note"), ("b", "note"), ("c", "note"), ("d", "note")],
            "note",
        ),
    ];
    let instances = vec![
        instance("pa", "Entry", &[]).into_entry(),
        instance("pb", "Entry", &[]).into_entry(),
        instance("pc", "Entry", &[]).into_entry(),
        instance("pd", "Entry", &[]).into_entry(),
        instance(
            "j",
            "Join",
            &[("a", "pa"), ("b", "pb"), ("c", "pc"), ("d", "pd")],
        ),
    ];
    definition(types, instances, &["j"])
}

#[cfg(test)]
#[test]
fn arguments_sharing_an_identifier_are_refused() {
    let workflow = four_entries();
    let (mut one, mut two) = (IdSource::new(), IdSource::new());

    // `pa` and `pb`: two sources, one identifier - the measured shape.
    let first = ctx(&mut one, "note");
    let second = ctx(&mut two, "note");
    assert_eq!(first.id(), second.id());
    // `pc` and `pd`: `pd` is new under its own identifier, and was composed
    // from a context repeating `pc`'s.
    let third = ctx(&mut one, "note");
    let repeat = ctx(&mut two, "note");
    assert_eq!(third.id(), repeat.id());
    let fourth = Context::compose(&mut two, context_type("note"), [&repeat], "")
        .expect("a fresh source issues");
    assert_ne!(fourth.id(), third.id());

    let arguments = Arguments::new()
        .supply("pa", "seed", first.clone())
        .supply("pb", "seed", second)
        .supply("pc", "seed", third.clone())
        .supply("pd", "seed", fourth);
    let refusal =
        Run::<Infallible>::start(&workflow, arguments, 10).expect_err("two contexts, one id");
    // Both identifiers, each once, as a refusal of its own kind.
    assert_eq!(
        refusal,
        StartRefusal::SharedIdentifiers(vec![first.id(), third.id()])
    );
}

#[cfg(test)]
#[test]
fn one_context_for_two_parameters_starts() {
    let mut source = IdSource::new();
    let workflow = four_entries();
    let shared = ctx(&mut source, "note");
    let composed = Context::compose(&mut source, context_type("note"), [&shared], "")
        .expect("a fresh source issues");

    // One context reached three times, and a composition of it: one identifier
    // per context, however often each is reached.
    let arguments = Arguments::new()
        .supply("pa", "seed", shared.clone())
        .supply("pb", "seed", shared.clone())
        .supply("pc", "seed", composed)
        .supply("pd", "seed", shared);
    let (ending, _) = drive(&workflow, arguments, 10, &mut source);
    assert!(matches!(ending, RunEnding::Completed(_)), "{ending:?}");
}

#[cfg(test)]
#[test]
fn part_sharing_an_identifier_is_refused() {
    let workflow = entry_chain(3);
    let (mut own, mut other) = (IdSource::new(), IdSource::new());
    let argument = ctx(&mut own, "note");
    let arguments = Arguments::new().supply("n0", "seed", argument.clone());
    let mut run = Run::<Infallible>::start(&workflow, arguments, 10).expect("sound");

    // The measured shape: a new part under the argument's identifier.
    let activation = offered(&mut run);
    let part = ctx(&mut other, "note");
    assert_eq!(part.id(), argument.id());
    let output = Context::compose(
        &mut other,
        context_type("note"),
        [&activation.inputs()[0].1, &part],
        "",
    )
    .expect("a fresh source issues");
    assert_eq!(
        run.produced(output),
        Err(OutputRefusal::IdentifierShared {
            instance: "n0".to_owned(),
            id: argument.id(),
        })
    );

    // Answered properly, `n0`'s output is composed of the argument and a part of
    // its own, both from the run's source, and accepted.
    let own_part = ctx(&mut own, "note");
    let accepted = Context::compose(
        &mut own,
        context_type("note"),
        [&activation.inputs()[0].1, &own_part],
        "",
    )
    .expect("a fresh source issues");
    run.produced(accepted).expect("one source, nothing shared");

    // A later output repeating the identifier of that earlier output's part -
    // which is neither an argument nor an output - is refused too, so the run
    // remembers what an accepted output brought in.
    offered(&mut run);
    let mut late = IdSource::new();
    let mut repeat = ctx(&mut late, "note");
    while repeat.id() != own_part.id() {
        repeat = ctx(&mut late, "note");
    }
    let output = Context::compose(&mut own, context_type("note"), [&repeat], "")
        .expect("a fresh source issues");
    assert_eq!(
        run.produced(output),
        Err(OutputRefusal::IdentifierShared {
            instance: "n1".to_owned(),
            id: own_part.id(),
        })
    );
}

#[cfg(test)]
#[test]
fn parts_sharing_an_identifier_are_refused() {
    let workflow = chain(1);
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");
    offered(&mut run);

    // Two new contexts, from two sources, under an identifier the run has never
    // held: they collide with each other and with nothing else.
    let (mut one, mut two) = (IdSource::new(), IdSource::new());
    let left = ctx(&mut one, "note");
    let right = ctx(&mut two, "note");
    assert_eq!(left.id(), right.id());
    let output = Context::compose(&mut one, context_type("note"), [&left, &right], "")
        .expect("a fresh source issues");
    assert_eq!(
        run.produced(output),
        Err(OutputRefusal::IdentifierShared {
            instance: "n0".to_owned(),
            id: left.id(),
        })
    );
}

#[cfg(test)]
#[test]
fn output_composing_held_parts_is_accepted() {
    let mut source = IdSource::new();
    let workflow = entry_chain(1);
    let inner = ctx(&mut source, "note");
    let argument = Context::compose(&mut source, context_type("note"), [&inner], "")
        .expect("a fresh source issues");
    let arguments = Arguments::new().supply("n0", "seed", argument);
    let mut run = Run::<Infallible>::start(&workflow, arguments, 10).expect("sound");

    // The input, a part the input was composed from, and a new context: every
    // held context in it is the context the run holds.
    let activation = offered(&mut run);
    let input = &activation.inputs()[0].1;
    let fresh = ctx(&mut source, "note");
    let output = Context::compose(
        &mut source,
        context_type("note"),
        [input, &input.parts()[0], &fresh],
        "",
    )
    .expect("a fresh source issues");
    run.produced(output)
        .expect("held contexts reached by reference are the held contexts");
    assert!(matches!(run.step(), Step::Ended(RunEnding::Completed(_))));
}

/// Every context reachable from `roots`, themselves included, grouped by
/// identifier - so that a test can ask whether any identifier names two.
#[cfg(test)]
fn by_identifier(roots: &[Context]) -> HashMap<ContextId, Vec<Context>> {
    let mut found: HashMap<ContextId, Vec<Context>> = HashMap::new();
    let mut pending: Vec<Context> = roots.to_vec();
    while let Some(context) = pending.pop() {
        let under = found.entry(context.id()).or_default();
        if under.iter().any(|known| known.is(&context)) {
            continue;
        }
        under.push(context.clone());
        pending.extend(context.parts().iter().cloned());
    }
    found
}

#[cfg(test)]
proptest! {
    /// However a caller mixes two identifier sources, composing freely of what
    /// the run holds and of new contexts, no identifier names two different
    /// contexts among everything the run accepted or started with.
    #[test]
    fn no_identifier_names_two_contexts(
        answers in prop::collection::vec(
            (any::<bool>(), prop::collection::vec(any::<prop::sample::Index>(), 0..3), any::<bool>()),
            1..7,
        ),
    ) {
        let workflow = entry_chain(answers.len());
        let mut sources = [IdSource::new(), IdSource::new()];
        let argument = ctx(&mut sources[0], "note");
        let mut accepted = vec![argument.clone()];
        let arguments = Arguments::new().supply("n0", "seed", argument);
        let mut run = Run::<Infallible>::start(&workflow, arguments, answers.len())
            .expect("sound");

        for (use_second, picks, add_new) in &answers {
            offered(&mut run);
            // Anything the run holds, parts included, may be picked.
            let holdable: Vec<Context> = by_identifier(&accepted)
                .into_values()
                .flatten()
                .collect();
            let mut parts: Vec<Context> = picks.iter().map(|pick| pick.get(&holdable).clone()).collect();
            let source = &mut sources[usize::from(*use_second)];
            if *add_new {
                parts.push(ctx(source, "note"));
            }
            let output = Context::compose(source, context_type("note"), &parts, "")
                .expect("a fresh source issues");
            if run.produced(output.clone()).is_ok() {
                accepted.push(output);
            } else {
                // Refused: answer from the run's own source instead, composed of
                // nothing it could collide with.
                let fresh = ctx(&mut sources[0], "note");
                let safe = Context::compose(&mut sources[0], context_type("note"), [&fresh], "")
                    .expect("a fresh source issues");
                if run.produced(safe.clone()).is_ok() {
                    accepted.push(safe);
                } else {
                    // Even that can repeat what the second source issued; the
                    // run stops here, and what it accepted is still checked.
                    break;
                }
            }
        }

        for (id, contexts) in by_identifier(&accepted) {
            prop_assert_eq!(contexts.len(), 1, "{:?} names {} contexts", id, contexts.len());
        }
    }
}

// --- calls and exchanges -------------------------------------------------------

/// A workflow whose `asker` may call `lookup` - two required parameters and an
/// optional one - beside `second`, which is ready at the same time, and `after`,
/// which consumes the asker's output. `designated` names the result.
#[cfg(test)]
fn calling(designated: &str) -> WorkflowDefinition {
    let types = vec![
        node_type("ask", &[], "note"),
        node_type("lookup", &[("query", "note"), ("scope", "note")], "note")
            .with_optional(&[("hint", "note")]),
        node_type("other", &[], "note"),
        node_type("pass", &[("input", "note")], "note"),
    ];
    let instances = vec![
        instance("asker", "ask", &[]).with_calls(&["lookup"]),
        instance("second", "other", &[]),
        instance("after", "pass", &[("input", "asker")]),
    ];
    definition(types, instances, &[designated])
}

/// A call to `lookup` filling both required parameters, `scope` first.
#[cfg(test)]
fn lookup_call(source: &mut IdSource, id: &str) -> (Call, Context, Context) {
    let query = ctx(source, "note");
    let scope = ctx(source, "note");
    let call = Call::new(id, "lookup")
        .input("scope", scope.clone())
        .input("query", query.clone());
    (call, query, scope)
}

#[cfg(test)]
#[test]
fn declared_call_is_offered_next() {
    let mut source = IdSource::new();
    let workflow = calling("second");
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");

    let asking = offered(&mut run);
    assert_eq!((asking.instance(), asking.call()), ("asker", None));
    let window = ctx(&mut source, "note");
    run.exchange(Exchange::new(window.clone(), ctx(&mut source, "note")).calling("call_7"))
        .expect("the exchange that made the call");
    let (call, query, scope) = lookup_call(&mut source, "call_7");
    run.call(call).expect("a declared call, filled");

    // Offered next, before `second`, for the asker, with its inputs in the order
    // lookup declares them rather than the order the call gave them - and
    // counted.
    let called = offered(&mut run);
    assert_eq!(
        (called.instance(), called.node_type(), called.call()),
        ("asker", "lookup", Some("call_7"))
    );
    let inputs: Vec<(&str, ContextId)> = called
        .inputs()
        .iter()
        .map(|(parameter, given)| (parameter.as_str(), given.id()))
        .collect();
    assert_eq!(inputs, [("query", query.id()), ("scope", scope.id())]);
    assert_eq!(called.output().as_str(), "note");
    assert_eq!(
        run.recorded().spent,
        2,
        "the call is an activation, counted"
    );

    // The calling activation survives its call: once the called output is
    // accepted, the same activation is outstanding again - not offered anew,
    // which would spend the budget twice and lose what it had exchanged - and
    // its own output is its own.
    run.produced(ctx(&mut source, "note"))
        .expect("lookup's output");
    let again = offered(&mut run);
    assert_eq!((again.instance(), again.call()), ("asker", None));
    assert_eq!(run.recorded().spent, 2, "the asker is not offered anew");
    let kept: Vec<ContextId> = run.exchanges().iter().map(|e| e.window().id()).collect();
    assert_eq!(kept, [window.id()], "its exchange is still held with it");
    run.produced(ctx(&mut source, "note"))
        .expect("the asker's own output");
    assert_eq!(offered(&mut run).instance(), "second");
}

#[cfg(test)]
#[test]
fn call_output_goes_back_to_the_caller() {
    let mut source = IdSource::new();
    let workflow = calling("asker");
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");

    offered(&mut run);
    let (call, query, _) = lookup_call(&mut source, "call_1");
    run.call(call).expect("declared");
    offered(&mut run);

    // A called node type handing back what it was called with would credit the
    // caller's context to it as well.
    assert_eq!(
        run.produced(query.clone()),
        Err(OutputRefusal::IdentifierHeld {
            instance: "asker".to_owned(),
            id: query.id(),
        })
    );
    let found = ctx(&mut source, "note");
    run.produced(found.clone()).expect("lookup's output");

    // Not the asker's output: the run has not completed on it, though the asker
    // is designated, and its consumer is not offered. The asker is.
    let again = offered(&mut run);
    assert_eq!((again.instance(), again.call()), ("asker", None));

    // Handed back as the asker's own, it would be credited to two producers.
    assert_eq!(
        run.produced(found.clone()),
        Err(OutputRefusal::IdentifierHeld {
            instance: "asker".to_owned(),
            id: found.id(),
        })
    );

    // Held by the run: a composition holding it is accepted, and is the result.
    let answer = Context::compose(&mut source, context_type("note"), [&found], "")
        .expect("a fresh source issues");
    run.produced(answer.clone())
        .expect("composing the called output");
    match run.step() {
        Step::Ended(RunEnding::Completed(result)) => assert!(result.is(&answer)),
        other => panic!("the asker's own output completes the run: {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn undeclared_call_is_refused() {
    let mut source = IdSource::new();
    let workflow = calling("second");
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");

    // Nothing outstanding yet.
    let (call, ..) = lookup_call(&mut source, "early");
    assert_eq!(run.call(call), Err(CallRefusal::NothingOutstanding));

    offered(&mut run);
    // A node type the catalogue holds and the asker does not declare.
    assert_eq!(
        run.call(Call::new("call_1", "other")),
        Err(CallRefusal::Undeclared {
            instance: "asker".to_owned(),
            node_type: "other".to_owned(),
        })
    );
    assert_eq!(run.recorded().spent, 1, "a refused call spends nothing");
    assert_eq!(offered(&mut run).call(), None, "the asker is outstanding");

    // A call made during a called node type's activation, to what its caller
    // declares: calls go one deep.
    let (call, ..) = lookup_call(&mut source, "call_2");
    run.call(call).expect("declared");
    offered(&mut run);
    let (nested, ..) = lookup_call(&mut source, "call_3");
    assert_eq!(
        run.call(nested),
        Err(CallRefusal::CalledFromACall {
            instance: "asker".to_owned(),
            node_type: "lookup".to_owned(),
        })
    );
    assert_eq!(run.recorded().spent, 2);
    assert_eq!(
        offered(&mut run).call(),
        Some("call_2"),
        "the call is outstanding"
    );
}

#[cfg(test)]
#[test]
fn unfilled_call_is_refused() {
    let mut source = IdSource::new();
    let workflow = calling("second");
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");
    offered(&mut run);

    // A parameter lookup does not declare, one of the wrong type, and a
    // required one left without a context - all three in one refusal.
    let unfilled = Call::new("call_1", "lookup")
        .input("extra", ctx(&mut source, "note"))
        .input("scope", ctx(&mut source, "draft"));
    assert_eq!(
        run.call(unfilled),
        Err(CallRefusal::Unfilled {
            instance: "asker".to_owned(),
            node_type: "lookup".to_owned(),
            faults: vec![
                CallFault::UndeclaredParameter {
                    parameter: "extra".to_owned()
                },
                CallFault::WrongType {
                    parameter: "scope".to_owned(),
                    declared: context_type("note"),
                    given: context_type("draft"),
                },
                CallFault::RequiredUnfilled {
                    parameter: "query".to_owned()
                },
            ],
        })
    );
    assert_eq!(run.recorded().spent, 1, "nothing offered or spent");
    assert_eq!(offered(&mut run).call(), None, "the asker is outstanding");

    // Leaving the optional parameter unfilled is a call as declared.
    let (call, ..) = lookup_call(&mut source, "call_2");
    run.call(call)
        .expect("an optional parameter may go unfilled");
}

#[cfg(test)]
#[test]
fn call_sharing_an_identifier_is_refused() {
    let mut source = IdSource::new();
    // A second source issues the identifiers the first did: a context from it
    // is a different context under an identifier the run holds.
    let mut second_source = IdSource::new();
    let workflow = calling("second");
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");
    offered(&mut run);

    let window = ctx(&mut source, "note");
    let answer = ctx(&mut source, "note");
    run.exchange(Exchange::new(window.clone(), answer.clone()))
        .expect("a first exchange");

    let impostor = ctx(&mut second_source, "note");
    assert_eq!(
        impostor.id(),
        window.id(),
        "the stimulus: one identifier, two contexts"
    );
    let call = Call::new("call_1", "lookup")
        .input("query", impostor.clone())
        .input("scope", ctx(&mut source, "note"));
    assert_eq!(
        run.call(call),
        Err(CallRefusal::IdentifierShared {
            instance: "asker".to_owned(),
            id: window.id(),
        })
    );
    assert_eq!(
        run.exchange(Exchange::new(ctx(&mut source, "note"), impostor)),
        Err(ExchangeRefusal::IdentifierShared {
            instance: "asker".to_owned(),
            id: window.id(),
        })
    );
    assert_eq!(run.exchanges().len(), 1, "nothing of either is held");
    assert_eq!(offered(&mut run).call(), None, "the asker is outstanding");

    // A window composing the one before brings its parts again by reference,
    // each the very context held, and is accepted.
    let next = Context::compose(&mut source, context_type("note"), [&window, &answer], "")
        .expect("a fresh source issues");
    run.exchange(Exchange::new(next, ctx(&mut source, "note")))
        .expect("the earlier window's parts are the ones held");
    assert_eq!(run.exchanges().len(), 2);
}

#[cfg(test)]
#[test]
fn exchanges_held_with_their_activation() {
    let mut source = IdSource::new();
    let types = vec![node_type("ask", &[], "note")];
    let workflow = definition(
        types,
        vec![instance("a", "ask", &[]), instance("b", "ask", &[])],
        &["b"],
    );
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");

    assert_eq!(offered(&mut run).instance(), "a");
    let first = ctx(&mut source, "note");
    let second = ctx(&mut source, "note");
    run.exchange(Exchange::new(ctx(&mut source, "note"), first.clone()).calling("call_1"))
        .expect("an exchange that made a call");
    run.exchange(Exchange::new(ctx(&mut source, "note"), second.clone()))
        .expect("one that made none");
    let answers: Vec<ContextId> = run
        .exchanges()
        .iter()
        .map(|exchange| exchange.answer().id())
        .collect();
    assert_eq!(answers, [first.id(), second.id()], "in the order reported");
    assert_eq!(run.exchanges()[0].calls(), ["call_1"]);

    // The asker's answer returned as its output is its own to return.
    run.produced(second.clone())
        .expect("an answer of its own activation");

    assert_eq!(offered(&mut run).instance(), "b");
    assert!(
        run.exchanges().is_empty(),
        "b's are b's, and it has none yet"
    );
    let third = ctx(&mut source, "note");
    run.exchange(Exchange::new(ctx(&mut source, "note"), third.clone()))
        .expect("b's exchange");
    run.produced(ctx(&mut source, "note")).expect("b's output");

    let settled: Vec<Vec<ContextId>> = run
        .settled_exchanges
        .iter()
        .map(|exchanges| exchanges.iter().map(|e| e.answer().id()).collect())
        .collect();
    assert_eq!(settled, [vec![first.id(), second.id()], vec![third.id()]]);

    // The run has completed; nothing is outstanding to hold another.
    assert!(matches!(run.step(), Step::Ended(RunEnding::Completed(_))));
    assert_eq!(
        run.exchange(Exchange::new(
            ctx(&mut source, "note"),
            ctx(&mut source, "note")
        )),
        Err(ExchangeRefusal::NothingOutstanding)
    );
}

#[cfg(test)]
#[test]
fn called_output_of_undeclared_type_is_refused() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("ask", &[], "draft"),
        node_type("lookup", &[], "note"),
    ];
    let workflow = definition(
        types,
        vec![instance("asker", "ask", &[]).with_calls(&["lookup"])],
        &["asker"],
    );
    let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), 10).expect("sound");
    offered(&mut run);
    run.call(Call::new("call_1", "lookup")).expect("declared");
    offered(&mut run);

    // Checked against lookup's declaration, not the asker's.
    assert_eq!(
        run.produced(ctx(&mut source, "draft")),
        Err(OutputRefusal::UndeclaredType {
            instance: "asker".to_owned(),
            declared: context_type("note"),
            reported: context_type("draft"),
        })
    );
    assert_eq!(
        offered(&mut run).call(),
        Some("call_1"),
        "still outstanding"
    );
    run.produced(ctx(&mut source, "note"))
        .expect("of lookup's declared type");
}

#[cfg(test)]
proptest! {
    /// However many calls the instances of a run make, the activations the run
    /// offers - the called ones counted - never exceed its budget, and a call
    /// reported with the budget spent ends the run as over budget without its
    /// activation being offered.
    ///
    /// Each instance's calls are to one node type, so a node type called twice
    /// by one instance is two activations: counted per instance, a run of two
    /// instances each calling twice would count two where it spent six.
    #[test]
    fn activations_with_calls_never_exceed_budget(
        budget in 0..10usize,
        calls in proptest::collection::vec(0..4usize, 1..=3),
    ) {
        let mut source = IdSource::new();
        let types = vec![node_type("ask", &[], "note"), node_type("lookup", &[], "note")];
        let names: Vec<String> = (0..calls.len()).map(|n| format!("n{n}")).collect();
        let instances = names
            .iter()
            .map(|name| instance(name, "ask", &[]).with_calls(&["lookup"]))
            .collect();
        let last = names.last().cloned().unwrap_or_default();
        let workflow = definition(types, instances, &[&last]);

        let mut run = Run::<Infallible>::start(&workflow, Arguments::new(), budget).expect("sound");
        let mut offers = 0;
        let mut seen = HashSet::new();
        let mut made: HashMap<String, usize> = HashMap::new();
        let ending = loop {
            match run.step() {
                Step::Ended(ending) => break ending,
                Step::Activate(activation) => {
                    let key = (activation.instance().to_owned(), activation.call().map(str::to_owned));
                    if seen.insert(key) {
                        offers += 1;
                    }
                    prop_assert!(offers <= budget, "{} offered on a budget of {}", offers, budget);
                    let index: usize = activation.instance()[1..].parse().expect("n<index>");
                    let so_far = made.entry(activation.instance().to_owned()).or_default();
                    if activation.call().is_none() && *so_far < calls[index] {
                        *so_far += 1;
                        let id = format!("{}-{}", activation.instance(), so_far);
                        run.call(Call::new(&id, "lookup")).expect("declared");
                        if offers == budget {
                            // The budget is spent: the call's activation is not
                            // offered, and the run ends as over budget.
                            let over = matches!(
                                run.step(),
                                Step::Ended(RunEnding::BudgetExceeded { .. })
                            );
                            prop_assert!(over, "a call with the budget spent ends the run");
                        }
                        continue;
                    }
                    run.produced(ctx(&mut source, "note")).expect("an activation was offered");
                }
            }
        };

        let wanted: usize = calls.iter().map(|made| made + 1).sum();
        if wanted <= budget {
            prop_assert!(matches!(ending, RunEnding::Completed(_)), "{:?}", ending);
            prop_assert_eq!(offers, wanted);
        } else {
            prop_assert!(matches!(ending, RunEnding::BudgetExceeded { .. }), "{:?}", ending);
            prop_assert_eq!(offers, budget);
        }
    }
}
