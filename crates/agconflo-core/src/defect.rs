//! What one wiring defect says about itself: what is wrong, and where.

use std::fmt;

use crate::ContextType;

/// One thing wrong with one workflow definition, carrying the place it
/// concerns as values - a node instance and a parameter, an instance alone, or
/// the definition itself - and saying the same through [`fmt::Display`]. A name
/// that resolved to nothing is carried as it was written.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
// @A place carried as a value,IMPL_DEFECT_PLACE,impl,[CREQ_DEFECT_NAMES_PLACE],[DEC_FAILURES_NON_EXHAUSTIVE]
pub enum WiringDefect {
    /// A binding names an instance the definition does not carry.
    UnresolvedInstance {
        /// The instance consuming the binding.
        instance: String,
        /// The parameter it fills.
        parameter: String,
        /// The instance name that resolved to nothing, as it was written.
        unresolved: String,
    },
    /// An instance names a node type the definition does not carry; it carries no
    /// parameter.
    UnresolvedNodeType {
        /// The instance whose declaration is missing.
        instance: String,
        /// The node type name that resolved to nothing, as it was written.
        unresolved: String,
    },
    /// A binding names a parameter that the node type of its instance does not
    /// declare.
    UndeclaredParameter {
        /// The instance carrying the binding.
        instance: String,
        /// The parameter it names, as it was written.
        parameter: String,
    },
    /// More than one instance carries one name, reported once for the name.
    RepeatedInstance {
        /// The name the instances share.
        instance: String,
    },
    /// An instance binds one parameter more than once, reported once for the
    /// parameter.
    RepeatedBinding {
        /// The instance carrying the bindings.
        instance: String,
        /// The parameter bound more than once.
        parameter: String,
    },
    /// A binding joins an output to a parameter declared for another context type,
    /// carrying both type names.
    ContextTypeDisagreement {
        /// The instance consuming the binding.
        instance: String,
        /// The parameter it fills.
        parameter: String,
        /// The context type that parameter is declared for.
        expected: ContextType,
        /// The context type the wired output is declared to produce.
        produced: ContextType,
    },
    /// The definition designates no output, or more than one; it carries no
    /// instance and no parameter.
    SignatureOutputs {
        /// The definition whose signature is malformed.
        definition: String,
        /// How many outputs it designates, where exactly one is required.
        designated: usize,
    },
    /// The one output the definition designates names no instance of it: a
    /// signature defect, carrying no instance though it names one.
    UnresolvedOutput {
        /// The definition whose output names nothing.
        definition: String,
        /// The instance name that resolved to nothing, as it was written.
        unresolved: String,
    },
    /// An instance declares a call to a node type the definition does not carry;
    /// it carries no parameter.
    UnresolvedCall {
        /// The instance declaring the call.
        instance: String,
        /// The node type name that resolved to nothing, as it was written.
        unresolved: String,
    },
    /// An instance declares a call to a node type whose name is not one both
    /// providers accept as a tool's, whether or not the node type resolves.
    UnportableCallName {
        /// The instance declaring the call.
        instance: String,
        /// The name as it was written.
        name: String,
    },
    /// A binding takes an input of an instance whose node type does not route,
    /// or does not declare that input.
    UnroutedInput {
        /// The instance consuming the binding.
        instance: String,
        /// The parameter it fills.
        parameter: String,
        /// The instance whose input it takes.
        source: String,
        /// The input it takes, as it was written.
        input: String,
    },
}

impl WiringDefect {
    /// The node instance this defect concerns, or `None` for one that concerns
    /// the definition itself.
    pub fn instance(&self) -> Option<&str> {
        match self {
            Self::UnresolvedInstance { instance, .. }
            | Self::UnresolvedNodeType { instance, .. }
            | Self::UndeclaredParameter { instance, .. }
            | Self::RepeatedInstance { instance }
            | Self::RepeatedBinding { instance, .. }
            | Self::ContextTypeDisagreement { instance, .. }
            | Self::UnresolvedCall { instance, .. }
            | Self::UnportableCallName { instance, .. }
            | Self::UnroutedInput { instance, .. } => Some(instance),
            Self::SignatureOutputs { .. } | Self::UnresolvedOutput { .. } => None,
        }
    }

