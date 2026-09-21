//! What one wiring defect says about itself: what is wrong, and where.

use std::fmt;

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
/// a node instance and one of its parameters; a defect about the definition's
/// signature concerns the definition and no node in it, and a variant that
/// carried an instance anyway would send an author to a node that is not wrong.
///
/// `#[non_exhaustive]` because the classes are not finished: three shapes are
/// recorded as not yet answered under `CREQ_VALIDATOR_BINDING_RESOLVES`, and
/// each may earn a variant when workflow loading settles it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
// @A place carried as a value,IMPL_DEFECT_PLACE,impl,[CREQ_DEFECT_NAMES_PLACE]
pub enum WiringDefect {
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
            Self::SignatureOutputs { .. } => None,
        }
    }

    /// The parameter this defect concerns, or `None` for one that concerns no
    /// single parameter.
    pub fn parameter(&self) -> Option<&str> {
        match self {
            Self::SignatureOutputs { .. } => None,
        }
    }
}

impl fmt::Display for WiringDefect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
use crate::workflow::{DEFINITION_NAME, definition};

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
