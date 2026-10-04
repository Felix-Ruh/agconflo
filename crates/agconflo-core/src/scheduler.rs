//! The run scheduler: from a definition, the arguments a run was started with
//! and what has been produced so far, which instance may activate next and what
//! that activation carries - the same answer however a run reached them.

use std::collections::HashMap;

use crate::clock::{Clock, Clocks};
use crate::run::Arguments;
use crate::workflow::{Binding, NodeInstance, WorkflowDefinition};
use crate::{Context, ContextType};

/// What each instance of a run has produced, pass by pass, and each router's
/// inputs and the branch it took on each of its passes - with the passes each
/// instance runs on, worked out when the run started, and the first context
/// each binding declaring one gives.
#[derive(Clone, Debug, Default)]
pub(crate) struct Passes {
    clocks: Clocks,
    /// The first context of each binding declaring one, by its instance and
    /// the parameter it fills.
    firsts: HashMap<(String, String), Context>,
    /// Each instance's own outputs, the output of its pass `k` at `k`.
    outputs: HashMap<String, Vec<Context>>,
    /// Each router's inputs on each of its passes.
    given: HashMap<String, Vec<Vec<(String, Context)>>>,
    /// The branch each router took on each of its passes, by its position
    /// among the branches its instance declares, or none where it named
    /// nothing.
    routes: HashMap<String, Vec<Option<usize>>>,
}

impl Passes {
    /// The passes of a run of `definition` whose bindings declaring a first
    /// context give `firsts`, by instance and parameter, before anything has
    /// run.
    pub(crate) fn new(
        definition: &WorkflowDefinition,
        firsts: HashMap<(String, String), Context>,
    ) -> Self {
        Self {
            clocks: Clocks::new(definition),
            firsts,
            ..Self::default()
        }
    }

    /// Record that `activation`, an instance's own, produced `output` as its
    /// instance's next pass - and for a router's, the inputs it was given and
    /// the branch `route` names it took, none where it named nothing.
    // @An output held as its instance's next pass,IMPL_RUN_WALKS_EVERY_EDGE,impl,[CREQ_RUN_WALKS_EVERY_EDGE, CREQ_RUN_WALKS_ROUTED],[DEC_PASS_CLOCKS, DEC_ONE_GRAPH]
    pub(crate) fn produced(
        &mut self,
        activation: &Activation,
        output: &Context,
        route: Option<Option<usize>>,
    ) {
        let instance = activation.instance.clone();
        self.outputs
            .entry(instance.clone())
            .or_default()
            .push(output.clone());
        if let Some(branch) = route {
            self.given
                .entry(instance.clone())
                .or_default()
                .push(activation.inputs.clone());
            self.routes.entry(instance).or_default().push(branch);
        }
    }

    /// The latest output of `instance`, when it has produced one.
    pub(crate) fn latest(&self, instance: &str) -> Option<&Context> {
        self.outputs.get(instance).and_then(|made| made.last())
    }

    /// Whether `instance` has had an activation of its own accepted.
    pub(crate) fn has_run(&self, instance: &str) -> bool {
        self.runs(instance) > 0
    }

    /// How many passes `instance` has run.
    fn runs(&self, instance: &str) -> usize {
        self.outputs.get(instance).map_or(0, Vec::len)
    }

    /// The pass of `router` on which it took one of `branches` for the time
    /// numbered `nth` from 0, once it has.
    fn taking(&self, router: &str, branches: &[usize], nth: usize) -> Option<usize> {
        self.routes
            .get(router)?
            .iter()
            .enumerate()
            .filter(|(_, took)| took.is_some_and(|branch| branches.contains(&branch)))
            .map(|(pass, _)| pass)
            .nth(nth)
    }

