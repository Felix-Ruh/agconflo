//! The run scheduler: from a definition, the arguments a run was started with
//! and what has been produced so far, which instance may activate next and what
//! that activation carries - the same answer however a run reached them.

use std::collections::{HashMap, HashSet};

use crate::run::Arguments;
use crate::workflow::{NodeInstance, WorkflowDefinition};
use crate::{Context, ContextType};

/// One edge into an instance: the instance, and the parameter of it the edge
/// fills.
type Edge = (String, String);

/// What has been walked along each edge of a run and how far each instance has
/// taken it: every context each edge holds in the order walked, its
/// generation being its place there; how many of them the instance at its end
/// has taken; how many activations of its own each instance has had; and the
/// latest output of each.
#[derive(Clone, Debug, Default)]
pub(crate) struct Edges {
    held: HashMap<Edge, Vec<Context>>,
    taken: HashMap<Edge, usize>,
    runs: HashMap<String, usize>,
    latest: HashMap<String, Context>,
    /// The instances whose output stands.
    standing: HashSet<String>,
}

impl Edges {
    /// The edges of a run of `definition` given `arguments`, before anything has
    /// run: empty but for a context the run was given for a parameter a binding
    /// fills, which is the first that edge holds.
    // @A context given for a bound parameter is its edge's first,IMPL_SCHEDULER_ARGUMENT_FIRST,impl,[CREQ_RUN_ARGUMENT_FIRST_ON_ITS_EDGE],[DEC_ARGUMENT_FIRST_ON_ITS_EDGE]
    pub(crate) fn new(definition: &WorkflowDefinition, arguments: &Arguments) -> Self {
        let mut held = HashMap::new();
        for instance in &definition.instances {
            for binding in &instance.bindings {
                if let Some(given) = arguments.context_for(&instance.name, &binding.parameter) {
                    held.insert(
                        (instance.name.clone(), binding.parameter.clone()),
                        vec![given.clone()],
                    );
                }
            }
        }
        Self {
            held,
            standing: standing(definition),
            ..Self::default()
        }
    }

    /// Record that `activation`, an instance's own, produced `output`: what it
    /// took, and `output` walked along every edge out of its instance - or for
    /// a router, along its edges into the instances `route` names, each
    /// carrying its output or the input of the router's it takes.
    pub(crate) fn produced(
        &mut self,
        definition: &WorkflowDefinition,
        activation: &Activation,
        output: &Context,
        route: Option<&[String]>,
    ) {
        for (parameter, generation) in &activation.taken {
            let taken = self
                .taken
                .entry((activation.instance.clone(), parameter.clone()))
                .or_default();
            *taken = (*taken).max(generation + 1);
        }
        *self.runs.entry(activation.instance.clone()).or_default() += 1;
        match route {
            None => self.walk(definition, &activation.instance, output),
            Some(route) => self.walk_routed(definition, activation, output, route),
        }
    }

    /// A router's `output` walked along each edge out of it into an instance
    /// `route` names, and along no other: the output, or where the edge takes
    /// one of the router's inputs, the context the activation was given for it.
    // @A router's edges walked where it named,IMPL_RUN_WALKS_ROUTED,impl,[CREQ_RUN_WALKS_ROUTED],[DEC_ONE_GRAPH, DEC_ROUTER_OUTPUT_IS_ITS_DECISION]
    fn walk_routed(
        &mut self,
        definition: &WorkflowDefinition,
        activation: &Activation,
        output: &Context,
        route: &[String],
    ) {
        let router = &activation.instance;
        for consumer in definition
            .instances
            .iter()
            .filter(|consumer| route.contains(&consumer.name))
        {
            for binding in consumer.bindings.iter().filter(|b| &b.source == router) {
                let walked = match &binding.input {
                    None => Some(output),
                    Some(input) => activation
                        .inputs
                        .iter()
                        .find(|(parameter, _)| parameter == input)
                        .map(|(_, given)| given),
                };
                if let Some(walked) = walked {
                    self.held
                        .entry((consumer.name.clone(), binding.parameter.clone()))
                        .or_default()
                        .push(walked.clone());
                }
            }
        }
        self.latest.insert(router.clone(), output.clone());
    }

