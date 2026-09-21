//! The wiring validator: one workflow definition compared against the node
//! types it names, and everything wrong with it reported at once.

use std::collections::HashMap;

use crate::defect::WiringDefect;
use crate::workflow::{Binding, NodeInstance, NodeType, WorkflowDefinition};

/// Every wiring defect `definition` carries, and nothing at all for one that
/// carries none.
///
/// A report rather than a refusal: the caller decides what a defect costs, and
/// an empty report is the answer for a well-formed workflow
/// (`CREQ_VALIDATOR_ACCEPTS_WELL_FORMED`).
///
/// The order is the definition's own - each instance in turn, then the
/// signature - so two runs over one definition produce the same report. Nothing
/// requires that, and it is worth having anyway: a report whose order came from
/// a hash map would make a duplicate defect hard to see and a test of the report
/// flaky.
pub fn validate_wiring(definition: &WorkflowDefinition) -> Vec<WiringDefect> {
    let mut defects = Vec::new();
    let declarations = by_name(&definition.node_types, |declared| &declared.name);
    let instances = by_name(&definition.instances, |instance| &instance.name);

    for instance in &definition.instances {
        check_instance(instance, &declarations, &instances, &mut defects);
    }
    check_signature(definition, &mut defects);

    defects
}

/// A lookup from a name to the first thing carrying it.
///
/// First rather than last, and neither is asserted anywhere. Two instances
/// sharing one name is one of the shapes `CREQ_VALIDATOR_BINDING_RESOLVES`
/// records as not yet answered, so a name that resolves to two things has to
/// resolve to something rather than stop the walk. Every instance is still
/// walked, because the walk runs over the definition's list and not over this.
fn by_name<'a, T>(items: &'a [T], name: impl Fn(&'a T) -> &'a str) -> HashMap<&'a str, &'a T> {
    let mut by_name = HashMap::new();
    for item in items {
        by_name.entry(name(item)).or_insert(item);
    }
    by_name
}

/// One instance against the declaration it names.
///
/// The declaration is resolved first, and an instance without one is reported
/// and left: its parameters are unknowable, so an unbound-parameter defect
/// about it would be invented rather than found, and so would a disagreement
/// about a wire whose declared types nobody can read. The missing type is in the
/// report, and the rest of the definition is still walked.
// @An instance checked against the type it names,IMPL_WIRING_INSTANCE_TYPE,impl,[CREQ_VALIDATOR_BINDING_RESOLVES]
fn check_instance(
    instance: &NodeInstance,
    declarations: &HashMap<&str, &NodeType>,
    instances: &HashMap<&str, &NodeInstance>,
    defects: &mut Vec<WiringDefect>,
) {
    let Some(declaration) = declarations.get(instance.node_type.as_str()) else {
        defects.push(WiringDefect::UnresolvedNodeType {
            instance: instance.name.clone(),
            unresolved: instance.node_type.clone(),
        });
        return;
    };

    check_required_bound(instance, declaration, defects);
    check_bindings(instance, declaration, declarations, instances, defects);
}

/// Every parameter the declaration requires carries a binding.
///
/// The walk runs over the declaration rather than over the bindings, and that is
/// the whole difference between finding this defect and never seeing it: a
/// parameter with no binding is exactly the one a walk over the bindings never
/// visits.
///
/// Optional parameters and declared globals are not required to be bound.
/// Demanding a binding for an optional makes the second declared list
/// meaningless, and demanding one for a global refuses a workflow that is
/// correct - globals are read by declaration rather than wired
/// (`DEC_DECLARED_PARAMETERS`).
///
/// An entry node is left alone: its parameters are the workflow's own, supplied
/// when the workflow is invoked rather than by a wire
/// (`DEC_WORKFLOW_SIGNATURE`).
// @Every required parameter carries a binding,IMPL_WIRING_REQUIRED_BOUND,impl,[CREQ_VALIDATOR_REQUIRED_BOUND]
fn check_required_bound(
    instance: &NodeInstance,
    declaration: &NodeType,
    defects: &mut Vec<WiringDefect>,
) {
    if instance.entry {
        return;
    }

    for parameter in &declaration.required {
        let bound = instance
            .bindings
            .iter()
            .any(|binding| binding.parameter == parameter.name);
        if !bound {
            defects.push(WiringDefect::RequiredParameterUnbound {
                instance: instance.name.clone(),
                parameter: parameter.name.clone(),
            });
        }
    }
}

