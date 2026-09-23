//! The run scheduler: from a definition, the arguments a run was started with
//! and what has been produced so far, which instance may activate next and what
//! that activation carries.
//!
//! There is no history in it. The same definition, the same arguments and the
//! same produced outputs give the same answer however a run reached them, which
//! is what makes the answer independent of the order a definition happens to
//! carry its instances in - the one defect measured on the prototype this was
//! built from (`EVD_RUN_OPTIONAL_BY_ORDER`).

use std::collections::HashMap;

use crate::run::Arguments;
use crate::workflow::{NodeInstance, WorkflowDefinition};
use crate::{Context, ContextType};

/// What every instance that has activated produced, by instance name.
pub(crate) type Produced = HashMap<String, Context>;

/// One node about to run: which instance, the context for each parameter it is
/// given, and the context type it is declared to produce.
///
/// Held by value rather than by reference into the definition, because the
/// caller performs the activation (`DEC_RUN_IS_DRIVEN`) and a `Context` is a
/// handle - cloning one shares the value rather than copying it.
///
/// It carries no identifier of its own. No instance activates twice in a run
/// (`DEC_ACTIVATION_ONCE_PER_RUN`), so the instance name says which activation
/// this is; the moment that stops being true, activation tagging is required
/// before anything is built on top.
#[derive(Clone, Debug)]
pub struct Activation {
    instance: String,
    inputs: Vec<(String, Context)>,
    output: ContextType,
}

impl Activation {
    /// The instance this activates.
    pub fn instance(&self) -> &str {
        &self.instance
    }

    /// One context per parameter the instance is given, each named by that
    /// parameter, in the order its node type declares them: the required list in
    /// its own order, then the optional list in its own.
    ///
    /// Declared order rather than the order the definition binds them in, and
    /// rather than sorted: a node assembles its own inputs from its declared
    /// parameter list (`DEC_DECLARED_PARAMETERS`), so the order is part of what
    /// an activation is.
    ///
    /// A parameter the definition leaves unbound is absent rather than empty,
    /// which is the only way an optional parameter can be distinguished from one
    /// bound to something that rendered to nothing.
    pub fn inputs(&self) -> &[(String, Context)] {
        &self.inputs
    }

    /// The context type the instance's node type declares for its one output,
    /// which is the type the run accepts an output of
    /// (`CREQ_RUN_REFUSES_UNDECLARED_OUTPUT`).
    ///
    /// Carried here rather than looked up again when the output is reported: the
    /// declaration was already in hand when the activation was built, and a
    /// caller performing the activation needs to know it just as much.
    pub fn output(&self) -> &ContextType {
        &self.output
    }
}

/// The next instance that may activate, or `None` when none may.
///
/// `None` is quiescence, and it is reached by asking every instance rather than
/// by inspecting the graph - which is what makes it true of a cycle nobody
/// detected as readily as of a workflow whose every instance has produced. What
/// a run does about it is the run's (`CREQ_RUN_ENDS_QUIESCENT`); this only says
/// that nothing may.
///
/// Which of several offerable instances is returned is the order the definition
/// carries them in, and that choice is deliberately the only thing order decides:
/// what any instance would be given does not depend on it.
// @Nothing may activate,IMPL_SCHEDULER_NONE_READY,impl,[CREQ_SCHEDULER_NONE_READY]
pub(crate) fn next_activation(
    definition: &WorkflowDefinition,
    arguments: &Arguments,
    produced: &Produced,
) -> Option<Activation> {
    definition
        .instances
        .iter()
        .find_map(|instance| activation_for(definition, arguments, produced, instance))
}