    /// `output` walked along every edge out of `instance`, as the next context
    /// each holds, and kept as that instance's latest.
    // @An output walked along every edge out of its instance,IMPL_RUN_WALKS_EVERY_EDGE,impl,[CREQ_RUN_WALKS_EVERY_EDGE],[DEC_EDGE_GENERATIONS]
    fn walk(&mut self, definition: &WorkflowDefinition, instance: &str, output: &Context) {
        for consumer in &definition.instances {
            for binding in consumer.bindings.iter().filter(|b| b.source == instance) {
                self.held
                    .entry((consumer.name.clone(), binding.parameter.clone()))
                    .or_default()
                    .push(output.clone());
            }
        }
        self.latest.insert(instance.to_owned(), output.clone());
    }

    /// The latest output of `instance`, when it has produced one.
    pub(crate) fn latest(&self, instance: &str) -> Option<&Context> {
        self.latest.get(instance)
    }

    /// Whether `instance` has had an activation of its own accepted.
    pub(crate) fn has_run(&self, instance: &str) -> bool {
        self.runs.contains_key(instance)
    }
}

/// The instances whose output stands: those whose node type declares it, and
/// those with no edge into them that does not stand - found as the least set
/// closed under both, so that a cycle of instances standing only on each other
/// does not stand.
// @Which outputs stand,IMPL_SCHEDULER_STANDING,impl,[CREQ_SCHEDULER_STANDING_SERVES],[DEC_STANDING_OUTPUTS, DEC_ONCE_RUN_OUTPUTS_STAND]
fn standing(definition: &WorkflowDefinition) -> HashSet<String> {
    let declared = |instance: &NodeInstance| {
        definition
            .node_types
            .iter()
            .find(|declared| declared.name == instance.node_type)
            .is_some_and(|declared| declared.standing)
    };
    let mut standing: HashSet<String> = HashSet::new();
    loop {
        let before = standing.len();
        for instance in &definition.instances {
            if declared(instance)
                || instance
                    .bindings
                    .iter()
                    .all(|binding| standing.contains(&binding.source))
            {
                standing.insert(instance.name.clone());
            }
        }
        if standing.len() == before {
            return standing;
        }
    }
}

/// One node about to run: which instance it is for, the node type it performs,
/// the context for each parameter it is given, and the context type it is
/// declared to produce - and, for a node type a model called, which call it
/// performs. Held by value, and told apart by instance and call rather than by
/// an identifier of its own.
// @An activation told apart by instance and call,TRACE_SCHEDULER_ACTIVATION,trace,[],[DEC_RUN_IS_DRIVEN, DEC_EDGE_GENERATIONS, DEC_CALL_IS_AN_ACTIVATION]
#[derive(Clone, Debug)]
pub struct Activation {
    pub(crate) instance: String,
    pub(crate) node_type: String,
    pub(crate) call: Option<String>,
    pub(crate) inputs: Vec<(String, Context)>,
    pub(crate) output: ContextType,
    /// The generation taken from each edge that gave a context not taken
    /// before, by parameter.
    pub(crate) taken: Vec<(String, usize)>,
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
    edges: &Edges,
) -> Option<Activation> {
    definition
        .instances
        .iter()
        .find_map(|instance| activation_for(definition, arguments, edges, instance))
}