/// Every binding names an instance the definition carries.
///
/// One defect per binding rather than one per name that failed to resolve.
/// Deduplicating by the name is the tidy-looking version of this and leaves
/// every wire but one unnamed: five parameters bound to a deleted instance are
/// five wires to repoint.
// @Every binding's source resolved,IMPL_WIRING_BINDING_SOURCE,impl,[CREQ_VALIDATOR_BINDING_RESOLVES]
fn check_bindings(
    instance: &NodeInstance,
    declaration: &NodeType,
    declarations: &HashMap<&str, &NodeType>,
    instances: &HashMap<&str, &NodeInstance>,
    defects: &mut Vec<WiringDefect>,
) {
    for binding in &instance.bindings {
        let Some(source) = instances.get(binding.source.as_str()) else {
            defects.push(WiringDefect::UnresolvedInstance {
                instance: instance.name.clone(),
                parameter: binding.parameter.clone(),
                unresolved: binding.source.clone(),
            });
            continue;
        };
        check_binding_type(
            instance,
            declaration,
            binding,
            source,
            declarations,
            defects,
        );
    }
}

/// The type declared for a parameter and the type declared for the output wired
/// to it are the same name, exactly.
///
/// Exactly, because every way of loosening the comparison - ignoring case,
/// trimming, matching a prefix - makes two distinct types compare as one, and a
/// node handed the wrong context produces a confident wrong answer rather than
/// failing.
///
/// Two ends are passed over rather than compared, and each is deliberate. A
/// binding naming a parameter no declaration carries is one of the shapes
/// recorded as not yet answered, so classifying it here would pin behaviour no
/// requirement asks for. And a producer of a node type the definition does not
/// carry has no declared output at all: its missing type is already in the
/// report, and comparing against a stand-in for one - an empty name, a default -
/// would make every wire out of it disagree and send the author to change a type
/// that is not wrong.
///
/// The check is reached for every binding of every instance, including one whose
/// instance already carries an unbound-parameter defect. A walk that moved on
/// after an instance's first defect would hide every disagreement below it.
// @Both ends of a wire declare one context type,IMPL_WIRING_TYPES_AGREE,impl,[CREQ_VALIDATOR_TYPES_AGREE]
fn check_binding_type(
    instance: &NodeInstance,
    declaration: &NodeType,
    binding: &Binding,
    source: &NodeInstance,
    declarations: &HashMap<&str, &NodeType>,
    defects: &mut Vec<WiringDefect>,
) {
    let declared = declaration
        .required
        .iter()
        .chain(&declaration.optional)
        .find(|parameter| parameter.name == binding.parameter);
    let (Some(parameter), Some(producer)) = (declared, declarations.get(source.node_type.as_str()))
    else {
        return;
    };

    if producer.output != parameter.context_type {
        defects.push(WiringDefect::ContextTypeDisagreement {
            instance: instance.name.clone(),
            parameter: binding.parameter.clone(),
            expected: parameter.context_type.clone(),
            produced: producer.output.clone(),
        });
    }
}

/// A workflow is wired into another through its signature
/// (`DEC_WORKFLOW_SIGNATURE`), so one designating no output, or several, cannot
/// be composed and is malformed on its own terms.
///
/// Counting rather than testing for absence: a definition designating two is a
/// workflow whose result is whichever output a caller happens to bind, which is
/// the half a check written as "is one missing" passes.
///
/// It runs here, in the walk, rather than where a definition is assembled.
/// Refusing a definition for its signature before its wiring is examined tells
/// the author about the signature and nothing else, and they learn the rest only
/// after fixing it - which is the round trip `FEAT_WIRING_ALL_DEFECTS` exists to
/// prevent.
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
#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::prelude::*;