    /// Pass `pass` of `from` as the pass of `to` it is part of, `to` enclosing
    /// `from`, or `None` while it cannot be told yet.
    // @A pass found on the passes enclosing it,IMPL_SCHEDULER_ENCLOSING_PASS,impl,[CREQ_SCHEDULER_READS_ENCLOSING_PASS],[DEC_PASS_CLOCKS]
    fn pass_of(&self, from: &Clock, pass: usize, to: &Clock) -> Option<usize> {
        let mut at = from.clone();
        let mut pass = pass;
        loop {
            if at == *to {
                return Some(pass);
            }
            match &at {
                Clock::Branch { router, branches } => {
                    let taken = self.taking(router, branches, pass)?;
                    if let Clock::Branch {
                        router: outside,
                        branches: more,
                    } = to
                        && outside == router
                    {
                        // The same router's passes on a set of branches this
                        // one is part of: counted among them.
                        return Some(
                            self.routes[router][..taken]
                                .iter()
                                .filter(|took| took.is_some_and(|b| more.contains(&b)))
                                .count(),
                        );
                    }
                    at = self.clocks.of(router).clone();
                    pass = taken;
                }
                Clock::Given(_) | Clock::Cycle(_) => {
                    at = Clock::Once;
                    pass = 0;
                }
                Clock::Once | Clock::Never => return None,
            }
        }
    }

    /// The context `binding` gives `instance` on its pass `pass`, or `None`
    /// while it has none: the context made on the same pass of what the binding
    /// carries, or on the pass enclosing it - and where the binding declares
    /// its first context, that on pass 0 and what it carries after.
    // @A binding's context of the activation's own pass,IMPL_SCHEDULER_ONE_PASS,impl,[CREQ_SCHEDULER_GIVES_ONE_PASS, CREQ_SCHEDULER_GIVES_FIRST],[DEC_PASS_CLOCKS, DEC_FIRST_CONTEXT_DECLARED]
    fn input(&self, instance: &NodeInstance, binding: &Binding, pass: usize) -> Option<&Context> {
        let edge = self.clocks.edge(&instance.name, &binding.parameter);
        let mine = self.clocks.of(&instance.name);
        let mut carried = self.pass_of(mine, pass, edge)?;
        if binding.first.is_some() {
            if carried == 0 {
                return self
                    .firsts
                    .get(&(instance.name.clone(), binding.parameter.clone()));
            }
            carried -= 1;
        }
        let source = binding.source.as_str();
        let branches = match edge {
            Clock::Given(inner) => match inner.as_ref() {
                Clock::Branch { router, branches } if router == source => Some(branches),
                _ => None,
            },
            Clock::Branch { router, branches } if router == source => Some(branches),
            _ => None,
        };
        match branches {
            Some(branches) => {
                let taken = self.taking(source, branches, carried)?;
                match &binding.input {
                    None => self.outputs.get(source)?.get(taken),
                    Some(input) => self
                        .given
                        .get(source)?
                        .get(taken)?
                        .iter()
                        .find(|(parameter, _)| parameter == input)
                        .map(|(_, given)| given),
                }
            }
            None => self.outputs.get(source)?.get(carried),
        }
    }
}

/// One node about to run: which instance it is for, the node type it performs,
/// the context for each parameter it is given, and the context type it is
/// declared to produce - and, for a node type a model called, which call it
/// performs. Held by value, and told apart by instance and call rather than by
/// an identifier of its own.
// @An activation told apart by instance and call,TRACE_SCHEDULER_ACTIVATION,trace,[],[DEC_RUN_IS_DRIVEN, DEC_PASS_CLOCKS, DEC_CALL_IS_AN_ACTIVATION]
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
    passes: &Passes,
) -> Option<Activation> {
    definition
        .instances
        .iter()
        .find_map(|instance| activation_for(definition, arguments, passes, instance))
}