/// The activation `instance` may have now, or `None` when it may not activate.
///
/// Every parameter its node type declares needs a context: a bound one the
/// earliest its edge holds that the instance has not taken, or the latest when
/// it has taken them all and its source's output stands; one nothing binds its
/// argument. An instance that has run needs at least one of them to be one it
/// has not taken, so an instance with no edge into it runs once.
// @Readiness and the activation it carries,IMPL_SCHEDULER_READY,impl,[CREQ_SCHEDULER_READY_WHEN_BOUND, CREQ_SCHEDULER_ACTIVATION_CARRIES, CREQ_SCHEDULER_TAKES_EARLIEST, CREQ_SCHEDULER_OFFERS_AGAIN],[DEC_EVERY_INPUT_REQUIRED, DEC_SIGNATURE_IS_WHAT_NOTHING_BINDS, DEC_EDGE_GENERATIONS, DEC_RUN_AGAIN_ON_SOMETHING_NEW]
pub(crate) fn activation_for(
    definition: &WorkflowDefinition,
    arguments: &Arguments,
    edges: &Edges,
    instance: &NodeInstance,
) -> Option<Activation> {
    let declared = definition
        .node_types
        .iter()
        .find(|declared| declared.name == instance.node_type)?;

    let mut inputs = Vec::new();
    let mut taken = Vec::new();
    for parameter in &declared.required {
        match filling(arguments, edges, instance, &parameter.name) {
            Filling::New(context, generation) => {
                inputs.push((parameter.name.clone(), context.clone()));
                taken.push((parameter.name.clone(), generation));
            }
            Filling::Held(context) => inputs.push((parameter.name.clone(), context.clone())),
            Filling::Unwired | Filling::Waiting => return None,
        }
    }
    if edges.has_run(&instance.name) && taken.is_empty() {
        return None;
    }

    Some(Activation {
        instance: instance.name.clone(),
        node_type: declared.name.clone(),
        call: None,
        inputs,
        output: declared.output.clone(),
        taken,
    })
}