/// The defect a definition designating `designated` outputs must report.
#[cfg(test)]
fn signature_defect(designated: usize) -> WiringDefect {
    WiringDefect::SignatureOutputs {
        definition: DEFINITION_NAME.to_owned(),
        designated,
    }
}

/// An unbound required parameter, as the report names it.
#[cfg(test)]
fn unbound(instance: &str, parameter: &str) -> WiringDefect {
    WiringDefect::RequiredParameterUnbound {
        instance: instance.to_owned(),
        parameter: parameter.to_owned(),
    }
}

/// A wire whose two ends declare different context types, as the report names
/// it.
#[cfg(test)]
fn disagreement(instance: &str, parameter: &str, expected: &str, produced: &str) -> WiringDefect {
    WiringDefect::ContextTypeDisagreement {
        instance: instance.to_owned(),
        parameter: parameter.to_owned(),
        expected: crate::workflow::context_type(expected),
        produced: crate::workflow::context_type(produced),
    }
}

/// A binding whose source resolves to nothing, as the report names it.
#[cfg(test)]
fn unresolved(instance: &str, parameter: &str, source: &str) -> WiringDefect {
    WiringDefect::UnresolvedInstance {
        instance: instance.to_owned(),
        parameter: parameter.to_owned(),
        unresolved: source.to_owned(),
    }
}

