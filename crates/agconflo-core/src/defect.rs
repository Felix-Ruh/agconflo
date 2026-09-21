//! What one wiring defect says about itself: what is wrong, and where.

use std::fmt;

use crate::ContextType;

/// One thing wrong with one workflow definition.
///
/// A value rather than a message. Every defect carries the place it concerns in
/// fields of its own, so a caller acts on it without parsing prose - and the
/// caller this feature was written for is an agent correcting its own workflow
/// (`CREQ_DEFECT_NAMES_PLACE`). The message exists too, through [`fmt::Display`],
/// and says the same thing for a person reading a report.
///
/// The place is not the same shape for every defect, and flattening it into one
/// would be the defect this is written to avoid. A defect about a wire concerns
/// a node instance and one of its parameters; a defect about an instance whose
/// node type is missing concerns that instance and no parameter, because its
/// declaration is what is missing and its parameter list is therefore
/// unknowable; a name several instances share concerns that name and no
/// parameter, since it is the one place such a defect can give; a defect about
/// the signature concerns the definition and no node in it. A variant that
/// carried an instance or a parameter anyway would send an author to a place
/// that is not wrong.
///
/// A name that resolved to nothing is carried as it was written, and is the one
/// field that names something the definition does not have. It is the only thing
/// tying the defect to what the author typed.
///
/// `#[non_exhaustive]` because the classes are not finished: the shapes still
/// recorded as open under `CREQ_VALIDATOR_BINDING_RESOLVES` - declarations a
/// definition carries twice, and a binding into an entry node - may each earn a
/// variant of their own.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
// @A place carried as a value,IMPL_DEFECT_PLACE,impl,[CREQ_DEFECT_NAMES_PLACE]
pub enum WiringDefect {
    /// A required parameter of an instance carries no binding
    /// (`CREQ_VALIDATOR_REQUIRED_BOUND`).
    RequiredParameterUnbound {
        /// The instance whose parameter is unwired.
        instance: String,
        /// The required parameter carrying no binding.
        parameter: String,
    },
    /// A binding names an instance the definition does not carry
    /// (`CREQ_VALIDATOR_BINDING_RESOLVES`).
    UnresolvedInstance {
        /// The instance consuming the binding.
        instance: String,
        /// The parameter it fills.
        parameter: String,
        /// The instance name that resolved to nothing, as it was written.
        unresolved: String,
    },
    /// An instance names a node type the definition does not carry
    /// (`CREQ_VALIDATOR_BINDING_RESOLVES`). It concerns the instance, whose
    /// parameters are unknowable without the declaration, so it carries no
    /// parameter.
    UnresolvedNodeType {
        /// The instance whose declaration is missing.
        instance: String,
        /// The node type name that resolved to nothing, as it was written.
        unresolved: String,
    },
    /// A binding names a parameter that the node type of its instance declares
    /// neither as required nor as optional
    /// (`CREQ_VALIDATOR_PARAMETER_DECLARED`).
    UndeclaredParameter {
        /// The instance carrying the binding.
        instance: String,
        /// The parameter it names, as it was written.
        parameter: String,
    },
    /// More than one instance carries one name
    /// (`CREQ_VALIDATOR_INSTANCE_NAMED_ONCE`). Reported once for the name,
    /// however many carry it, and nothing about any of them is reported
    /// besides: a place naming several instances is no place.
    RepeatedInstance {
        /// The name the instances share.
        instance: String,
    },
    /// An instance binds one parameter more than once
    /// (`CREQ_VALIDATOR_PARAMETER_BOUND_ONCE`). Reported once for the
    /// parameter, however many bindings it has.
    RepeatedBinding {
        /// The instance carrying the bindings.
        instance: String,
        /// The parameter bound more than once.
        parameter: String,
    },
    /// A binding joins an output to a parameter declared for another context
    /// type (`CREQ_VALIDATOR_TYPES_AGREE`).
    ///
    /// Both type names are carried, because what a reader acts on is which type
    /// was expected and which arrived; a defect saying only that the two differ
    /// sends them back to the declarations to find out.
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
    /// The definition designates no output, or designates more than one
    /// (`CREQ_VALIDATOR_ONE_OUTPUT`). It concerns the definition itself, so it
    /// carries no instance and no parameter.
    SignatureOutputs {
        /// The definition whose signature is malformed.
        definition: String,
        /// How many outputs it designates, where exactly one is required.
        designated: usize,
    },
    /// The one output the definition designates names no instance of it
    /// (`CREQ_VALIDATOR_OUTPUT_RESOLVES`). A signature defect, so it concerns
    /// the definition and carries no instance, though it names one.
    UnresolvedOutput {
        /// The definition whose output names nothing.
        definition: String,
        /// The instance name that resolved to nothing, as it was written.
        unresolved: String,
    },
}