/// What stands where one parameter's context would.
enum Filling<'c> {
    /// A context its edge holds that the instance has not taken, and its
    /// generation.
    New(&'c Context, usize),
    /// A context it has been given before and is given again: a standing
    /// output, or the argument filling a parameter nothing binds.
    Held(&'c Context),
    /// Something will fill it and has not yet.
    Waiting,
    /// Nothing ever will: the definition binds nothing to it and the run was
    /// given nothing for it.
    Unwired,
}

/// Where one parameter of one instance gets its context, and whether it has one.
/// A requested global context type is not a parameter and never reaches here.
// @The earliest context not taken or the one that stands,IMPL_SCHEDULER_TAKES_EARLIEST,impl,[CREQ_SCHEDULER_TAKES_EARLIEST, CREQ_SCHEDULER_STANDING_SERVES],[DEC_EDGE_GENERATIONS, DEC_STANDING_OUTPUTS]
fn filling<'c>(
    arguments: &'c Arguments,
    edges: &'c Edges,
    instance: &NodeInstance,
    parameter: &str,
) -> Filling<'c> {
    let Some(binding) = instance.bindings.iter().find(|b| b.parameter == parameter) else {
        return match arguments.context_for(&instance.name, parameter) {
            Some(context) => Filling::Held(context),
            None => Filling::Unwired,
        };
    };

    let edge = (instance.name.clone(), parameter.to_owned());
    let held = edges.held.get(&edge).map_or(&[][..], Vec::as_slice);
    let taken = edges.taken.get(&edge).copied().unwrap_or(0);
    match held.get(taken) {
        Some(context) => Filling::New(context, taken),
        None => match held.last() {
            Some(context) if edges.standing.contains(&binding.source) => Filling::Held(context),
            _ => Filling::Waiting,
        },
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

/// What each instance has produced, by name.
#[cfg(test)]
type Produced = HashMap<String, Context>;

#[cfg(test)]
impl Edges {
    /// `output` walked out of `instance` as its own activation's, having taken
    /// nothing.
    fn walked(&mut self, definition: &WorkflowDefinition, instance: &str, output: &Context) {
        *self.runs.entry(instance.to_owned()).or_default() += 1;
        self.walk(definition, instance, output);
    }
}

/// The edges of a run of `workflow` in which each instance `produced` names has
/// produced its context once, in the definition's order.
#[cfg(test)]
fn edges_of(workflow: &WorkflowDefinition, produced: &Produced) -> Edges {
    let mut edges = Edges::new(workflow, &Arguments::new());
    for node in &workflow.instances {
        if let Some(output) = produced.get(&node.name) {
            edges.walked(workflow, &node.name, output);
        }
    }
    edges
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

/// Whether every edge of `workflow` is one edge: no two instances share a name,
/// and no instance binds a parameter twice. A run refuses a definition in which
/// either happens, and an edge is an instance's parameter by name, so where
/// they do, two wires feed one queue.
#[cfg(test)]
fn each_edge_once(workflow: &WorkflowDefinition) -> bool {
    let unique = |mut names: Vec<&str>| {
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        names.len() == count
    };
    unique(
        workflow
            .instances
            .iter()
            .map(|node| node.name.as_str())
            .collect(),
    ) && workflow
        .instances
        .iter()
        .all(|node| unique(node.bindings.iter().map(|b| b.parameter.as_str()).collect()))
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
    let mut edges = Edges::new(workflow, &arguments);
    let mut answers = Vec::new();

    while let Some(activation) = next_activation(workflow, &arguments, &edges) {
        assert!(
            answers.len() < 1000,
            "a definition with no arguments never repeats"
        );
        answers.push(format!(
            "{}|{}",
            activation.instance(),
            given(&activation).join(",")
        ));
        let context = ctx(&mut source, "note");
        edges.produced(workflow, &activation, &context, None);
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
    assert!(activation_for(&workflow, &arguments, &edges_of(&workflow, &produced), sink).is_none());

    // Once it arrives the instance is offered, and the waiting was for
    // something: both contexts are carried, in declared order.
    let may = ctx(&mut source, "note");
    let may_id = may.id();
    produced.insert("b".to_owned(), may);
    let activation = activation_for(&workflow, &arguments, &edges_of(&workflow, &produced), sink)
        .expect("everything bound is there");
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
            &edges_of(&workflow, &produced),
            &workflow.instances[1]
        )
        .is_none()
    );
    // The control: bound in full, the same source is enough.
    let activation = activation_for(
        &workflow,
        &Arguments::new(),
        &edges_of(&workflow, &produced),
        &workflow.instances[2],
    )
    .expect("every parameter bound and filled");
    assert_eq!(given(&activation), ["must"]);
}

#[cfg(test)]
#[test]
fn input_is_ready_at_once() {
    let mut source = IdSource::new();
    let types = vec![node_type("Given", &[("seed", "note")], "note")];
    let instances = vec![instance("e", "Given", &[])];
    let workflow = definition(types, instances, &["e"]);

    let seed = ctx(&mut source, "note");
    let seed_id = seed.id();
    let arguments = Arguments::new().supply("e", "seed", seed);

    let activation = next_activation(
        &workflow,
        &arguments,
        &edges_of(&workflow, &Produced::new()),
    )
    .expect("an instance given its inputs is ready before anything has run");
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
    assert!(
        next_activation(
            &workflow,
            &Arguments::new(),
            &edges_of(&workflow, &produced)
        )
        .is_none()
    );
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
    assert!(
        next_activation(
            &workflow,
            &arguments,
            &edges_of(&workflow, &Produced::new())
        )
        .is_some()
    );

    // Having produced, it is not offered again, and with nothing else to offer
    // the scheduler says so rather than cycling over it for ever.
    let produced = produced_by(&mut source, &["a"], "note");
    assert!(next_activation(&workflow, &arguments, &edges_of(&workflow, &produced)).is_none());
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
        &edges_of(&workflow, &produced),
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
        &edges_of(&workflow, &produced),
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
        &edges_of(&workflow, &produced),
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
        node_type("Given", &[], "note"),
        node_type("Step", &[("held", "note"), ("awaited", "note")], "note"),
    ];
    let instances = vec![
        instance("e", "Given", &[]),
        instance("c1", "Step", &[("held", "e"), ("awaited", "c3")]),
        instance("c2", "Step", &[("held", "e"), ("awaited", "c1")]),
        instance("c3", "Step", &[("held", "e"), ("awaited", "c2")]),
    ];
    let workflow = definition(types, instances, &["c1"]);
    let produced = produced_by(&mut source, &["e"], "note");

    assert!(
        next_activation(
            &workflow,
            &Arguments::new(),
            &edges_of(&workflow, &produced)
        )
        .is_none()
    );
}