/// The activation `instance` may have now, or `None` when it may not activate.
///
/// It is offered for its next pass once every parameter its node type
/// declares has a context of that pass: a bound one what its binding carries
/// for the pass, one nothing binds its argument. An instance on no pass is
/// never offered, and one on the run's one pass once.
// @Readiness and the activation it carries,IMPL_SCHEDULER_READY,impl,[CREQ_SCHEDULER_READY_WHEN_BOUND, CREQ_SCHEDULER_ACTIVATION_CARRIES, CREQ_SCHEDULER_OFFERS_AGAIN],[DEC_EVERY_INPUT_REQUIRED, DEC_SIGNATURE_IS_WHAT_NOTHING_BINDS, DEC_PASS_CLOCKS]
pub(crate) fn activation_for(
    definition: &WorkflowDefinition,
    arguments: &Arguments,
    passes: &Passes,
    instance: &NodeInstance,
) -> Option<Activation> {
    let declared = definition
        .node_types
        .iter()
        .find(|declared| declared.name == instance.node_type)?;
    let pass = passes.runs(&instance.name);
    match passes.clocks.of(&instance.name) {
        Clock::Never => return None,
        Clock::Once if pass > 0 => return None,
        _ => {}
    }

    let mut inputs = Vec::new();
    for parameter in &declared.required {
        let context = match instance
            .bindings
            .iter()
            .find(|b| b.parameter == parameter.name)
        {
            Some(binding) => passes.input(instance, binding, pass)?,
            None => arguments.context_for(&instance.name, &parameter.name)?,
        };
        inputs.push((parameter.name.clone(), context.clone()));
    }

    Some(Activation {
        instance: instance.name.clone(),
        node_type: declared.name.clone(),
        call: None,
        inputs,
        output: declared.output.clone(),
    })
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
impl Passes {
    /// `output` as the next pass of `instance`, which does not route.
    fn walked(&mut self, instance: &str, output: &Context) {
        self.outputs
            .entry(instance.to_owned())
            .or_default()
            .push(output.clone());
    }
}

/// The passes of a run of `workflow` given no argument, in which each instance
/// `produced` names has produced its context once, in the definition's order.
#[cfg(test)]
fn passes_of(workflow: &WorkflowDefinition, produced: &Produced) -> Passes {
    let mut passes = Passes::new(workflow, HashMap::new());
    for node in &workflow.instances {
        if let Some(output) = produced.get(&node.name) {
            passes.walked(&node.name, output);
        }
    }
    passes
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
/// either happens.
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
    let mut passes = Passes::new(workflow, HashMap::new());
    let mut answers = Vec::new();

    while let Some(activation) = next_activation(workflow, &arguments, &passes) {
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
        passes.produced(&activation, &context, None);
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
    assert!(
        activation_for(
            &workflow,
            &arguments,
            &passes_of(&workflow, &produced),
            sink
        )
        .is_none()
    );

    // Once it arrives the instance is offered, and the waiting was for
    // something: both contexts are carried, in declared order.
    let may = ctx(&mut source, "note");
    let may_id = may.id();
    produced.insert("b".to_owned(), may);
    let activation = activation_for(
        &workflow,
        &arguments,
        &passes_of(&workflow, &produced),
        sink,
    )
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
            &passes_of(&workflow, &produced),
            &workflow.instances[1]
        )
        .is_none()
    );
    // The control: bound in full, the same source is enough.
    let activation = activation_for(
        &workflow,
        &Arguments::new(),
        &passes_of(&workflow, &produced),
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
        &Passes::new(&workflow, HashMap::new()),
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
            &passes_of(&workflow, &produced)
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
            &passes_of(&workflow, &Produced::new())
        )
        .is_some()
    );

    // Having produced, it is not offered again, and with nothing else to offer
    // the scheduler says so rather than cycling over it for ever.
    let produced = produced_by(&mut source, &["a"], "note");
    assert!(next_activation(&workflow, &arguments, &passes_of(&workflow, &produced)).is_none());
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
        &passes_of(&workflow, &produced),
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
        &passes_of(&workflow, &produced),
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
        &passes_of(&workflow, &produced),
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
            &passes_of(&workflow, &produced)
        )
        .is_none()
    );
}

