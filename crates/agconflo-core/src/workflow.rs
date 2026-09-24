//! What a workflow is made of: the node types it names, the instances wired
//! into them, and the outputs it designates.
//!
//! Data rather than behaviour. `ARCH_WIRING` allocates no requirement to these
//! types - they are what the validator reads, and the other layer it reads them
//! against (`DEC_TWO_LAYERS`) - so nothing here carries a trace marker and
//! nothing here is tested on its own. What holds them to their shape is the
//! validator's cases.
//!
//! Every field is public and nothing is checked on the way in, which is the
//! opposite of how a `Context` is built and deliberately so. A definition has
//! to be able to hold every malformed shape there is, or the defects the
//! validator exists to report could not be written down: a binding naming an
//! instance that was deleted, an instance of a type nobody supplied, a
//! designated output naming nothing, two instances sharing a name. A
//! constructor refusing them would move the refusal to where only the first
//! defect is ever seen, which is what `FEAT_WIRING_ALL_DEFECTS` rules out.

use crate::ContextType;

/// One parameter a node type declares: its name, and the context type it is
/// declared for.
///
/// Ordered within its list and typed, which is what lets a node assemble its
/// own inputs rather than take them in the order edges arrive
/// (`DEC_DECLARED_PARAMETERS`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter {
    /// The name a binding uses to reach this parameter.
    pub name: String,
    /// The context type a value arriving here is declared to be.
    pub context_type: ContextType,
}

/// What one kind of node consumes and produces, declared once and instantiated
/// as often as a workflow likes (`DEC_TWO_LAYERS`).
///
/// Three declared lists rather than one, and each is load-bearing
/// (`DEC_DECLARED_PARAMETERS`): `required` is what a workflow is checked
/// against, `optional` is what lets a node be useful with less than everything
/// wired, and `globals` are the context types this kind of node reads by
/// declaration rather than through a wire.
///
/// One output, and it is typed: a node produces exactly one thing
/// (`STKH_ONE_OUTPUT`), and the type declared for it is what a consuming
/// parameter is compared against without running anything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeType {
    /// The name an instance names to reach this declaration.
    pub name: String,
    /// Parameters a node of this type cannot run without.
    pub required: Vec<Parameter>,
    /// Parameters a node of this type uses when they are wired.
    pub optional: Vec<Parameter>,
    /// Context types read by declaration rather than through a binding.
    pub globals: Vec<ContextType>,
    /// The context type of the one output a node of this type produces.
    pub output: ContextType,
}

/// One parameter of one instance, wired to the output of a named instance.
///
/// Both ends are names the definition carries (`DEC_BINDING_BY_PORT`):
/// `parameter` is declared by the consuming instance's node type, and `source`
/// is the instance whose one output arrives there. Names rather than handles,
/// deliberately - a wire to nowhere is a defect to report rather than a value
/// nobody can build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    /// The parameter of the consuming instance this binding fills.
    pub parameter: String,
    /// The instance whose output is wired to it.
    pub source: String,
}

/// One node in a workflow: its name, the type it instantiates, and what is
/// wired into it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeInstance {
    /// What bindings elsewhere name to reach this instance.
    pub name: String,
    /// The node type this is an instance of.
    pub node_type: String,
    /// Whether this is an entry node, whose parameters are the workflow's own
    /// rather than wires (`DEC_WORKFLOW_SIGNATURE`).
    ///
    /// A flag rather than a list of entry instance names on the definition, and
    /// rather than a second kind of binding source, because either of those is
    /// a fourth kind of name that can resolve to nothing - and what a validator
    /// owes an unresolved entry name is a question no requirement answers yet.
    /// Nothing is resolved here, so nothing can dangle: the workflow's typed
    /// parameter list is the parameters declared by the types its entry
    /// instances name.
    pub entry: bool,
    /// The bindings that fill this instance's parameters.
    pub bindings: Vec<Binding>,
    /// The node types a model performing this instance may call, in the order
    /// the workflow lists them (`DEC_CALLS_DECLARED_ON_THE_INSTANCE`).
    ///
    /// On the instance rather than on its node type, because what a node may
    /// call is wiring, and one node type used in two workflows may call
    /// different things in each. Names rather than handles, as a binding's are:
    /// a call to a node type nobody supplied is a defect to report
    /// (`CREQ_VALIDATOR_CALL_RESOLVES`). A name listed twice is kept twice, as
    /// written.
    pub calls: Vec<String>,
}