/// The activation `name` may have now in `workflow`, given no argument.
#[cfg(test)]
fn offered(workflow: &WorkflowDefinition, edges: &Edges, name: &str) -> Option<Activation> {
    let node = workflow
        .instances
        .iter()
        .find(|node| node.name == name)
        .expect("the instance is in the workflow");
    activation_for(workflow, &Arguments::new(), edges, node)
}

/// The identifiers an activation was given, in declared order.
#[cfg(test)]
fn ids(activation: &Activation) -> Vec<crate::ContextId> {
    activation
        .inputs()
        .iter()
        .map(|(_, context)| context.id())
        .collect()
}

#[cfg(test)]
#[test]
fn takes_earliest() {
    let mut source = IdSource::new();
    // Each source reads its own output, so neither stands.
    let types = vec![
        node_type("Again", &[("input", "note")], "note"),
        node_type("Pair", &[("left", "note"), ("right", "note")], "note"),
    ];
    let instances = vec![
        instance("x", "Again", &[("input", "x")]),
        instance("y", "Again", &[("input", "y")]),
        instance("c", "Pair", &[("left", "x"), ("right", "y")]),
    ];
    let workflow = definition(types, instances, &["c"]);
    let mut edges = Edges::new(&workflow, &Arguments::new());

    // Three contexts along the left edge, one along the right.
    let lefts: Vec<Context> = (0..3).map(|_| ctx(&mut source, "note")).collect();
    for left in &lefts {
        edges.walked(&workflow, "x", left);
    }
    let first_right = ctx(&mut source, "note");
    edges.walked(&workflow, "y", &first_right);

    // The earliest of each, not the latest of the left.
    let first = offered(&workflow, &edges, "c").expect("both edges hold one");
    assert_eq!(ids(&first), [lefts[0].id(), first_right.id()]);
    edges.produced(&workflow, &first, &ctx(&mut source, "note"), None);

    // The right edge holds nothing new, so the left edge's second waits.
    assert!(offered(&workflow, &edges, "c").is_none());

    // Given another on the right, the next of each: nothing taken twice.
    let second_right = ctx(&mut source, "note");
    edges.walked(&workflow, "y", &second_right);
    let second = offered(&workflow, &edges, "c").expect("both edges hold a new one");
    assert_eq!(ids(&second), [lefts[1].id(), second_right.id()]);
}

#[cfg(test)]
#[test]
fn offered_again_on_something_new() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Brief", &[], "note").standing(),
        node_type("Src", &[], "note"),
        node_type("Pair", &[("left", "note"), ("right", "note")], "note"),
        node_type("Take", &[("input", "note")], "note"),
    ];
    let instances = vec![
        instance("s", "Brief", &[]),
        instance("v", "Src", &[]),
        instance("c", "Pair", &[("left", "s"), ("right", "v")]),
        instance("only", "Take", &[("input", "s")]),
    ];
    let workflow = definition(types, instances, &["c"]);
    let mut edges = Edges::new(&workflow, &Arguments::new());

    // An instance reading nothing is offered once, and never again.
    let src = offered(&workflow, &edges, "v").expect("nothing to wait for");
    edges.produced(&workflow, &src, &ctx(&mut source, "note"), None);
    for _ in 0..3 {
        assert!(offered(&workflow, &edges, "v").is_none());
    }

    let brief = offered(&workflow, &edges, "s").expect("nothing to wait for");
    edges.produced(&workflow, &brief, &ctx(&mut source, "note"), None);

    // Offered on a standing output and a new one; then, with nothing new on
    // the changing edge, not offered however often it is asked.
    let first = offered(&workflow, &edges, "c").expect("both edges hold one");
    edges.produced(&workflow, &first, &ctx(&mut source, "note"), None);
    for _ in 0..3 {
        assert!(offered(&workflow, &edges, "c").is_none());
    }

    // Something new on the changing edge: offered again.
    edges.walked(&workflow, "v", &ctx(&mut source, "note"));
    assert!(offered(&workflow, &edges, "c").is_some());

    // An instance reading only a standing output is offered once, and not
    // again on the same output.
    let only = offered(&workflow, &edges, "only").expect("the brief is there");
    edges.produced(&workflow, &only, &ctx(&mut source, "note"), None);
    for _ in 0..3 {
        assert!(offered(&workflow, &edges, "only").is_none());
    }
}

