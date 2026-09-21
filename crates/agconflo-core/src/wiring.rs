//! The wiring validator: one workflow definition compared against the node
//! types it names, and everything wrong with it reported at once.

use crate::defect::WiringDefect;
use crate::workflow::WorkflowDefinition;

/// Every wiring defect `definition` carries, and nothing at all for one that
/// carries none.
///
/// A report rather than a refusal: the caller decides what a defect costs, and
/// an empty report is the answer for a well-formed workflow
/// (`CREQ_VALIDATOR_ACCEPTS_WELL_FORMED`).
///
/// The order is the definition's own, with the signature last, so two runs over
/// one definition produce the same report. Nothing requires that, and it is
/// worth having anyway: a report whose order came from a hash map would make a
/// duplicate defect hard to see and a test of the report flaky.
pub fn validate_wiring(definition: &WorkflowDefinition) -> Vec<WiringDefect> {
    let mut defects = Vec::new();
    check_signature(definition, &mut defects);
    defects
}

/// A workflow is wired into another through its signature
/// (`DEC_WORKFLOW_SIGNATURE`), so one designating no output, or several, cannot
/// be composed and is malformed on its own terms.
///
/// Counting rather than testing for absence: a definition designating two is a
/// workflow whose result is whichever output a caller happens to bind, which is
/// the half a check written as "is one missing" passes.
///
/// It runs here, with everything else, rather than where a definition is
/// assembled. Refusing a definition for its signature before its wiring is
/// examined tells the author about the signature and nothing else, and they
/// learn the rest only after fixing it - which is the round trip
/// `FEAT_WIRING_ALL_DEFECTS` exists to prevent.
// @Exactly one designated output,IMPL_WIRING_SIGNATURE,impl,[CREQ_VALIDATOR_ONE_OUTPUT]
fn check_signature(definition: &WorkflowDefinition, defects: &mut Vec<WiringDefect>) {
    let designated = definition.designated_outputs.len();
    if designated != 1 {
        defects.push(WiringDefect::SignatureOutputs {
            definition: definition.name.clone(),
            designated,
        });
    }
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases, for the reason given in id.rs.

#[cfg(test)]
use crate::workflow::{DEFINITION_NAME, definition, instance, node_type};

/// The defect a definition designating `designated` outputs must report.
#[cfg(test)]
fn signature_defect(designated: usize) -> WiringDefect {
    WiringDefect::SignatureOutputs {
        definition: DEFINITION_NAME.to_owned(),
        designated,
    }
}

#[test]
fn signature_without_one_output_is_reported() {
    // Wiring that is sound on both sides of the signature, so the signature
    // defect is the whole report rather than one defect among several.
    let types = vec![
        node_type("source", &[], "note"),
        node_type("sink", &[("input", "note")], "note"),
    ];
    let instances = vec![
        instance("a", "source", &[]),
        instance("b", "sink", &[("input", "a")]),
    ];

    let none = definition(types.clone(), instances.clone(), &[]);
    assert_eq!(validate_wiring(&none), vec![signature_defect(0)]);

    // Asserted separately, and this is the half a check written as "is the
    // output missing" passes: a workflow with two results has no declared one.
    let two = definition(types, instances, &["a", "b"]);
    assert_eq!(validate_wiring(&two), vec![signature_defect(2)]);
}

#[test]
fn legal_signatures_pass() {
    // A workflow with no entry parameters at all, which is legal.
    let closed = definition(
        vec![node_type("source", &[], "note")],
        vec![instance("a", "source", &[])],
        &["a"],
    );
    assert_eq!(validate_wiring(&closed), Vec::new());

    // A designated output whose node also feeds another node. Feeding
    // something does not make it less terminal, and it is the ordinary shape of
    // a loop (`DEC_BACK_EDGES_ALLOWED`), where the designated output feeds the
    // node that starts the next pass.
    let feeding = definition(
        vec![
            node_type("source", &[], "note"),
            node_type("pass", &[("input", "note")], "note"),
        ],
        vec![
            instance("a", "source", &[]),
            instance("b", "pass", &[("input", "a")]),
            instance("c", "pass", &[("input", "b")]),
        ],
        &["b"],
    );
    assert_eq!(validate_wiring(&feeding), Vec::new());
}

#[test]
fn empty_definition_is_refused_for_its_signature() {
    let report = validate_wiring(&definition(Vec::new(), Vec::new(), &[]));

    // The count and the class, not merely that something was reported: an empty
    // definition has nothing to wire and nothing to designate, so it is the case
    // where two requirements could quietly both fire.
    assert_eq!(report, vec![signature_defect(0)]);
}