/// The activation `instance` may have now, or `None` when it may not activate.
///
/// Readiness and what the activation carries are one answer rather than two
/// steps, and they could not sensibly be separated: knowing that every parameter
/// an instance binds has a context *is* gathering those contexts. That is why
/// `ARCH_RUN` has no component for an activation.
///
/// An instance may activate when it has not already produced and every parameter
/// it binds has a context, whether that parameter is declared required or
/// optional (`DEC_BINDING_IS_AWAITED`). A parameter left unbound by the
/// definition is skipped when it is optional and blocks for ever when it is
/// required, the latter being a wiring defect a run refuses to start on.
///
/// Two shapes are the run's to refuse before anything reaches here, and are read
/// rather than guarded so that the refusal stays in one place: an entry
/// instance's parameters are taken from the arguments alone, so a binding on one
/// is not read (`CREQ_RUN_REFUSES_UNFILLED_SIGNATURE`), and a parameter declared
/// both required and optional by one node type is read twice, which is the shape
/// the reader already refuses a document for.
// @Readiness and the activation it carries,IMPL_SCHEDULER_READY,impl,[CREQ_SCHEDULER_READY_WHEN_BOUND, CREQ_SCHEDULER_ACTIVATION_CARRIES]
pub(crate) fn activation_for(
    definition: &WorkflowDefinition,
    arguments: &Arguments,
    produced: &Produced,
    instance: &NodeInstance,
) -> Option<Activation> {
    if produced.contains_key(&instance.name) {
        return None;
    }

    let declared = definition
        .node_types
        .iter()
        .find(|declared| declared.name == instance.node_type)?;

    let listed = (declared.required.iter().map(|p| (p, true)))
        .chain(declared.optional.iter().map(|p| (p, false)));

    let mut inputs = Vec::new();
    for (parameter, required) in listed {
        match filling(arguments, produced, instance, &parameter.name) {
            Filling::Ready(context) => inputs.push((parameter.name.clone(), context.clone())),
            Filling::Unwired if !required => continue,
            Filling::Unwired | Filling::Waiting => return None,
        }
    }

    Some(Activation {
        instance: instance.name.clone(),
        inputs,
        output: declared.output.clone(),
    })
}

/// What stands where one parameter's context would.
enum Filling<'c> {
    /// The context is there.
    Ready(&'c Context),
    /// Something will fill it and has not yet.
    Waiting,
    /// Nothing ever will: the definition binds nothing to it and supplies
    /// nothing for it.
    Unwired,
}

/// Where one parameter of one instance gets its context, and whether it has one.
///
/// A requested global context type is not a parameter and never reaches here: a
/// node type declares the global types it reads and nothing supplies them, so a
/// context arriving under that heading would have been invented.
fn filling<'c>(
    arguments: &'c Arguments,
    produced: &'c Produced,
    instance: &NodeInstance,
    parameter: &str,
) -> Filling<'c> {
    if instance.entry {
        return match arguments.context_for(&instance.name, parameter) {
            Some(context) => Filling::Ready(context),
            None => Filling::Unwired,
        };
    }

    let Some(binding) = instance.bindings.iter().find(|b| b.parameter == parameter) else {
        return Filling::Unwired;
    };

    match produced.get(&binding.source) {
        Some(context) => Filling::Ready(context),
        None => Filling::Waiting,
    }
}

#[cfg(test)]
use crate::IdSource;
#[cfg(test)]
use crate::wiring::any_definition;
#[cfg(test)]
use crate::workflow::{context_type, definition, instance, node_type};
#[cfg(test)]
use proptest::prelude::*;

/// A context of `type_name`, from a source the caller keeps.
#[cfg(test)]
fn ctx(source: &mut IdSource, type_name: &str) -> Context {
    Context::text(source, context_type(type_name), "x").expect("a fresh source issues")
}

/// What `names` have produced, a fresh context of `type_name` each.
#[cfg(test)]
fn produced_by(source: &mut IdSource, names: &[&str], type_name: &str) -> Produced {
    let mut produced = Produced::new();
    for &name in names {
        produced.insert(name.to_owned(), ctx(source, type_name));
    }
    produced
}

/// The parameter names an activation carries, in the order it carries them.
#[cfg(test)]
fn given(activation: &Activation) -> Vec<&str> {
    activation
        .inputs()
        .iter()
        .map(|(parameter, _)| parameter.as_str())
        .collect()
}

