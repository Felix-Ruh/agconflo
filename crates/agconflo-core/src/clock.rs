//! The passes each instance of a workflow runs on, worked out from its wiring
//! and the first contexts its bindings declare, before anything runs; and
//! every instance whose inputs come on passes no one of which encloses the
//! rest.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::workflow::{NodeInstance, WorkflowDefinition};

/// The passes an instance runs on, or a binding carries contexts on.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Clock {
    /// The run's one pass.
    Once,
    /// No pass: a cycle no declared first context starts, and whatever reads
    /// from it.
    Never,
    /// The passes of `router` on which it takes one of `branches`, by their
    /// position among the branches its instance declares, in ascending order.
    Branch {
        router: String,
        branches: Vec<usize>,
    },
    /// Pass 0 the first context a binding declares, then each pass of the
    /// clock inside.
    Given(Box<Clock>),
    /// The passes of a cycle no router is on, named by its first instance in
    /// the definition's order.
    Cycle(String),
}

/// An instance whose inputs come on passes no one of which encloses the rest,
/// so no activation of it could be given one pass's contexts.
// @An instance whose inputs cannot be paired as values,TRACE_CLOCK_UNPAIRED,trace,[],[DEC_PAIRING_IS_WIRING, DEC_FAILURES_NON_EXHAUSTIVE]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unpaired {
    /// The instance.
    pub instance: String,
    /// Each parameter a binding fills, with the passes its contexts come on,
    /// in the order the instance binds them.
    pub inputs: Vec<(String, String)>,
}

impl fmt::Display for Unpaired {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inputs: Vec<String> = self
            .inputs
            .iter()
            .map(|(parameter, passes)| format!("'{parameter}' on {passes}"))
            .collect();
        write!(
            f,
            "the inputs of '{}' come on passes no one of which encloses the others, so no activation of it could be given one pass's contexts: {}",
            self.instance,
            inputs.join(", ")
        )
    }
}

impl std::error::Error for Unpaired {}

/// The passes of every instance of one definition.
#[derive(Clone, Debug, Default)]
pub(crate) struct Clocks {
    /// Each instance's passes, by name.
    of: HashMap<String, Clock>,
    /// The passes each binding carries contexts on, by its instance and the
    /// parameter it fills.
    edges: HashMap<(String, String), Clock>,
    /// Every instance whose inputs cannot be paired, in the definition's order.
    unpaired: Vec<Unpaired>,
}

impl Clocks {
    /// The passes of every instance of `definition`.
    ///
    /// An instance runs on the passes of the input that comes most often,
    /// every other input coming on passes enclosing those, and one with no
    /// input a binding fills runs once. A binding declaring its first context
    /// has it on pass 0 and what it carries after. Instances on a cycle no
    /// router is on share the cycle's passes when a binding on it declares its
    /// first context, and have none otherwise; so does whatever reads from an
    /// instance that has none.
    // @Each instance's passes from its wiring,IMPL_CLOCK_PASSES,impl,[CREQ_SCHEDULER_REPORTS_UNPAIRED],[DEC_PASS_CLOCKS, DEC_FIRST_CONTEXT_DECLARED]
    pub(crate) fn new(definition: &WorkflowDefinition) -> Self {
        let mut work = Work::new(definition);
        for instance in &definition.instances {
            work.clock(&instance.name);
        }
        work.settle();
        work.finish()
    }

    /// The passes `instance` runs on: none for an instance the definition
    /// does not carry.
    pub(crate) fn of(&self, instance: &str) -> &Clock {
        self.of.get(instance).unwrap_or(&Clock::Never)
    }

    /// The passes the binding filling `parameter` of `instance` carries
    /// contexts on.
    pub(crate) fn edge(&self, instance: &str, parameter: &str) -> &Clock {
        self.edges
            .get(&(instance.to_owned(), parameter.to_owned()))
            .unwrap_or(&Clock::Never)
    }

    /// Every instance whose inputs cannot be paired.
    pub(crate) fn unpaired(&self) -> &[Unpaired] {
        &self.unpaired
    }
}

