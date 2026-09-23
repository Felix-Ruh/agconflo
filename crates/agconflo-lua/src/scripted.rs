//! A run driven from its start to its ending by scripts: each activation the run
//! offers is performed by the script host, and what came of it is reported back.
//!
//! Not a component of its own. `ARCH_BEHAVIOUR` gives the reason, the one
//! `ARCH_RUN` gave for the activation it dropped: nothing about driving a run is
//! true or false taken alone, and every requirement it could carry is already
//! the run's, the behaviour set's or the host's.

use std::fmt;

use agconflo_core::{
    Arguments, IdSource, NothingOutstanding, Run, RunEnding, StartRefusal, Step, WorkflowDefinition,
};

use crate::behaviours::{BehaviourFault, Behaviours};
use crate::host::{self, Limits, ScriptFailure};

/// Why a scripted run was not started.
///
/// Two classes, asked in order: whatever the run itself refuses a workflow or
/// its arguments for, and then the scripts. The scripts are asked about only
/// once the run would start, because a workflow with a wiring defect may name a
/// node type that does not exist, and whether it has a script is not a question
/// worth answering first.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScriptedRefusal {
    /// The run refused to start: the workflow's wiring or its arguments.
    Start(StartRefusal),
    /// The scripts cannot run, every fault of them
    /// (`CREQ_BEHAVIOURS_EVERY_FAULT`).
    Behaviours(Vec<BehaviourFault>),
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
        }
    }
}

impl std::error::Error for ScriptedRefusal {}

/// Run `definition` to its ending, performing each activation with the script
/// `behaviours` gives its node type - or refuse to start it.
///
/// Nothing is performed before every refusal has been asked: the run's own,
/// then the scripts' (`FEAT_BEHAVIOUR_REFUSED_BEFORE_START`). After that the run
/// decides what activates and how it ends, exactly as for any caller, and a
/// script's failure is carried in the run's ending as a [`ScriptFailure`].
///
/// `source` issues the identifiers of every context the scripts make, and must
/// be the one `arguments` were made from: two sources repeat each other's
/// identifiers, and the run refuses an output carrying one it already holds.
// @Refused before anything runs,IMPL_SCRIPTED_REFUSAL,impl,[CREQ_BEHAVIOURS_REFUSE_MISSING, CREQ_BEHAVIOURS_REFUSE_UNCOMPILABLE, CREQ_BEHAVIOURS_REFUSE_TWICE]
pub fn run_scripted(
    definition: &WorkflowDefinition,
    behaviours: &Behaviours,
    arguments: Arguments,
    source: &mut IdSource,
    budget: usize,
    limits: Limits,
) -> Result<RunEnding<ScriptFailure>, ScriptedRefusal> {
    let mut run = Run::start(definition, arguments, budget).map_err(ScriptedRefusal::Start)?;

    let faults = behaviours.faults(definition);
    if !faults.is_empty() {
        return Err(ScriptedRefusal::Behaviours(faults));
    }

    loop {
        let activation = match run.step() {
            Step::Ended(ending) => return Ok(ending),
            Step::Activate(activation) => activation,
        };

        // Both lookups succeed: the run offers only instances of a sound
        // definition, whose names are unique and whose types are declared, and
        // the behaviour set has refused the run unless each type it instantiates
        // has exactly one script.
        let node_type = definition
            .instances
            .iter()
            .find(|instance| instance.name == activation.instance())
            .map(|instance| instance.node_type.as_str())
            .expect("the run offers only instances the definition carries");
        let script = behaviours
            .script_for(node_type)
            .expect("every instantiated type has exactly one script");

        let outcome = host::perform(script, &activation, source, limits)
            .and_then(|output| run.produced(output).map_err(ScriptFailure::OutputRefused));
        if let Err(failure) = outcome {
            return Ok(fail(run, failure));
        }
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
use agconflo_core::{Context, ContextType, TypeCatalogue, read_node_types, read_workflow};

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
};

/// A context of type `declared` holding `text`, from `source`.
#[cfg(test)]
pub(crate) fn note(source: &mut IdSource, declared: &str, text: &str) -> Context {
    let declared = ContextType::new(declared).expect("a non-empty type name");
    Context::text(source, declared, text).expect("a fresh source issues")
}

/// Run `definition` with `behaviours` under the small limits, its one entry
/// instance `entry` given `argument` for its parameter `input`.
#[cfg(test)]
pub(crate) fn run_with(
    definition: &WorkflowDefinition,
    behaviours: &Behaviours,
    entry: Option<(&str, &str)>,
) -> Result<RunEnding<ScriptFailure>, ScriptedRefusal> {
    let mut source = IdSource::new();
    let mut arguments = Arguments::new();
    if let Some((instance, text)) = entry {
        let argument = note(&mut source, "note", text);
        arguments = arguments.supply(instance, "input", argument);
    }
    run_scripted(definition, behaviours, arguments, &mut source, 20, SMALL)
}

/// What a completed run rendered, or a panic naming how it ended instead.
#[cfg(test)]
pub(crate) fn rendered(ending: Result<RunEnding<ScriptFailure>, ScriptedRefusal>) -> String {
    match ending {
        Ok(RunEnding::Completed(result)) => result.render().into_owned(),
        other => panic!("expected the run to complete, got {other:?}"),
    }
}

/// The failure a run ended on and the instance it was reported for, or a panic
/// naming how it ended instead.
#[cfg(test)]
pub(crate) fn failed(
    ending: Result<RunEnding<ScriptFailure>, ScriptedRefusal>,
) -> (String, ScriptFailure) {
    match ending {
        Ok(RunEnding::NodeFailed { instance, failure }) => (instance, failure),
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
        let ending = run_scripted(
            &definition,
            &behaviours,
            arguments,
            &mut IdSource::new(),
            20,
            SMALL,
        );

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