#[cfg(test)]
proptest! {
    /// Each instance is of a type of its own, declaring required parameters,
    /// optional parameters and requested globals together - a walk that reports
    /// an optional or a global is over-blocking, and a definition built from
    /// required parameters alone would never show it. The first instance is
    /// given no bindings whatever its mask says, because an instance a walk over
    /// the bindings never visits is the case this exists to catch.
    ///
    /// What the report should be is worked out from the masks, which is a
    /// different computation from the walk under test.
    #[test]
    fn every_unbound_required_is_reported(
        shapes in vec((1..=3usize, 0..=2usize, 0..=2usize, any::<u8>()), 1..=4)
    ) {
        let mut node_types = Vec::new();
        let mut instances = Vec::new();
        let mut expected = Vec::new();

        for (node, &(required, optional, globals, mask)) in shapes.iter().enumerate() {
            let name = format!("n{node}");
            let declared_type = format!("t{node}");
            let required_names: Vec<String> = (0..required).map(|p| format!("r{p}")).collect();
            let optional_names: Vec<String> = (0..optional).map(|p| format!("o{p}")).collect();
            let global_names: Vec<String> = (0..globals).map(|g| format!("g{g}")).collect();

            let declared: Vec<(&str, &str)> =
                required_names.iter().map(|p| (p.as_str(), "note")).collect();
            let accepted: Vec<(&str, &str)> =
                optional_names.iter().map(|p| (p.as_str(), "note")).collect();
            let read: Vec<&str> = global_names.iter().map(String::as_str).collect();
            node_types.push(
                node_type(&declared_type, &declared, "note")
                    .with_optional(&accepted)
                    .with_globals(&read),
            );

            // The first instance carries no bindings at all; every other binds
            // the subset its mask picks, always to a name that resolves.
            let mask = if node == 0 { 0 } else { mask };
            let mut bindings = Vec::new();
            for (bit, parameter) in required_names.iter().chain(&optional_names).enumerate() {
                if mask & (1u8 << bit) != 0 {
                    bindings.push((parameter.as_str(), "n0"));
                } else if bit < required {
                    expected.push(unbound(&name, parameter));
                }
            }
            instances.push(instance(&name, &declared_type, &bindings));
        }

        let designated = [instances[0].name.clone()];
        let designated: Vec<&str> = designated.iter().map(String::as_str).collect();
        let report = validate_wiring(&definition(node_types, instances, &designated));
        prop_assert_eq!(report, expected);
    }

    /// One name the definition does not carry, bound by several parameters of
    /// several consumers. Each is its own wire to repoint.
    #[test]
    fn every_broken_wire_is_reported(consumers in vec(1..=3usize, 1..=3)) {
        let mut node_types = Vec::new();
        let mut instances = Vec::new();
        let mut expected = Vec::new();

        for (consumer, &parameters) in consumers.iter().enumerate() {
            let name = format!("c{consumer}");
            let declared_type = format!("t{consumer}");
            let parameter_names: Vec<String> = (0..parameters).map(|p| format!("p{p}")).collect();
            let declared: Vec<(&str, &str)> =
                parameter_names.iter().map(|p| (p.as_str(), "note")).collect();
            node_types.push(node_type(&declared_type, &declared, "note"));

            let bindings: Vec<(&str, &str)> = parameter_names
                .iter()
                .map(|p| (p.as_str(), "ghost"))
                .collect();
            instances.push(instance(&name, &declared_type, &bindings));

            for parameter in &parameter_names {
                expected.push(unresolved(&name, parameter, "ghost"));
            }
        }

        let report = validate_wiring(&definition(node_types, instances, &["c0"]));
        prop_assert_eq!(report, expected);
    }

    /// One wire between two declared type names, the second derived from the
    /// first by each way a comparison gets loosened: the same name, the same
    /// name upper-cased, the same name with a leading or a trailing space, and
    /// the same name with a character appended so that one is a prefix of the
    /// other. Two unrelated random names would prove only that unrelated names
    /// differ, which no comparison anyone would write gets wrong.
    #[test]
    fn type_names_compare_exactly(name in "[a-z]{1,6}", variation in 0..5usize) {
        let produced = match variation {
            0 => name.clone(),
            1 => name.to_ascii_uppercase(),
            2 => format!(" {name}"),
            3 => format!("{name} "),
            _ => format!("{name}x"),
        };

        let wired = definition(
            vec![
                node_type("source", &[], &produced),
                node_type("sink", &[("input", &name)], &name),
            ],
            vec![
                instance("a", "source", &[]),
                instance("b", "sink", &[("input", "a")]),
            ],
            &["b"],
        );
        let report = validate_wiring(&wired);

        if produced == name {
            prop_assert_eq!(report, Vec::new());
        } else {
            prop_assert_eq!(report, vec![disagreement("b", "input", &name, &produced)]);
        }
    }
}

#[test]
fn type_disagreement_is_reported() {
    let crossed = definition(
        vec![
            node_type("differ", &[], "diff"),
            node_type("summarise", &[("input", "summary")], "summary"),
        ],
        vec![
            instance("a", "differ", &[]),
            instance("b", "summarise", &[("input", "a")]),
        ],
        &["b"],
    );

    // Both names are asserted on, because what a reader has to act on is which
    // type was expected and which arrived; a defect saying only that the two
    // differ sends them back to the declarations to find out.
    assert_eq!(
        validate_wiring(&crossed),
        vec![disagreement("b", "input", "summary", "diff")]
    );
}

#[test]
fn agreeing_wires_pass() {
    let fanned = definition(
        vec![
            // One context type declared on two different node types.
            node_type("source", &[], "note"),
            node_type("pair", &[("left", "note"), ("right", "note")], "note"),
        ],
        vec![
            instance("a", "source", &[]),
            // One output bound by several parameters, and by several
            // instances: fan-out, which the model allows
            // (`DEC_BINDING_BY_PORT`). A check recording one consumer per
            // output refuses this while comparing every name correctly.
            instance("b", "pair", &[("left", "a"), ("right", "a")]),
            instance("c", "pair", &[("left", "a"), ("right", "b")]),
        ],
        &["c"],
    );

    assert_eq!(validate_wiring(&fanned), Vec::new());
}