/// What every activation was given, over a whole sequence of them, as sorted
/// text.
///
/// The scheduler is **driven to quiescence** rather than asked once per
/// instance, and that is the whole point of the helper. Asked one instance at a
/// time the answer cannot depend on the definition's order at all, so a property
/// built that way passes against the very defect it exists to catch - measured:
/// written the careless way, reverting readiness to the required parameters
/// alone left it green.
///
/// The run is deliberately not used. It refuses a definition carrying wiring
/// defects and most generated ones carry some, which would leave the property
/// comparing two empty lists.
///
/// Sorted, because which instance is offered first legitimately depends on the
/// order; what each is given does not. Text rather than the values, because a
/// `ContextId` is deliberately not `Ord`.
#[cfg(test)]
fn answers(workflow: &WorkflowDefinition) -> Vec<String> {
    let mut source = IdSource::new();
    let arguments = Arguments::new();
    let mut produced = Produced::new();
    let mut answers = Vec::new();

    while let Some(activation) = next_activation(workflow, &arguments, &produced) {
        answers.push(format!(
            "{}|{}",
            activation.instance(),
            given(&activation).join(",")
        ));
        let context = ctx(&mut source, "note");
        produced.insert(activation.instance().to_owned(), context);
    }

    answers.sort();
    answers
}

#[cfg(test)]
#[test]
fn bound_optional_is_awaited() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Sink", &[("must", "note")], "note").with_optional(&[("may", "note")]),
    ];
    let instances = vec![
        instance("a", "Src", &[]),
        instance("b", "Src", &[]),
        instance("sink", "Sink", &[("must", "a"), ("may", "b")]),
    ];
    let workflow = definition(types, instances, &["sink"]);
    let arguments = Arguments::new();
    let sink = &workflow.instances[2];

    // The required parameter has arrived and the optional one has not. The
    // optional one is bound, so it is waited for.
    let mut produced = produced_by(&mut source, &["a"], "note");
    assert!(activation_for(&workflow, &arguments, &produced, sink).is_none());

    // Once it arrives the instance is offered, and the waiting was for
    // something: both contexts are carried, in declared order.
    let may = ctx(&mut source, "note");
    let may_id = may.id();
    produced.insert("b".to_owned(), may);
    let activation =
        activation_for(&workflow, &arguments, &produced, sink).expect("everything bound is there");
    assert_eq!(given(&activation), ["must", "may"]);
    assert_eq!(activation.inputs()[1].1.id(), may_id);
}

#[cfg(test)]
#[test]
fn unbound_optional_is_ready() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Sink", &[("must", "note")], "note").with_optional(&[("may", "note")]),
    ];
    let instances = vec![
        instance("a", "Src", &[]),
        instance("sink", "Sink", &[("must", "a")]),
    ];
    let workflow = definition(types, instances, &["sink"]);
    let produced = produced_by(&mut source, &["a"], "note");

    let activation = activation_for(
        &workflow,
        &Arguments::new(),
        &produced,
        &workflow.instances[1],
    )
    .expect("an unbound optional parameter holds nothing back");
    assert_eq!(given(&activation), ["must"]);
}

#[cfg(test)]
#[test]
fn entry_is_ready_at_once() {
    let mut source = IdSource::new();
    let types = vec![node_type("Entry", &[("seed", "note")], "note")];
    let instances = vec![instance("e", "Entry", &[]).into_entry()];
    let workflow = definition(types, instances, &["e"]);

    let seed = ctx(&mut source, "note");
    let seed_id = seed.id();
    let arguments = Arguments::new().supply("e", "seed", seed);

    let activation = next_activation(&workflow, &arguments, &Produced::new())
        .expect("an entry instance is ready before anything has run");
    assert_eq!(activation.instance(), "e");
    assert_eq!(given(&activation), ["seed"]);
    assert_eq!(activation.inputs()[0].1.id(), seed_id);
}

#[cfg(test)]
#[test]
fn unresolved_source_never_ready() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Sink", &[("must", "note")], "note"),
    ];
    let instances = vec![
        instance("a", "Src", &[]),
        instance("sink", "Sink", &[("must", "ghost")]),
    ];
    let workflow = definition(types, instances, &["sink"]);

    // Everything else has produced, so the only thing left is the instance
    // bound to a name that resolves to nothing.
    let produced = produced_by(&mut source, &["a"], "note");
    assert!(next_activation(&workflow, &Arguments::new(), &produced).is_none());
}