/// The clocks of one definition being worked out.
struct Work<'d> {
    definition: &'d WorkflowDefinition,
    /// The instances whose node type routes.
    routers: HashSet<&'d str>,
    /// For each instance on a cycle no router is on, the cycle's first
    /// instance and its passes.
    cycles: HashMap<&'d str, (&'d str, Clock)>,
    /// Each instance's clock, once worked out.
    of: HashMap<String, Clock>,
    /// Each binding's clock, once its instance's is worked out.
    edges: HashMap<(String, String), Clock>,
    /// The instances being worked out, innermost last.
    visiting: Vec<String>,
    /// The outermost of those reached again while being worked out: what is
    /// worked out beneath it until it is done is provisional.
    reached_again: Option<usize>,
    /// Instances whose inputs cannot be paired, with each binding's passes.
    unpaired: HashMap<String, Vec<(String, Clock)>>,
}

impl<'d> Work<'d> {
    fn new(definition: &'d WorkflowDefinition) -> Self {
        let routers = definition
            .instances
            .iter()
            .filter(|instance| {
                definition
                    .node_types
                    .iter()
                    .any(|declared| declared.name == instance.node_type && declared.routes)
            })
            .map(|instance| instance.name.as_str())
            .collect();
        let mut work = Self {
            definition,
            routers,
            cycles: HashMap::new(),
            of: HashMap::new(),
            edges: HashMap::new(),
            visiting: Vec::new(),
            reached_again: None,
            unpaired: HashMap::new(),
        };
        work.cycles = work.find_cycles();
        work
    }