impl WiringDefect {
    /// The node instance this defect concerns, or `None` for one that concerns
    /// the definition itself.
    pub fn instance(&self) -> Option<&str> {
        match self {
            Self::RequiredParameterUnbound { instance, .. }
            | Self::UnresolvedInstance { instance, .. }
            | Self::UnresolvedNodeType { instance, .. }
            | Self::UndeclaredParameter { instance, .. }
            | Self::RepeatedInstance { instance }
            | Self::RepeatedBinding { instance, .. }
            | Self::ContextTypeDisagreement { instance, .. } => Some(instance),
            Self::SignatureOutputs { .. } | Self::UnresolvedOutput { .. } => None,
        }
    }

    /// The parameter this defect concerns, or `None` for one that concerns no
    /// single parameter.
    pub fn parameter(&self) -> Option<&str> {
        match self {
            Self::RequiredParameterUnbound { parameter, .. }
            | Self::UnresolvedInstance { parameter, .. }
            | Self::UndeclaredParameter { parameter, .. }
            | Self::RepeatedBinding { parameter, .. }
            | Self::ContextTypeDisagreement { parameter, .. } => Some(parameter),
            Self::UnresolvedNodeType { .. }
            | Self::RepeatedInstance { .. }
            | Self::SignatureOutputs { .. }
            | Self::UnresolvedOutput { .. } => None,
        }
    }
}

impl fmt::Display for WiringDefect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequiredParameterUnbound {
                instance,
                parameter,
            } => write!(
                f,
                "the node '{instance}' requires '{parameter}', and nothing is bound to it"
            ),
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
        }
    }
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases, for the reason given in id.rs.

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
    /// read from the defect itself rather than from its rendering. A place that
    /// can be recovered only by parsing a message is a place an agent
    /// correcting its own workflow cannot use, and the two are
    /// indistinguishable to a human reading the output - which is how this
    /// requirement would be lost without anyone noticing.
    ///
    /// Each also names something the definition carries: the instance is one of
    /// its own, and the parameter is either declared by the type that instance
    /// names or written in one of its bindings.
    #[test]
    fn names_instance_and_parameter(workflow in any_definition()) {
        for defect in validate_wiring(&workflow) {
            // Exhaustive on purpose, though the enum is `non_exhaustive`: a
            // class added later has to be filed as concerning a wire or not
            // concerning one, rather than escaping this property in silence.
            match defect {
                WiringDefect::RequiredParameterUnbound { .. }
                | WiringDefect::UnresolvedInstance { .. }
                | WiringDefect::UndeclaredParameter { .. }
                | WiringDefect::RepeatedBinding { .. }
                | WiringDefect::ContextTypeDisagreement { .. } => {}
                WiringDefect::UnresolvedNodeType { .. }
                | WiringDefect::RepeatedInstance { .. }
                | WiringDefect::SignatureOutputs { .. }
                | WiringDefect::UnresolvedOutput { .. } => continue,
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
                        .chain(&declared.optional)
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
    // The absence is the assertion. A defect shaped so that every one of them
    // has an instance forces this one to name some node, and the author is then
    // sent to a node that has nothing to do with it.
    assert_eq!(defect.instance(), None, "{defect:?}");
    assert_eq!(defect.parameter(), None, "{defect:?}");
    assert!(
        matches!(defect, WiringDefect::SignatureOutputs { definition, .. } if definition == DEFINITION_NAME),
        "it names the definition: {defect:?}"
    );

    // An output naming no instance names an instance - the one that is not
    // there - and still concerns the definition. Filing the name it carries as
    // the instance the defect is about would send the author to a node that
    // does not exist.
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
