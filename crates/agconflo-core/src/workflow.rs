//! What a workflow is made of: the node types it names, the instances wired
//! into them, and the outputs it designates.
//!
//! Data, with every field public and nothing checked on the way in: a
//! definition holds every malformed shape, and the validator reports them.

use crate::ContextType;

/// One parameter a node type declares: its name, and the context type it is
/// declared for.
// @A parameter declared by name and type,TRACE_WORKFLOW_PARAMETER,trace,[],[DEC_EVERY_INPUT_REQUIRED]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter {
    /// The name a binding uses to reach this parameter.
    pub name: String,
    /// The context type a value arriving here is declared to be.
    pub context_type: ContextType,
}

/// What one kind of node consumes and produces, declared once and instantiated
/// as often as a workflow likes: the parameters it requires, the global context
/// types it reads by declaration, and one typed output.
// @A node type declared once,TRACE_WORKFLOW_NODE_TYPE,trace,[],[DEC_TWO_LAYERS, DEC_EVERY_INPUT_REQUIRED]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeType {
    /// The name an instance names to reach this declaration.
    pub name: String,
    /// What a node of this type does, in the words of its declaration, or empty
    /// when it gives none.
    // @A node type's description,TRACE_WORKFLOW_DESCRIPTION,trace,[],[DEC_TOOLS_OFFERED_AS_CONTEXTS]
    pub description: String,
    /// Parameters a node of this type cannot run without, which are all of
    /// its parameters.
    pub required: Vec<Parameter>,
    /// Context types read by declaration rather than through a binding.
    pub globals: Vec<ContextType>,
    /// The context type of the one output a node of this type produces.
    pub output: ContextType,
}

/// One parameter of one instance, wired to the output of a named instance: both
/// ends are names the definition carries.
// @A binding by names,TRACE_WORKFLOW_BINDING,trace,[],[DEC_BINDING_BY_PORT, NOTE_WORKFLOW_SHAPE]
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
    /// The bindings that fill this instance's parameters.
    pub bindings: Vec<Binding>,
    /// The node types a model performing this instance may call, in the order
    /// the workflow lists them, a name listed twice kept twice.
    // @Calls declared on the instance,TRACE_WORKFLOW_CALLS,trace,[],[DEC_CALLS_DECLARED_ON_THE_INSTANCE]
    pub calls: Vec<String>,
}

/// A workflow definition: the node types it carries, the instances wired into
/// it, and the outputs it designates - as many as it was given, naming
/// instances or not.
// @A definition holding every malformed shape,TRACE_WORKFLOW_DEFINITION,trace,[],[NOTE_WORKFLOW_SHAPE]
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
// Used by the validator's tests and by the defect's.

/// A context type, from a name the caller knows is not empty.
#[cfg(test)]
pub(crate) fn context_type(name: &str) -> ContextType {
    ContextType::new(name).expect("a non-empty type name is accepted")
}

/// A node type requiring `required` and producing `output`, with no globals. Each parameter is `(name, context type)`.
#[cfg(test)]
pub(crate) fn node_type(name: &str, required: &[(&str, &str)], output: &str) -> NodeType {
    NodeType {
        name: name.to_owned(),
        description: String::new(),
        required: parameters(required),
        globals: Vec::new(),
        output: context_type(output),
    }
}

#[cfg(test)]
impl NodeType {
    /// The same declaration, described as `description`.
    pub(crate) fn described(mut self, description: &str) -> Self {
        self.description = description.to_owned();
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
/// instance)` pairs.
#[cfg(test)]
pub(crate) fn instance(name: &str, node_type: &str, bindings: &[(&str, &str)]) -> NodeInstance {
    NodeInstance {
        name: name.to_owned(),
        node_type: node_type.to_owned(),
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