#[cfg(test)]
#[test]
fn produced_instance_not_offered() {
    let mut source = IdSource::new();
    let types = vec![node_type("Src", &[], "note")];
    let instances = vec![instance("a", "Src", &[])];
    let workflow = definition(types, instances, &["a"]);
    let arguments = Arguments::new();

    // An instance with nothing to wait for is offered.
    assert!(next_activation(&workflow, &arguments, &Produced::new()).is_some());

    // Having produced, it is not offered again, and with nothing else to offer
    // the scheduler says so rather than cycling over it for ever.
    let produced = produced_by(&mut source, &["a"], "note");
    assert!(next_activation(&workflow, &arguments, &produced).is_none());
}

#[cfg(test)]
#[test]
fn inputs_in_declared_order() {
    let mut source = IdSource::new();
    // Declared order is neither alphabetical nor the order the definition binds
    // them in, so three readings that agree on a careless example disagree here.
    let types = vec![
        node_type("Src", &[], "note"),
        node_type(
            "Sink",
            &[("zebra", "note"), ("alpha", "note"), ("middle", "note")],
            "note",
        ),
    ];
    let instances = vec![
        instance("p", "Src", &[]),
        instance("q", "Src", &[]),
        instance("r", "Src", &[]),
        instance(
            "sink",
            "Sink",
            &[("middle", "p"), ("zebra", "q"), ("alpha", "r")],
        ),
    ];
    let workflow = definition(types, instances, &["sink"]);
    let produced = produced_by(&mut source, &["p", "q", "r"], "note");

    let activation = activation_for(
        &workflow,
        &Arguments::new(),
        &produced,
        &workflow.instances[3],
    )
    .expect("every parameter is bound and produced");
    assert_eq!(given(&activation), ["zebra", "alpha", "middle"]);
}

#[cfg(test)]
#[test]
fn each_parameter_gets_its_own() {
    let mut source = IdSource::new();
    // Both parameters are declared for one context type, so only the identity of
    // the context catches them being swapped.
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Sink", &[("left", "note"), ("right", "note")], "note"),
    ];
    let instances = vec![
        instance("p", "Src", &[]),
        instance("q", "Src", &[]),
        instance("sink", "Sink", &[("left", "p"), ("right", "q")]),
    ];
    let workflow = definition(types, instances, &["sink"]);
    let produced = produced_by(&mut source, &["p", "q"], "note");
    let (left, right) = (produced["p"].id(), produced["q"].id());

    let activation = activation_for(
        &workflow,
        &Arguments::new(),
        &produced,
        &workflow.instances[2],
    )
    .expect("both are there");
    assert_eq!(given(&activation), ["left", "right"]);
    assert_eq!(activation.inputs()[0].1.id(), left);
    assert_eq!(activation.inputs()[1].1.id(), right);
}

#[cfg(test)]
#[test]
fn globals_are_not_given() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Sink", &[("must", "note")], "note").with_globals(&["secret"]),
    ];
    let instances = vec![
        instance("a", "Src", &[]),
        instance("sink", "Sink", &[("must", "a")]),
    ];
    let workflow = definition(types, instances, &["sink"]);
    let produced = produced_by(&mut source, &["a"], "note");

    let activation = activation_for(
        &workflow,
        &Arguments::new(),
        &produced,
        &workflow.instances[1],
    )
    .expect("the bound parameter is there");
    assert_eq!(given(&activation), ["must"]);
}

#[cfg(test)]
#[test]
fn partial_inputs_still_quiescent() {
    let mut source = IdSource::new();
    // Three instances in a cycle, each holding one of its two inputs and waiting
    // for the other. A reading that reports quiescence only when no instance
    // holds any context at all never fires here.
    let types = vec![
        node_type("Entry", &[], "note"),
        node_type("Step", &[("held", "note"), ("awaited", "note")], "note"),
    ];
    let instances = vec![
        instance("e", "Entry", &[]),
        instance("c1", "Step", &[("held", "e"), ("awaited", "c3")]),
        instance("c2", "Step", &[("held", "e"), ("awaited", "c1")]),
        instance("c3", "Step", &[("held", "e"), ("awaited", "c2")]),
    ];
    let workflow = definition(types, instances, &["c1"]);
    let produced = produced_by(&mut source, &["e"], "note");

    assert!(next_activation(&workflow, &Arguments::new(), &produced).is_none());
}

