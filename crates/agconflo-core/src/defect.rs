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
/// unknowable; a defect about the signature concerns the definition and no node
/// in it. A variant that carried an instance or a parameter anyway would send an
/// author to a place that is not wrong.
///
/// A name that resolved to nothing is carried as it was written, and is the one
/// field that names something the definition does not have. It is the only thing
/// tying the defect to what the author typed.
///
/// `#[non_exhaustive]` because the classes are not finished: three shapes are
/// recorded as not yet answered under `CREQ_VALIDATOR_BINDING_RESOLVES`, and
/// each may earn a variant when workflow loading settles it.
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
}

impl WiringDefect {
    /// The node instance this defect concerns, or `None` for one that concerns
    /// the definition itself.
    pub fn instance(&self) -> Option<&str> {
        match self {
            Self::RequiredParameterUnbound { instance, .. }
            | Self::UnresolvedInstance { instance, .. }
            | Self::UnresolvedNodeType { instance, .. }
            | Self::ContextTypeDisagreement { instance, .. } => Some(instance),
            Self::SignatureOutputs { .. } => None,
        }
    }

    /// The parameter this defect concerns, or `None` for one that concerns no
    /// single parameter.
    pub fn parameter(&self) -> Option<&str> {
        match self {
            Self::RequiredParameterUnbound { parameter, .. }
            | Self::UnresolvedInstance { parameter, .. }
            | Self::ContextTypeDisagreement { parameter, .. } => Some(parameter),
            Self::UnresolvedNodeType { .. } | Self::SignatureOutputs { .. } => None,
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
        }
    }
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases, for the reason given in id.rs.

#[cfg(test)]
use crate::validate_wiring;
#[cfg(test)]
use crate::workflow::{DEFINITION_NAME, definition, instance, node_type};

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
}