    /// The parameter this defect concerns, or `None` for one that concerns no
    /// single parameter.
    pub fn parameter(&self) -> Option<&str> {
        match self {
            Self::UnresolvedInstance { parameter, .. }
            | Self::UndeclaredParameter { parameter, .. }
            | Self::RepeatedBinding { parameter, .. }
            | Self::ContextTypeDisagreement { parameter, .. }
            | Self::UnroutedInput { parameter, .. } => Some(parameter),
            Self::UnresolvedNodeType { .. }
            | Self::RepeatedInstance { .. }
            | Self::SignatureOutputs { .. }
            | Self::UnresolvedOutput { .. }
            | Self::UnresolvedCall { .. }
            | Self::UnportableCallName { .. } => None,
        }
    }

    /// The call this defect concerns, as the node type name its instance lists,
    /// or `None` for one that concerns no call.
    ///
    /// Part of the place: an instance with two bad calls has two defects of one
    /// class, and the call is what tells them apart.
    pub fn call(&self) -> Option<&str> {
        match self {
            Self::UnresolvedCall { unresolved, .. } => Some(unresolved),
            Self::UnportableCallName { name, .. } => Some(name),
            _ => None,
        }
    }
}

impl fmt::Display for WiringDefect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnresolvedInstance {
                instance,
                parameter,
                unresolved,
            } => write!(
                f,
                "'{parameter}' of the node '{instance}' is bound to '{unresolved}', which this workflow does not carry"
            ),
            Self::UnresolvedNodeType {
                instance,
                unresolved,
            } => write!(
                f,
                "the node '{instance}' is of the type '{unresolved}', which this workflow does not carry"
            ),
            Self::UndeclaredParameter {
                instance,
                parameter,
            } => write!(
                f,
                "'{parameter}' is bound on the node '{instance}', and its type declares no such parameter"
            ),
            Self::RepeatedInstance { instance } => {
                write!(f, "the name '{instance}' is given to more than one node")
            }
            Self::RepeatedBinding {
                instance,
                parameter,
            } => write!(
                f,
                "'{parameter}' of the node '{instance}' is bound more than once, where one binding is allowed"
            ),
            Self::ContextTypeDisagreement {
                instance,
                parameter,
                expected,
                produced,
            } => write!(
                f,
                "'{parameter}' of the node '{instance}' is declared for '{}', and what is wired to it produces '{}'",
                expected.as_str(),
                produced.as_str()
            ),
            Self::SignatureOutputs {
                definition,
                designated,
            } => write!(
                f,
                "the workflow '{definition}' designates {designated} outputs, where exactly one is required"
            ),
            Self::UnresolvedOutput {
                definition,
                unresolved,
            } => write!(
                f,
                "the workflow '{definition}' designates '{unresolved}' as its output, which this workflow does not carry"
            ),
            Self::UnresolvedCall {
                instance,
                unresolved,
            } => write!(
                f,
                "the node '{instance}' may call '{unresolved}', which this workflow does not carry"
            ),
            Self::UnportableCallName { instance, name } => write!(
                f,
                "the node '{instance}' may call '{name}', which is not a tool name every provider accepts: 1 to 64 ASCII letters, digits, underscores or hyphens"
            ),
            Self::UnroutedInput {
                instance,
                parameter,
                source,
                input,
            } => write!(
                f,
                "'{parameter}' of the node '{instance}' takes the input '{input}' of '{source}', which is not a router declaring that input"
            ),
        }
    }
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases.

#[cfg(test)]
use crate::validate_wiring;
#[cfg(test)]
use crate::wiring::any_definition;
#[cfg(test)]
use crate::workflow::{DEFINITION_NAME, definition, instance, node_type};
#[cfg(test)]
use proptest::prelude::*;

