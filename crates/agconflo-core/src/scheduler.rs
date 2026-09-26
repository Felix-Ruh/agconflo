//! The run scheduler: from a definition, the arguments a run was started with
//! and what has been produced so far, which instance may activate next and what
//! that activation carries - the same answer however a run reached them.

use std::collections::HashMap;

use crate::run::Arguments;
use crate::workflow::{NodeInstance, WorkflowDefinition};
use crate::{Context, ContextType};

/// What every instance that has activated produced, by instance name.
pub(crate) type Produced = HashMap<String, Context>;

/// One node about to run: which instance it is for, the node type it performs,
/// the context for each parameter it is given, and the context type it is
/// declared to produce - and, for a node type a model called, which call it
/// performs. Held by value, and told apart by instance and call rather than by
/// an identifier of its own.
// @An activation told apart by instance and call,TRACE_SCHEDULER_ACTIVATION,trace,[],[DEC_RUN_IS_DRIVEN, DEC_ACTIVATION_ONCE_PER_RUN, DEC_CALL_IS_AN_ACTIVATION]
#[derive(Clone, Debug)]
pub struct Activation {
    pub(crate) instance: String,
    pub(crate) node_type: String,
    pub(crate) call: Option<String>,
    pub(crate) inputs: Vec<(String, Context)>,
    pub(crate) output: ContextType,
}

impl Activation {
    /// The instance this activation is for: the one activated, or for a call,
    /// the one whose model made it.
    pub fn instance(&self) -> &str {
        &self.instance
    }

    /// The node type this activation performs: the instance's own, or for a
    /// call, the node type called.
    pub fn node_type(&self) -> &str {
        &self.node_type
    }

    /// The identifier of the call this activation performs, as the provider
    /// issued it, or `None` for an instance's own activation.
    pub fn call(&self) -> Option<&str> {
        self.call.as_deref()
    }

    /// One context per parameter the instance's node type declares, each named
    /// by that parameter, in the order its node type declares them.
    // @Inputs in declared order,TRACE_SCHEDULER_INPUTS,trace,[],[DEC_EVERY_INPUT_REQUIRED]
    pub fn inputs(&self) -> &[(String, Context)] {
        &self.inputs
    }

    /// The context type the activation's node type declares for its one output,
    /// which is the type the run accepts an output of.
    pub fn output(&self) -> &ContextType {
        &self.output
    }
}

/// The next instance that may activate, or `None` when none may, found by
/// asking every instance. Of several offerable instances, the first in the
/// definition's order is returned.
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

/// The activation `instance` may have now, or `None` when it may not activate:
/// when it has not already produced and every parameter its node type declares
/// has a context. A bound parameter takes its source's output, and one nothing
/// binds takes its argument; one with neither blocks for ever.
// @Readiness and the activation it carries,IMPL_SCHEDULER_READY,impl,[CREQ_SCHEDULER_READY_WHEN_BOUND, CREQ_SCHEDULER_ACTIVATION_CARRIES],[DEC_EVERY_INPUT_REQUIRED, DEC_SIGNATURE_IS_WHAT_NOTHING_BINDS]
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

    let mut inputs = Vec::new();
    for parameter in &declared.required {
        match filling(arguments, produced, instance, &parameter.name) {
            Filling::Ready(context) => inputs.push((parameter.name.clone(), context.clone())),
            Filling::Unwired | Filling::Waiting => return None,
        }
    }

    Some(Activation {
        instance: instance.name.clone(),
        node_type: declared.name.clone(),
        call: None,
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
    /// Nothing ever will: the definition binds nothing to it and the run was
    /// given nothing for it.
    Unwired,
}

/// Where one parameter of one instance gets its context, and whether it has one.
/// A requested global context type is not a parameter and never reaches here.
fn filling<'c>(
    arguments: &'c Arguments,
    produced: &'c Produced,
    instance: &NodeInstance,
    parameter: &str,
) -> Filling<'c> {
    let Some(binding) = instance.bindings.iter().find(|b| b.parameter == parameter) else {
        return match arguments.context_for(&instance.name, parameter) {
            Some(context) => Filling::Ready(context),
            None => Filling::Unwired,
        };
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

/// What every activation was given, over a whole sequence of them driven to
/// quiescence, as sorted text.
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
fn every_input_is_awaited() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Sink", &[("must", "note"), ("may", "note")], "note"),
    ];
    let instances = vec![
        instance("a", "Src", &[]),
        instance("b", "Src", &[]),
        instance("sink", "Sink", &[("must", "a"), ("may", "b")]),
    ];
    let workflow = definition(types, instances, &["sink"]);
    let arguments = Arguments::new();
    let sink = &workflow.instances[2];

    // The first parameter has arrived and the second has not, so it is
    // waited for.
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
fn unbound_parameter_never_ready() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Sink", &[("must", "note"), ("may", "note")], "note"),
        node_type("Bound", &[("must", "note")], "note"),
    ];
    let instances = vec![
        instance("a", "Src", &[]),
        instance("sink", "Sink", &[("must", "a")]),
        instance("bound", "Bound", &[("must", "a")]),
    ];
    let workflow = definition(types, instances, &["sink"]);
    let produced = produced_by(&mut source, &["a"], "note");

    // Every binding it has holds a context, and one parameter it declares has
    // none: it is never offered.
    assert!(
        activation_for(
            &workflow,
            &Arguments::new(),
            &produced,
            &workflow.instances[1]
        )
        .is_none()
    );
    // The control: bound in full, the same source is enough.
    let activation = activation_for(
        &workflow,
        &Arguments::new(),
        &produced,
        &workflow.instances[2],
    )
    .expect("every parameter bound and filled");
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
    // for the other.
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
    /// which instances are activated nor what each is given.
    #[test]
    fn inputs_ignore_instance_order(workflow in any_definition()) {
        // Definitions in which two instances share a name are excluded.
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
                    declared.required.iter().any(|p| &p.name == parameter)
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

    /// Quiescence and offering nothing are one answer.
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
            // Some instance carrying that name is offerable.
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