#[cfg(test)]
#[test]
fn standing_serves_later() {
    let mut source = IdSource::new();
    let types = vec![
        node_type("Brief", &[], "note").standing(),
        node_type("Src", &[], "note"),
        node_type("Given", &[("seed", "note")], "note"),
        node_type("Pair", &[("left", "note"), ("right", "note")], "note"),
    ];
    let instances = vec![
        instance("s", "Brief", &[]),
        instance("v", "Src", &[]),
        instance("g", "Given", &[]),
        instance("c", "Pair", &[("left", "s"), ("right", "v")]),
        instance("d", "Pair", &[("left", "g"), ("right", "v")]),
    ];
    let workflow = definition(types, instances, &["c"]);
    let mut edges = Edges::new(&workflow, &Arguments::new());

    let first_brief = ctx(&mut source, "note");
    edges.walked(&workflow, "s", &first_brief);
    // g is given its input by the run and runs once; its output stands though
    // its node type does not say so.
    let given = ctx(&mut source, "note");
    edges.walked(&workflow, "g", &given);

    let mut passes = Vec::new();
    for pass in 0..3 {
        if pass == 2 {
            // The standing node produces again, before the third pass.
            let second_brief = ctx(&mut source, "note");
            edges.walked(&workflow, "s", &second_brief);
            passes.push(second_brief.id());
        }
        edges.walked(&workflow, "v", &ctx(&mut source, "note"));
        let c = offered(&workflow, &edges, "c").expect("a new context on the right");
        let d = offered(&workflow, &edges, "d").expect("a new context on the right");
        passes.push(c.inputs()[0].1.id());
        passes.push(d.inputs()[0].1.id());
        edges.produced(&workflow, &c, &ctx(&mut source, "note"), None);
        edges.produced(&workflow, &d, &ctx(&mut source, "note"), None);
    }

    let second_brief = passes[4];
    assert_eq!(
        passes,
        [
            first_brief.id(),
            given.id(),
            first_brief.id(),
            given.id(),
            second_brief,
            second_brief,
            given.id(),
        ]
    );
}

#[cfg(test)]
proptest! {
    /// Permuting the order a definition carries its instances in changes neither
    /// which instances are activated nor what each is given.
    #[test]
    fn inputs_ignore_instance_order(workflow in any_definition()) {
        prop_assume!(each_edge_once(&workflow));

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
        prop_assume!(each_edge_once(&workflow));

        let mut source = IdSource::new();
        let mut produced = Produced::new();
        for (position, node) in workflow.instances.iter().enumerate() {
            if taken >> (position % 64) & 1 == 1 {
                produced.insert(node.name.clone(), ctx(&mut source, "note"));
            }
        }

        for node in &workflow.instances {
            let Some(activation) = activation_for(&workflow, &Arguments::new(), &edges_of(&workflow, &produced), node)
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
                    .expect("an instance given no argument is given only what it binds");
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

        let offered = next_activation(&workflow, &arguments, &edges_of(&workflow, &produced));
        let any_offerable = workflow
            .instances
            .iter()
            .any(|node| activation_for(&workflow, &arguments, &edges_of(&workflow, &produced), node).is_some());
        prop_assert_eq!(offered.is_some(), any_offerable);

        if let Some(activation) = offered {
            // Some instance carrying that name is offerable.
            prop_assert!(
                workflow
                    .instances
                    .iter()
                    .filter(|node| node.name == activation.instance())
                    .any(|node| activation_for(&workflow, &arguments, &edges_of(&workflow, &produced), node).is_some())
            );
        }
    }
}