    /// The instance carrying `name`, the first when several do.
    fn instance(&self, name: &str) -> Option<&'d NodeInstance> {
        self.definition.instances.iter().find(|i| i.name == name)
    }

    /// Whether `instance` takes `binding`'s source's output from an instance
    /// that does not route: a binding a cycle no router is on can be made of.
    fn plain(&self, binding: &crate::workflow::Binding) -> bool {
        binding.input.is_none() && !self.routers.contains(binding.source.as_str())
    }

    /// Every instance `from`'s output reaches through such bindings.
    fn reach(&self, from: &str) -> HashSet<&'d str> {
        let mut reached = HashSet::new();
        let mut pending = vec![from.to_owned()];
        while let Some(at) = pending.pop() {
            for consumer in &self.definition.instances {
                if consumer
                    .bindings
                    .iter()
                    .any(|b| self.plain(b) && b.source == at)
                    && reached.insert(consumer.name.as_str())
                {
                    pending.push(consumer.name.clone());
                }
            }
        }
        reached
    }

    /// The cycles no router is on: each set of instances reaching one another
    /// through bindings from instances that do not route, one instance reading
    /// its own output among them. A cycle has passes of its own when a binding
    /// between two of its instances declares its first context, and none
    /// otherwise.
    // @Cycles no router is on found,IMPL_CLOCK_CYCLES,impl,[CREQ_SCHEDULER_REPORTS_UNPAIRED],[DEC_PASS_CLOCKS]
    fn find_cycles(&self) -> HashMap<&'d str, (&'d str, Clock)> {
        let mut cycles: HashMap<&'d str, (&'d str, Clock)> = HashMap::new();
        for first in &self.definition.instances {
            let first = first.name.as_str();
            if cycles.contains_key(first) {
                continue;
            }
            let reached = self.reach(first);
            if !reached.contains(first) {
                continue;
            }
            let members: Vec<&'d str> = self
                .definition
                .instances
                .iter()
                .map(|i| i.name.as_str())
                .filter(|&member| reached.contains(member) && self.reach(member).contains(first))
                .collect();
            let given = members.iter().any(|&member| {
                self.instance(member).is_some_and(|instance| {
                    instance.bindings.iter().any(|b| {
                        self.plain(b) && members.contains(&b.source.as_str()) && b.first.is_some()
                    })
                })
            });
            let clock = if given {
                Clock::Cycle(first.to_owned())
            } else {
                Clock::Never
            };
            for member in members {
                cycles.insert(member, (first, clock.clone()));
            }
        }
        cycles
    }

    /// The passes the binding filling `parameter` of `instance` carries
    /// contexts on.
    fn edge(&mut self, instance: &NodeInstance, parameter: &str) -> Clock {
        let Some(binding) = instance.bindings.iter().find(|b| b.parameter == parameter) else {
            return Clock::Once;
        };
        let source = binding.source.as_str();
        if self.instance(source).is_none() {
            return Clock::Never;
        }
        if self.plain(binding)
            && let (Some((mine, clock)), Some((theirs, _))) = (
                self.cycles.get(instance.name.as_str()),
                self.cycles.get(source),
            )
            && mine == theirs
        {
            // A binding within the cycle carries the cycle's passes, its
            // first context among them.
            return clock.clone();
        }
        let inner = if self.routers.contains(source) {
            let branches = self.instance(source).map_or_else(Vec::new, |router| {
                router
                    .branches
                    .iter()
                    .enumerate()
                    .filter(|(_, branch)| branch.instances.contains(&instance.name))
                    .map(|(position, _)| position)
                    .collect()
            });
            Clock::Branch {
                router: source.to_owned(),
                branches,
            }
        } else {
            self.clock(source)
        };
        if binding.first.is_some() {
            Clock::Given(Box::new(inner))
        } else {
            inner
        }
    }

    /// The passes `name` runs on, worked out on first asking.
    fn clock(&mut self, name: &str) -> Clock {
        if let Some(clock) = self.of.get(name) {
            return clock.clone();
        }
        let Some(instance) = self.instance(name) else {
            return Clock::Never;
        };
        if let Some(depth) = self.visiting.iter().position(|visiting| visiting == name) {
            // Reached again while its own passes are being worked out, through
            // a router whose passes are its own: no pass of it can be part of
            // its own, and what was worked out on the way here is provisional.
            self.reached_again = Some(self.reached_again.map_or(depth, |at| at.min(depth)));
            return Clock::Never;
        }
        let depth = self.visiting.len();
        self.visiting.push(name.to_owned());

        let mut inputs: Vec<(String, Clock)> = Vec::new();
        for binding in &instance.bindings {
            if inputs
                .iter()
                .any(|(parameter, _)| *parameter == binding.parameter)
            {
                continue;
            }
            let clock = self.edge(instance, &binding.parameter);
            inputs.push((binding.parameter.clone(), clock));
        }

        let clock = if inputs.is_empty() {
            Clock::Once
        } else if inputs.iter().any(|(_, clock)| *clock == Clock::Never) {
            Clock::Never
        } else {
            let mut found = None;
            for (_, candidate) in &inputs {
                let mut all = true;
                for (_, other) in &inputs {
                    if !self.encloses(other, candidate) {
                        all = false;
                        break;
                    }
                }
                if all {
                    found = Some(candidate.clone());
                    break;
                }
            }
            let found = match found {
                Some(clock) => Some(clock),
                None => self.shared(&inputs),
            };
            match found {
                Some(clock) => clock,
                None => {
                    // Refused when the run starts: the first input's passes
                    // keep what reads it from stalling and hiding the refusal.
                    if self.reached_again.is_none_or(|at| at >= depth) {
                        self.unpaired.insert(name.to_owned(), inputs.clone());
                    }
                    inputs
                        .iter()
                        .map(|(_, clock)| clock.clone())
                        .find(|clock| *clock != Clock::Once)
                        .unwrap_or(Clock::Once)
                }
            }
        };

        self.visiting.pop();
        match self.reached_again {
            Some(at) if at < depth => return clock,
            Some(at) if at == depth => self.reached_again = None,
            _ => {}
        }
        self.of.insert(name.to_owned(), clock.clone());
        for (parameter, edge) in inputs {
            self.edges.insert((name.to_owned(), parameter), edge);
        }
        clock
    }

    /// The passes every one of `inputs` comes on, where no input's passes
    /// enclose all the others': the passes on which one router takes a branch
    /// in every set of its branches the inputs name, when there is such a
    /// branch and every input's passes enclose those.
    // @The passes inputs from one router's branches share,IMPL_CLOCK_SHARED,impl,[CREQ_SCHEDULER_REPORTS_UNPAIRED],[DEC_PASS_CLOCKS]
    fn shared(&mut self, inputs: &[(String, Clock)]) -> Option<Clock> {
        let mut routers: Vec<(&str, Vec<usize>)> = Vec::new();
        for (_, clock) in inputs {
            if let Clock::Branch { router, branches } = clock {
                match routers.iter_mut().find(|(r, _)| *r == router) {
                    Some((_, common)) => common.retain(|branch| branches.contains(branch)),
                    None => routers.push((router, branches.clone())),
                }
            }
        }
        let [(router, common)] = routers.as_slice() else {
            return None;
        };
        if common.is_empty() {
            return None;
        }
        let candidate = Clock::Branch {
            router: (*router).to_owned(),
            branches: common.clone(),
        };
        for (_, clock) in inputs {
            if !self.encloses(clock, &candidate) {
                return None;
            }
        }
        Some(candidate)
    }

    /// The clock each pass of `clock` is part of a pass of, or `None` for the
    /// run's one pass and for no pass.
    fn parent(&mut self, clock: &Clock) -> Option<Clock> {
        match clock {
            Clock::Once | Clock::Never => None,
            Clock::Given(_) | Clock::Cycle(_) => Some(Clock::Once),
            Clock::Branch { router, .. } => {
                let router = router.clone();
                Some(self.clock(&router))
            }
        }
    }

    /// Whether every pass of `inner` is part of a pass of `outer`: the run's
    /// one pass encloses every other, a router's passes its branches', and a
    /// set of a router's branches each set it is part of.
    fn encloses(&mut self, outer: &Clock, inner: &Clock) -> bool {
        if *inner == Clock::Never {
            return false;
        }
        if *outer == Clock::Once {
            return true;
        }
        let mut seen: Vec<Clock> = Vec::new();
        let mut at = inner.clone();
        loop {
            if at == *outer {
                return true;
            }
            if let (
                Clock::Branch {
                    router: inside,
                    branches: some,
                },
                Clock::Branch {
                    router: outside,
                    branches: more,
                },
            ) = (&at, outer)
                && inside == outside
                && some.iter().all(|branch| more.contains(branch))
            {
                return true;
            }
            if seen.contains(&at) {
                return false;
            }
            seen.push(at.clone());
            match self.parent(&at) {
                Some(parent) => at = parent,
                None => return false,
            }
        }
    }

    /// Whether `clock` has a first pass: whether following the passes each of
    /// its passes is part of reaches the run's one pass.
    fn starts(&self, clock: &Clock) -> bool {
        let mut seen: Vec<&str> = Vec::new();
        let mut at = clock;
        loop {
            match at {
                Clock::Once | Clock::Given(_) | Clock::Cycle(_) => return true,
                Clock::Never => return false,
                Clock::Branch { router, .. } => {
                    if seen.contains(&router.as_str()) {
                        return false;
                    }
                    seen.push(router);
                    at = self.of.get(router).unwrap_or(&Clock::Never);
                }
            }
        }
    }

    /// No passes for every instance whose own passes never start, or that
    /// reads through a binding whose passes never start, until nothing
    /// changes.
    fn settle(&mut self) {
        loop {
            let stalled: Vec<String> = self
                .of
                .iter()
                .filter(|(name, clock)| {
                    **clock != Clock::Never
                        && (!self.starts(clock)
                            || self
                                .edges
                                .iter()
                                .any(|((at, _), edge)| at == *name && !self.starts(edge)))
                })
                .map(|(name, _)| name.clone())
                .collect();
            if stalled.is_empty() {
                return;
            }
            for name in stalled {
                self.of.insert(name, Clock::Never);
            }
        }
    }

    /// The clocks worked out, and each instance whose inputs cannot be paired
    /// and every one of them has a first pass, in the definition's order.
    fn finish(self) -> Clocks {
        let mut unpaired = Vec::new();
        for instance in &self.definition.instances {
            let Some(inputs) = self.unpaired.get(&instance.name) else {
                continue;
            };
            if !inputs.iter().all(|(_, clock)| self.starts(clock)) {
                continue;
            }
            unpaired.push(Unpaired {
                instance: instance.name.clone(),
                inputs: inputs
                    .iter()
                    .map(|(parameter, clock)| (parameter.clone(), self.describe(clock)))
                    .collect(),
            });
        }
        Clocks {
            of: self.of,
            edges: self.edges,
            unpaired,
        }
    }

    /// `clock` in words, naming instances and branches as the definition does.
    fn describe(&self, clock: &Clock) -> String {
        match clock {
            Clock::Once => "the run's one pass".to_owned(),
            Clock::Never => "no pass".to_owned(),
            Clock::Given(inner) => {
                format!("a first context declared, then {}", self.describe(inner))
            }
            Clock::Cycle(first) => format!("the passes of the cycle through '{first}'"),
            Clock::Branch { router, branches } => {
                let named: Vec<String> = self.instance(router).map_or_else(Vec::new, |router| {
                    branches
                        .iter()
                        .filter_map(|&position| router.branches.get(position))
                        .map(|branch| format!("'{}'", branch.name))
                        .collect()
                });
                format!(
                    "the passes on which '{router}' takes {}",
                    named.join(" or ")
                )
            }
        }
    }
}