/// The activation `name` may have now in `workflow`.
#[cfg(test)]
fn offered(
    workflow: &WorkflowDefinition,
    arguments: &Arguments,
    passes: &Passes,
    name: &str,
) -> Option<Activation> {
    let node = workflow
        .instances
        .iter()
        .find(|node| node.name == name)
        .expect("the instance is in the workflow");
    activation_for(workflow, arguments, passes, node)
}

#[cfg(test)]
#[test]
fn offered_once_per_pass() {
    let mut source = IdSource::new();
    // `x` reads its own output, declaring the first; `c` reads `x` and
    // `v`, which reads nothing and runs once.
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Take", &[("seed", "note")], "note"),
        node_type("Pair", &[("left", "note"), ("right", "note")], "note"),
    ];
    let instances = vec![
        instance("v", "Src", &[]),
        instance("x", "Take", &[("seed", "x")]).first("seed", "x"),
        instance("c", "Pair", &[("left", "v"), ("right", "x")]),
    ];
    let workflow = definition(types, instances, &["c"]);
    let seed = ctx(&mut source, "note");
    let arguments = Arguments::new();
    let firsts = HashMap::from([(("x".to_owned(), "seed".to_owned()), seed.clone())]);
    let mut passes = Passes::new(&workflow, firsts);

    // An instance reading nothing is offered once, and never again.
    let v = offered(&workflow, &arguments, &passes, "v").expect("nothing to wait for");
    let v0 = ctx(&mut source, "note");
    passes.produced(&v, &v0, None);
    for _ in 0..3 {
        assert!(offered(&workflow, &arguments, &passes, "v").is_none());
    }

    // `x` on its first pass is given its declared first; `c` waits for it.
    assert!(offered(&workflow, &arguments, &passes, "c").is_none());
    let x = offered(&workflow, &arguments, &passes, "x").expect("given its first");
    assert!(x.inputs()[0].1.is(&seed));
    let x0 = ctx(&mut source, "note");
    passes.produced(&x, &x0, None);

    // `c`'s first pass: then, with nothing of a second pass, not offered
    // however often it is asked.
    let c = offered(&workflow, &arguments, &passes, "c").expect("its first pass");
    assert!(c.inputs()[0].1.is(&v0) && c.inputs()[1].1.is(&x0));
    passes.produced(&c, &ctx(&mut source, "note"), None);
    for _ in 0..3 {
        assert!(offered(&workflow, &arguments, &passes, "c").is_none());
    }

    // `x`'s second pass is given its first's output, and `c`'s second pass the
    // same `v` and `x`'s second.
    let x = offered(&workflow, &arguments, &passes, "x").expect("its second pass");
    assert!(x.inputs()[0].1.is(&x0));
    let x1 = ctx(&mut source, "note");
    passes.produced(&x, &x1, None);
    let c = offered(&workflow, &arguments, &passes, "c").expect("its second pass");
    assert!(c.inputs()[0].1.is(&v0) && c.inputs()[1].1.is(&x1));
}

#[cfg(test)]
use crate::clock::Unpaired;

/// `workflow` with the bindings of `given`, each an instance and a parameter,
/// declaring an empty first context.
#[cfg(test)]
fn declaring(workflow: &WorkflowDefinition, given: &[(&str, &str)]) -> WorkflowDefinition {
    let mut declared = workflow.clone();
    for &(instance, parameter) in given {
        let at = declared
            .instances
            .iter()
            .position(|node| node.name == instance)
            .expect("the instance is in the workflow");
        declared.instances[at] = declared.instances[at].clone().first(parameter, "");
    }
    declared
}

/// The instances of `workflow` whose inputs share no pass, with the bindings
/// of `given` declaring their first contexts, each paired with a
/// description of the passes it runs on.
#[cfg(test)]
fn unpaired_of(workflow: &WorkflowDefinition, given: &[(&str, &str)]) -> Vec<Unpaired> {
    Clocks::new(&declaring(workflow, given)).unpaired().to_vec()
}