/// A workflow definition: the node types it carries, the instances wired into
/// it, and the outputs it designates.
///
/// `designated_outputs` holds instance names, and holds as many as it was
/// given. None and several are exactly the shapes `CREQ_VALIDATOR_ONE_OUTPUT`
/// refuses, and a designation naming no instance is what
/// `CREQ_VALIDATOR_OUTPUT_RESOLVES` refuses: all of them have to be
/// representable here to be checkable there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkflowDefinition {
    /// What this definition is called, which is the place a signature defect
    /// concerns.
    pub name: String,
    /// The declarations this definition's instances are checked against.
    pub node_types: Vec<NodeType>,
    /// The nodes wired into this workflow, in the order it carries them.
    pub instances: Vec<NodeInstance>,
    /// The instances whose output this workflow designates as its own.
    pub designated_outputs: Vec<String>,
}

// --- test builders -----------------------------------------------------------
// Used by the validator's tests and by the defect's, so they live beside the
// types they build rather than in either module. They are not tests themselves:
// this module has none, because every test in the crate is imported as a run
// naming the case it ran, and the cases are per component.

/// A context type, from a name the caller knows is not empty.
#[cfg(test)]
pub(crate) fn context_type(name: &str) -> ContextType {
    ContextType::new(name).expect("a non-empty type name is accepted")
}

/// A node type requiring `required` and producing `output`, with no optional
/// parameters and no globals. Each parameter is `(name, context type)`.
#[cfg(test)]
pub(crate) fn node_type(name: &str, required: &[(&str, &str)], output: &str) -> NodeType {
    NodeType {
        name: name.to_owned(),
        required: parameters(required),
        optional: Vec::new(),
        globals: Vec::new(),
        output: context_type(output),
    }
}

#[cfg(test)]
impl NodeType {
    /// The same declaration, also accepting `optional`.
    pub(crate) fn with_optional(mut self, optional: &[(&str, &str)]) -> Self {
        self.optional = parameters(optional);
        self
    }

    /// The same declaration, also reading `globals` by declaration.
    pub(crate) fn with_globals(mut self, globals: &[&str]) -> Self {
        self.globals = globals.iter().map(|&name| context_type(name)).collect();
        self
    }
}

#[cfg(test)]
impl NodeInstance {
    /// The same instance, as an entry node: its parameters are the workflow's
    /// own rather than wires.
    pub(crate) fn into_entry(mut self) -> Self {
        self.entry = true;
        self
    }

    /// The same instance, also declaring calls to `calls`.
    pub(crate) fn with_calls(mut self, calls: &[&str]) -> Self {
        self.calls = calls.iter().map(|&name| name.to_owned()).collect();
        self
    }
}

/// Parameters from `(name, context type)` pairs, in the order given.
#[cfg(test)]
pub(crate) fn parameters(declared: &[(&str, &str)]) -> Vec<Parameter> {
    declared
        .iter()
        .map(|&(name, declared_type)| Parameter {
            name: name.to_owned(),
            context_type: context_type(declared_type),
        })
        .collect()
}

/// An instance of `node_type`, with `bindings` given as `(parameter, source
/// instance)` pairs. Not an entry node; a test that wants one sets the flag.
#[cfg(test)]
pub(crate) fn instance(name: &str, node_type: &str, bindings: &[(&str, &str)]) -> NodeInstance {
    NodeInstance {
        name: name.to_owned(),
        node_type: node_type.to_owned(),
        entry: false,
        bindings: bindings
            .iter()
            .map(|&(parameter, source)| Binding {
                parameter: parameter.to_owned(),
                source: source.to_owned(),
            })
            .collect(),
        calls: Vec::new(),
    }
}

/// What every definition these builders make is called, and so the name a
/// signature defect about one carries.
#[cfg(test)]
pub(crate) const DEFINITION_NAME: &str = "workflow";

/// A definition of the given types and instances, designating `designated`.
#[cfg(test)]
pub(crate) fn definition(
    node_types: Vec<NodeType>,
    instances: Vec<NodeInstance>,
    designated: &[&str],
) -> WorkflowDefinition {
    WorkflowDefinition {
        name: DEFINITION_NAME.to_owned(),
        node_types,
        instances,
        designated_outputs: designated.iter().map(|&name| name.to_owned()).collect(),
    }
}