#[test]
fn unwired_instance_is_reported() {
    let types = vec![
        node_type("pair", &[("left", "note"), ("right", "note")], "note"),
        node_type("source", &[], "note"),
    ];
    let instances = vec![
        instance("a", "source", &[]),
        instance("b", "pair", &[]),
        instance("c", "pair", &[("left", "a")]),
    ];
    let report = validate_wiring(&definition(types, instances, &["a"]));

    // The granularity is the assertion: one defect saying the instance is
    // unwired would leave the author to work out which parameters it meant.
    // What holds afterwards is that the walk continues - c's own unbound
    // parameter is in the same report.
    assert_eq!(
        report,
        vec![
            unbound("b", "left"),
            unbound("b", "right"),
            unbound("c", "right"),
        ]
    );
}

#[test]
fn degenerate_declarations_pass() {
    let types = vec![
        // Optionals all unbound, and a global that no binding carries.
        node_type("lenient", &[], "note")
            .with_optional(&[("hint", "note")])
            .with_globals(&["policy"]),
        // A type declaring no parameters at all.
        node_type("bare", &[], "note"),
        // Required parameters, supplied by the workflow rather than by wires.
        node_type("entry", &[("argument", "note")], "note"),
    ];
    let instances = vec![
        instance("a", "lenient", &[]),
        instance("b", "bare", &[]),
        instance("c", "entry", &[]).into_entry(),
    ];

    assert_eq!(
        validate_wiring(&definition(types, instances, &["a"])),
        Vec::new()
    );
}

#[test]
fn binding_to_missing_instance_is_reported() {
    let broken = definition(
        vec![node_type("sink", &[("input", "note")], "note")],
        vec![instance("b", "sink", &[("input", "renamed")])],
        &["b"],
    );

    // What must not also appear is the point of the case: the parameter is
    // bound, so no unbound-parameter defect is reported for it, and a report
    // carrying both would send the author to add a wire that is already there.
    assert_eq!(
        validate_wiring(&broken),
        vec![unresolved("b", "input", "renamed")]
    );
}

#[test]
fn instance_of_missing_type_is_reported() {
    let types = vec![node_type("sink", &[("input", "note")], "note")];
    let instances = vec![
        // Of a type nobody supplied, and carrying a broken wire of its own.
        instance("a", "not-supplied", &[("whatever", "ghost")]),
        // Fed by it: its producer has no declared output type at all.
        instance("b", "sink", &[("input", "a")]),
        instance("c", "sink", &[]),
    ];
    let report = validate_wiring(&definition(types, instances, &["b"]));

    // Nothing else about that instance is reported, which is the half that can
    // regress quietly: without its declaration nothing about its parameters is
    // knowable, so every other defect about it would be invented - and so would
    // a disagreement about the wire it feeds, where comparing against a stand-in
    // for the type it does not declare sends the author to change a type that is
    // not wrong. The rest of the definition is still walked, so c's own defect
    // is in the same report.
    assert_eq!(
        report,
        vec![
            WiringDefect::UnresolvedNodeType {
                instance: "a".to_owned(),
                unresolved: "not-supplied".to_owned(),
            },
            unbound("c", "input"),
        ]
    );
}

#[test]
fn self_binding_and_shared_types_pass() {
    // A binding from an instance to itself: a cycle of length one, and legal.
    let looping = definition(
        vec![node_type("pass", &[("input", "note")], "note")],
        vec![instance("a", "pass", &[("input", "a")])],
        &["a"],
    );
    assert_eq!(validate_wiring(&looping), Vec::new());

    // Two instances of one node type, which share a declaration and nothing
    // else. A resolver written as if the definition were a tree of distinct
    // nodes refuses both of these by accident.
    let shared = definition(
        vec![
            node_type("source", &[], "note"),
            node_type("pass", &[("input", "note")], "note"),
        ],
        vec![
            instance("a", "source", &[]),
            instance("b", "pass", &[("input", "a")]),
            instance("c", "pass", &[("input", "a")]),
        ],
        &["b"],
    );
    assert_eq!(validate_wiring(&shared), Vec::new());
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