/// A review loop: `d` reads a brief and what the router `r` sends back, with
/// no first context declared; `r` reads the draft and names `branches`; then
/// whatever `more` adds.
#[cfg(test)]
fn looping(branches: &[(&str, &[&str])], more: Vec<NodeInstance>) -> WorkflowDefinition {
    let types = vec![
        node_type("Src", &[], "note"),
        node_type("Draft", &[("brief", "note"), ("feedback", "note")], "note"),
        node_type("Route", &[("draft", "note")], "note").routing(),
        node_type("Take", &[("input", "note")], "note"),
        node_type("Pair", &[("left", "note"), ("right", "note")], "note"),
    ];
    let mut instances = vec![
        instance("b", "Src", &[]),
        instance("d", "Draft", &[("brief", "b"), ("feedback", "r")]),
        instance("r", "Route", &[("draft", "d")]).branching(branches),
    ];
    instances.extend(more);
    definition(types, instances, &["b"])
}

#[cfg(test)]
#[test]
fn unpaired_inputs_reported() {
    // Sound, each: a review loop sending the draft on; a node on a branch
    // reading one made on every pass, and one made once; two instances on one
    // branch; an instance in two branches read beside one in one of them; a
    // cycle no router is on, declaring its first context; a loop declaring
    // none, whose instances run on no pass and are reported for nothing.
    let sound = looping(
        &[
            ("back", &["d", "w"]),
            ("both", &["d", "y", "w"]),
            ("on", &["y", "f"]),
        ],
        vec![
            instance("x", "Take", &[("input", "d")]),
            instance("y", "Take", &[]).taking("input", "r", "draft"),
            instance("f", "Take", &[]).taking("input", "r", "draft"),
            instance("w", "Take", &[]).taking("input", "r", "draft"),
            instance("j", "Pair", &[("left", "x"), ("right", "y")]),
            instance("k", "Pair", &[("left", "b"), ("right", "y")]),
            instance("l", "Pair", &[("left", "y"), ("right", "f")]),
            instance("n", "Pair", &[("left", "w"), ("right", "y")]),
            instance("z", "Take", &[("input", "z")]),
        ],
    );
    assert_eq!(
        unpaired_of(&sound, &[("d", "feedback"), ("z", "input")]),
        []
    );
    let clocks = Clocks::new(&declaring(&sound, &[("d", "feedback"), ("z", "input")]));
    let on = |branches: &[usize]| Clock::Branch {
        router: "r".to_owned(),
        branches: branches.to_vec(),
    };
    // A branch's node reading one of every pass, and one made once, runs on
    // the branch's passes; two sets of one router's branches, on the passes
    // of the branch both name.
    assert_eq!(clocks.of("j"), &on(&[1, 2]));
    assert_eq!(clocks.of("k"), &on(&[1, 2]));
    assert_eq!(clocks.of("l"), &on(&[2]));
    assert_eq!(clocks.of("n"), &on(&[1]));
    assert_eq!(clocks.of("z"), &Clock::Cycle("z".to_owned()));
    let clocks = Clocks::new(&sound);
    assert_eq!(clocks.of("d"), &Clock::Never);
    assert_eq!(clocks.of("j"), &Clock::Never);
    assert_eq!(unpaired_of(&sound, &[]), []);

    // A node joining two branches of one router.
    let siblings = looping(
        &[("back", &["d"]), ("ya", &["d", "y"]), ("za", &["d", "z"])],
        vec![
            instance("y", "Take", &[]).taking("input", "r", "draft"),
            instance("z", "Take", &[]).taking("input", "r", "draft"),
            instance("j", "Pair", &[("left", "y"), ("right", "z")]),
        ],
    );
    assert_eq!(
        unpaired_of(&siblings, &[("d", "feedback")]),
        [Unpaired {
            instance: "j".to_owned(),
            inputs: vec![
                (
                    "left".to_owned(),
                    "the passes on which 'r' takes 'ya'".to_owned()
                ),
                (
                    "right".to_owned(),
                    "the passes on which 'r' takes 'za'".to_owned()
                ),
            ],
        }]
    );

    // The loop's drafter, on every pass of the loop, also reading one made on
    // a branch's passes alone.
    let mut slower = looping(
        &[("back", &["d"]), ("side", &["d", "s"])],
        vec![instance("s", "Take", &[]).taking("input", "r", "draft")],
    );
    slower.node_types.push(node_type(
        "Draft3",
        &[("brief", "note"), ("feedback", "note"), ("aside", "note")],
        "note",
    ));
    slower.instances[1] = instance(
        "d",
        "Draft3",
        &[("brief", "b"), ("feedback", "r"), ("aside", "s")],
    );
    let found = unpaired_of(&slower, &[("d", "feedback")]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].instance, "d");
    assert_eq!(
        found[0].inputs,
        [
            ("brief".to_owned(), "the run's one pass".to_owned()),
            (
                "feedback".to_owned(),
                "a first context declared, then the passes on which 'r' takes 'back' or 'side'"
                    .to_owned()
            ),
            (
                "aside".to_owned(),
                "the passes on which 'r' takes 'side'".to_owned()
            ),
        ]
    );
    assert!(
        found[0]
            .to_string()
            .starts_with("the inputs of 'd' come on passes no one of which"),
        "{}",
        found[0]
    );

    // Cycles no router is on: one of two instances, declaring its first context,
    // read by a third, which runs on its passes; and two such cycles, each of
    // one instance, joined, which share none.
    let types = vec![
        node_type("Take", &[("input", "note")], "note"),
        node_type("Pair", &[("left", "note"), ("right", "note")], "note"),
    ];
    let cycle = definition(
        types.clone(),
        vec![
            instance("a", "Take", &[("input", "b")]),
            instance("b", "Take", &[("input", "a")]),
            instance("c", "Pair", &[("left", "a"), ("right", "b")]),
        ],
        &["c"],
    );
    assert_eq!(unpaired_of(&cycle, &[("a", "input")]), []);
    let clocks = Clocks::new(&declaring(&cycle, &[("a", "input")]));
    let through_a = Clock::Cycle("a".to_owned());
    assert_eq!(
        [clocks.of("a"), clocks.of("b"), clocks.of("c")],
        [&through_a, &through_a, &through_a]
    );
    let two = definition(
        types,
        vec![
            instance("x", "Take", &[("input", "x")]),
            instance("y", "Take", &[("input", "y")]),
            instance("j", "Pair", &[("left", "x"), ("right", "y")]),
        ],
        &["j"],
    );
    assert_eq!(
        unpaired_of(&two, &[("x", "input"), ("y", "input")]),
        [Unpaired {
            instance: "j".to_owned(),
            inputs: vec![
                (
                    "left".to_owned(),
                    "the passes of the cycle through 'x'".to_owned()
                ),
                (
                    "right".to_owned(),
                    "the passes of the cycle through 'y'".to_owned()
                ),
            ],
        }]
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
            let Some(activation) = activation_for(&workflow, &Arguments::new(), &passes_of(&workflow, &produced), node)
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

        let offered = next_activation(&workflow, &arguments, &passes_of(&workflow, &produced));
        let any_offerable = workflow
            .instances
            .iter()
            .any(|node| activation_for(&workflow, &arguments, &passes_of(&workflow, &produced), node).is_some());
        prop_assert_eq!(offered.is_some(), any_offerable);

        if let Some(activation) = offered {
            // Some instance carrying that name is offerable.
            prop_assert!(
                workflow
                    .instances
                    .iter()
                    .filter(|node| node.name == activation.instance())
                    .any(|node| activation_for(&workflow, &arguments, &passes_of(&workflow, &produced), node).is_some())
            );
        }
    }
}