#[cfg(test)]
proptest! {
    /// Permuting the order a definition carries its instances in changes neither
    /// which instances are activated nor what each is given. The property form
    /// of the measured defect, which was visible only because one line moved.
    #[test]
    fn inputs_ignore_instance_order(workflow in any_definition()) {
        // Definitions in which two instances share a name are excluded, and the
        // exclusion is the finding rather than a convenience. What has produced
        // is keyed by instance name, so a shared name makes one key stand for
        // two instances and which of them is offered first does depend on the
        // order - the property is simply false there. A run never meets the
        // shape: a shared name is a wiring defect and the run refuses to start
        // (`CREQ_VALIDATOR_INSTANCE_NAMED_ONCE`, `CREQ_RUN_REFUSES_DEFECTS`).
        let mut names: Vec<&str> =
            workflow.instances.iter().map(|node| node.name.as_str()).collect();
        names.sort_unstable();
        let unique = names.len();
        names.dedup();
        prop_assume!(names.len() == unique);

        let first = answers(&workflow);

        let mut reversed = workflow.clone();
        reversed.instances.reverse();
        prop_assert_eq!(&first, &answers(&reversed));

        let mut rotated = workflow.clone();
        if !rotated.instances.is_empty() {
            rotated.instances.rotate_left(1);
        }
        prop_assert_eq!(&first, &answers(&rotated));
    }

    /// Every activation carries one context per parameter its instance binds,
    /// each the output of that binding's source, with none missing and none
    /// added.
    #[test]
    fn activation_is_exactly_its_bindings(workflow in any_definition(), taken in any::<u64>()) {
        let mut source = IdSource::new();
        let mut produced = Produced::new();
        for (position, node) in workflow.instances.iter().enumerate() {
            if taken >> (position % 64) & 1 == 1 {
                produced.insert(node.name.clone(), ctx(&mut source, "note"));
            }
        }

        for node in &workflow.instances {
            let Some(activation) = activation_for(&workflow, &Arguments::new(), &produced, node)
            else {
                continue;
            };
            let declared = workflow
                .node_types
                .iter()
                .find(|declared| declared.name == node.node_type)
                .expect("an instance with no declaration is never offered");

            for (parameter, context) in activation.inputs() {
                // Declared by the node type, rather than invented or a global.
                prop_assert!(
                    declared.required.iter().chain(&declared.optional)
                        .any(|p| &p.name == parameter)
                );
                // The context of the binding's own source, not of another.
                let binding = node.bindings.iter().find(|b| &b.parameter == parameter)
                    .expect("a non-entry instance is given only what it binds");
                prop_assert_eq!(context.id(), produced[&binding.source].id());
            }

            // Every required parameter is there.
            for parameter in &declared.required {
                prop_assert!(activation.inputs().iter().any(|(name, _)| name == &parameter.name));
            }
        }
    }

    /// Quiescence and offering nothing are one answer. A scheduler reporting
    /// that no instance may activate while one is offerable would end a run
    /// whose result was still reachable.
    #[test]
    fn none_ready_only_when_none(workflow in any_definition(), taken in any::<u64>()) {
        let mut source = IdSource::new();
        let mut produced = Produced::new();
        for (position, node) in workflow.instances.iter().enumerate() {
            if taken >> (position % 64) & 1 == 1 {
                produced.insert(node.name.clone(), ctx(&mut source, "note"));
            }
        }
        let arguments = Arguments::new();

        let offered = next_activation(&workflow, &arguments, &produced);
        let any_offerable = workflow
            .instances
            .iter()
            .any(|node| activation_for(&workflow, &arguments, &produced, node).is_some());
        prop_assert_eq!(offered.is_some(), any_offerable);

        if let Some(activation) = offered {
            // Some instance carrying that name is offerable. Not "the first one
            // carrying it": two instances may share a name, and the first is
            // then not necessarily the one that was offered - which is what this
            // property caught when it was written the careless way.
            prop_assert!(
                workflow
                    .instances
                    .iter()
                    .filter(|node| node.name == activation.instance())
                    .any(|node| activation_for(&workflow, &arguments, &produced, node).is_some())
            );
        }
    }
}