#[cfg(test)]
proptest! {
    /// Every defect that concerns a wire names its consumer and the parameter,
    /// read from the defect itself rather than from its rendering, and each names
    /// something the definition carries.
    #[test]
    fn names_instance_and_parameter(workflow in any_definition()) {
        for defect in validate_wiring(&workflow) {
            // Exhaustive, so that a class added later is filed as concerning a wire
            // or not.
            match defect {
                WiringDefect::UnresolvedInstance { .. }
                | WiringDefect::UndeclaredParameter { .. }
                | WiringDefect::RepeatedBinding { .. }
                | WiringDefect::ContextTypeDisagreement { .. }
                | WiringDefect::UnroutedInput { .. } => {}
                WiringDefect::UnresolvedNodeType { .. }
                | WiringDefect::RepeatedInstance { .. }
                | WiringDefect::SignatureOutputs { .. }
                | WiringDefect::UnresolvedOutput { .. }
                | WiringDefect::UnresolvedCall { .. }
                | WiringDefect::UnportableCallName { .. } => continue,
            }

            let (Some(named), Some(parameter)) = (defect.instance(), defect.parameter()) else {
                return Err(TestCaseError::fail(format!("{defect:?} names no place")));
            };
            let Some(instance) = workflow
                .instances
                .iter()
                .find(|instance| instance.name == named)
            else {
                return Err(TestCaseError::fail(format!("{named} is not in the workflow")));
            };

            let declared = workflow
                .node_types
                .iter()
                .find(|declared| declared.name == instance.node_type)
                .is_some_and(|declared| {
                    declared
                        .required
                        .iter()
                        .any(|declared| declared.name == parameter)
                });
            let wired = instance
                .bindings
                .iter()
                .any(|binding| binding.parameter == parameter);
            prop_assert!(
                declared || wired,
                "{:?} names a parameter {} does not carry",
                defect,
                named
            );
        }
    }
}

#[test]
fn signature_names_the_definition() {
    let report = validate_wiring(&definition(Vec::new(), Vec::new(), &[]));

    let [defect] = report.as_slice() else {
        panic!("a definition designating no output reports one defect: {report:?}");
    };
    // The absence is the assertion.
    assert_eq!(defect.instance(), None, "{defect:?}");
    assert_eq!(defect.parameter(), None, "{defect:?}");
    assert!(
        matches!(defect, WiringDefect::SignatureOutputs { definition, .. } if definition == DEFINITION_NAME),
        "it names the definition: {defect:?}"
    );

    // An output naming no instance names that missing instance and still
    // concerns the definition.
    let renamed = definition(
        vec![node_type("source", &[], "note")],
        vec![instance("a", "source", &[])],
        &["nowhere"],
    );
    let report = validate_wiring(&renamed);
    let [defect] = report.as_slice() else {
        panic!("an output naming no instance reports one defect: {report:?}");
    };
    assert_eq!(defect.instance(), None, "{defect:?}");
    assert_eq!(defect.parameter(), None, "{defect:?}");
    assert!(
        matches!(defect, WiringDefect::UnresolvedOutput { definition, .. } if definition == DEFINITION_NAME),
        "it names the definition: {defect:?}"
    );
}

#[test]
fn echoes_an_unresolved_name() {
    // The name resolves to nothing, which is exactly why it has to be echoed:
    // it is the only thing tying the defect to what the author typed.
    let broken_wire = definition(
        vec![node_type("sink", &[("input", "note")], "note")],
        vec![instance("b", "sink", &[("input", "deleted")])],
        &["b"],
    );
    let report = validate_wiring(&broken_wire);
    assert!(
        matches!(
            report.as_slice(),
            [WiringDefect::UnresolvedInstance { unresolved, .. }] if unresolved == "deleted"
        ),
        "the instance name as it was written: {report:?}"
    );

    let missing_type = definition(Vec::new(), vec![instance("b", "not-supplied", &[])], &["b"]);
    let report = validate_wiring(&missing_type);
    assert!(
        matches!(
            report.as_slice(),
            [WiringDefect::UnresolvedNodeType { unresolved, .. }] if unresolved == "not-supplied"
        ),
        "the type name as it was written: {report:?}"
    );

    let renamed_output = definition(
        vec![node_type("source", &[], "note")],
        vec![instance("a", "source", &[])],
        &["renamed"],
    );
    let report = validate_wiring(&renamed_output);
    assert!(
        matches!(
            report.as_slice(),
            [WiringDefect::UnresolvedOutput { unresolved, .. }] if unresolved == "renamed"
        ),
        "the output name as it was written: {report:?}"
    );
}
