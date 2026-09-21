//! The wiring validator: one workflow definition compared against the node
//! types it names, and everything wrong with it reported at once.

use std::collections::HashMap;

use crate::defect::WiringDefect;
use crate::workflow::{Binding, NodeInstance, NodeType, Parameter, WorkflowDefinition};

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
///
/// Nothing here returns early, and that is the requirement rather than a
/// stylistic preference. A validator that stops at the first defect reports each
/// of them perfectly and still turns ten defects into ten rounds of edit,
/// revalidate and read, for an author that pays per round trip. The signature is
/// checked with everything else for the same reason, rather than refusing the
/// definition before its wiring is looked at.
// @Every instance walked and everything collected,IMPL_WIRING_WALK,impl,[CREQ_VALIDATOR_EVERY_DEFECT, CREQ_VALIDATOR_ACCEPTS_WELL_FORMED]
pub fn validate_wiring(definition: &WorkflowDefinition) -> Vec<WiringDefect> {
    let mut defects = Vec::new();
    let declarations = by_name(&definition.node_types, |declared| &declared.name);
    let instances = by_name(&definition.instances, |instance| &instance.name);

    for instance in &definition.instances {
        check_instance(instance, &declarations, &instances, &mut defects);
    }
    check_signature(definition, &mut defects);
    check_output_resolves(definition, &instances, &mut defects);

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

/// Every binding fills a parameter its instance's type declares, and names an
/// instance the definition carries.
///
/// One defect per binding rather than one per name that failed to resolve.
/// Deduplicating by the name is the tidy-looking version of this and leaves
/// every wire but one unnamed: five parameters bound to a deleted instance are
/// five wires to repoint.
///
/// Both ends are looked at whatever the other turned out to be. A binding to an
/// undeclared parameter from an instance that is not there is two fixes, the
/// parameter renamed and the source repointed, and a walk moving on after the
/// first leaves the second for the next round trip.
// @Every binding's source resolved,IMPL_WIRING_BINDING_SOURCE,impl,[CREQ_VALIDATOR_BINDING_RESOLVES]
fn check_bindings(
    instance: &NodeInstance,
    declaration: &NodeType,
    declarations: &HashMap<&str, &NodeType>,
    instances: &HashMap<&str, &NodeInstance>,
    defects: &mut Vec<WiringDefect>,
) {
    for binding in &instance.bindings {
        let parameter = declared_parameter(instance, declaration, binding, defects);
        let Some(source) = instances.get(binding.source.as_str()) else {
            defects.push(WiringDefect::UnresolvedInstance {
                instance: instance.name.clone(),
                parameter: binding.parameter.clone(),
                unresolved: binding.source.clone(),
            });
            continue;
        };
        if let Some(parameter) = parameter {
            check_binding_type(instance, parameter, binding, source, declarations, defects);
        }
    }
}

/// The parameter `binding` fills, as its instance's type declares it - or
/// nothing, and a defect saying so, where the type declares no parameter by
/// that name.
///
/// Both lists are searched, required and optional, and nothing else is. A
/// requested global is a context type read by declaration rather than a
/// parameter, so a binding spelled like one fills nothing
/// (`DEC_DECLARED_PARAMETERS`).
///
/// Only an instance whose declaration resolved gets here, and that is the only
/// kind whose parameters are knowable: for one of a type nobody supplied,
/// "undeclared" would be invented, and its missing type is in the report
/// already.
// @Every bound parameter declared by its instance's type,IMPL_WIRING_PARAMETER_DECLARED,impl,[CREQ_VALIDATOR_PARAMETER_DECLARED]
fn declared_parameter<'d>(
    instance: &NodeInstance,
    declaration: &'d NodeType,
    binding: &Binding,
    defects: &mut Vec<WiringDefect>,
) -> Option<&'d Parameter> {
    let declared = declaration
        .required
        .iter()
        .chain(&declaration.optional)
        .find(|parameter| parameter.name == binding.parameter);
    if declared.is_none() {
        defects.push(WiringDefect::UndeclaredParameter {
            instance: instance.name.clone(),
            parameter: binding.parameter.clone(),
        });
    }
    declared
}

/// The type declared for a parameter and the type declared for the output wired
/// to it are the same name, exactly.
///
/// Exactly, because every way of loosening the comparison - ignoring case,
/// trimming, matching a prefix - makes two distinct types compare as one, and a
/// node handed the wrong context produces a confident wrong answer rather than
/// failing.
///
/// Two ends are never compared, and each is deliberate. A parameter the type
/// does not declare has no declared type, so its binding never gets here: it is
/// reported as undeclared, and a disagreement with a stand-in type would be
/// invented. And a producer of a node type the definition does not carry has no
/// declared output at all: its missing type is already in the report, and
/// comparing against a stand-in for one - an empty name, a default - would make
/// every wire out of it disagree and send the author to change a type that is
/// not wrong.
///
/// The check is reached for every binding of every instance, including one whose
/// instance already carries an unbound-parameter defect. A walk that moved on
/// after an instance's first defect would hide every disagreement below it.
// @Both ends of a wire declare one context type,IMPL_WIRING_TYPES_AGREE,impl,[CREQ_VALIDATOR_TYPES_AGREE]
fn check_binding_type(
    instance: &NodeInstance,
    parameter: &Parameter,
    binding: &Binding,
    source: &NodeInstance,
    declarations: &HashMap<&str, &NodeType>,
    defects: &mut Vec<WiringDefect>,
) {
    let Some(producer) = declarations.get(source.node_type.as_str()) else {
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

/// The one output a definition designates names one of its instances.
///
/// Only one: a definition designating several is refused for that by the check
/// above, and which of them was meant is what its author decides first.
/// Resolving each as well would name the definition once per designation, and a
/// document cannot hold several (`DEC_ONE_OUTPUT_KEY`).
///
/// Reported as a signature defect of its own rather than as designating
/// nothing, because a count of nought loses the name the author typed - the one
/// thing tying the defect to what has to be changed.
// @The one designated output resolved,IMPL_WIRING_OUTPUT_RESOLVES,impl,[CREQ_VALIDATOR_OUTPUT_RESOLVES]
fn check_output_resolves(
    definition: &WorkflowDefinition,
    instances: &HashMap<&str, &NodeInstance>,
    defects: &mut Vec<WiringDefect>,
) {
    let [output] = definition.designated_outputs.as_slice() else {
        return;
    };
    if !instances.contains_key(output.as_str()) {
        defects.push(WiringDefect::UnresolvedOutput {
            definition: definition.name.clone(),
            unresolved: output.clone(),
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

/// Definitions of every shape, sound and broken alike: node types that may or
/// may not be declared, bindings that may or may not resolve, parameters that
/// may or may not agree on a type, and nought, one or two designated outputs.
///
/// Two things are held fixed rather than generated, and neither is laxity. Each
/// instance carries at most one binding per parameter, because a parameter binds
/// to exactly one output (`DEC_BINDING_BY_PORT`) and a definition binding one
/// twice is outside the model rather than a defective use of it. And instance
/// names are distinct, because two instances sharing a name is one of the shapes
/// recorded as not yet answered, which
/// `TEST_WIRING_MALFORMED_DEFINITION_STILL_REPORTS` holds to the one thing that
/// is settled about it.
#[cfg(test)]
pub(crate) fn any_definition() -> impl Strategy<Value = WorkflowDefinition> {
    const CONTEXT_TYPES: [&str; 2] = ["note", "diff"];
    const PARAMETERS: [&str; 3] = ["p0", "p1", "q0"];

    let declarations = vec((0..=2usize, 0..=1usize, 0..=1usize, 0..2usize), 1..=3);
    let nodes = vec(
        (0..4usize, any::<bool>(), vec((0..3usize, 0..5usize), 0..=3)),
        1..=4,
    );
    let designated = vec(0..5usize, 0..=2);

    (declarations, nodes, designated).prop_map(|(declarations, nodes, designated)| {
        let node_types: Vec<NodeType> = declarations
            .iter()
            .enumerate()
            .map(|(declared, &(required, optional, globals, output))| {
                let required: Vec<(&str, &str)> = PARAMETERS[..required]
                    .iter()
                    .enumerate()
                    .map(|(p, &name)| (name, CONTEXT_TYPES[(declared + p) % 2]))
                    .collect();
                let optional: Vec<(&str, &str)> = PARAMETERS[2..2 + optional]
                    .iter()
                    .map(|&name| (name, CONTEXT_TYPES[declared % 2]))
                    .collect();
                let read: Vec<&str> = ["policy"][..globals].to_vec();
                node_type(&format!("t{declared}"), &required, CONTEXT_TYPES[output])
                    .with_optional(&optional)
                    .with_globals(&read)
            })
            .collect();

        let instances: Vec<NodeInstance> = nodes
            .iter()
            .enumerate()
            .map(|(node, (declared, entry, bindings))| {
                // A type index past the declarations names a type nobody
                // supplied; a source index past the instances names a node the
                // definition does not carry.
                let declared = if *declared < declarations.len() {
                    format!("t{declared}")
                } else {
                    "not-supplied".to_owned()
                };
                let mut bound: Vec<(&str, String)> = Vec::new();
                for &(parameter, source) in bindings {
                    let parameter = PARAMETERS[parameter];
                    if bound.iter().any(|(filled, _)| *filled == parameter) {
                        continue;
                    }
                    let source = if source < nodes.len() {
                        format!("n{source}")
                    } else {
                        "ghost".to_owned()
                    };
                    bound.push((parameter, source));
                }
                let bound: Vec<(&str, &str)> = bound
                    .iter()
                    .map(|(parameter, source)| (*parameter, source.as_str()))
                    .collect();
                let built = instance(&format!("n{node}"), &declared, &bound);
                if *entry { built.into_entry() } else { built }
            })
            .collect();

        let designated: Vec<String> = designated
            .iter()
            .map(|&output| {
                if output < nodes.len() {
                    format!("n{output}")
                } else {
                    "ghost".to_owned()
                }
            })
            .collect();
        let designated: Vec<&str> = designated.iter().map(String::as_str).collect();

        definition(node_types, instances, &designated)
    })
}

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

/// A binding to a parameter its instance's type does not declare, as the report
/// names it.
#[cfg(test)]
fn undeclared(instance: &str, parameter: &str) -> WiringDefect {
    WiringDefect::UndeclaredParameter {
        instance: instance.to_owned(),
        parameter: parameter.to_owned(),
    }
}

/// The one designated output naming no instance, as the report names it.
#[cfg(test)]
fn unresolved_output(output: &str) -> WiringDefect {
    WiringDefect::UnresolvedOutput {
        definition: DEFINITION_NAME.to_owned(),
        unresolved: output.to_owned(),
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

    /// Each instance is of a type of its own declaring required parameters,
    /// optional parameters and requested globals in any number, or of a type
    /// nobody supplied, and binds names from one pool: every name either list
    /// could declare, names spelled like the globals, and a name nothing
    /// declares. Declared and undeclared names therefore sit side by side on one
    /// instance, which is where a check reading one list short, or reading the
    /// globals as parameters, gets some of them wrong and not all.
    ///
    /// What the report should say is worked out from which names each shape
    /// declares, a different computation from the walk under test. Only
    /// undeclared-parameter defects are compared: every binding is wired to an
    /// instance that resolves and agrees on its type, and the unbound
    /// parameters beside them are another case's business.
    #[test]
    fn every_undeclared_parameter_is_reported(
        shapes in vec(
            (0..=2usize, 0..=2usize, 0..=2usize, any::<bool>(), vec(0..7usize, 0..=5)),
            1..=4,
        )
    ) {
        const POOL: [&str; 7] = ["r0", "r1", "o0", "o1", "g0", "g1", "stray"];
        let mut node_types = vec![node_type("source", &[], "note")];
        let mut instances = vec![instance("s", "source", &[])];
        let mut expected = Vec::new();

        for (node, (required, optional, globals, supplied, picks)) in shapes.iter().enumerate() {
            let name = format!("n{node}");
            let required = &POOL[..*required];
            let optional = &POOL[2..2 + optional];
            let declared_type = if *supplied {
                let typed = |names: &[&'static str]| -> Vec<(&str, &str)> {
                    names.iter().map(|&name| (name, "note")).collect()
                };
                let name = format!("t{node}");
                node_types.push(
                    node_type(&name, &typed(required), "note")
                        .with_optional(&typed(optional))
                        .with_globals(&POOL[4..4 + globals]),
                );
                name
            } else {
                "not-supplied".to_owned()
            };

            // Each name at most once, in the order first picked: a parameter
            // bound twice is another requirement's shape.
            let mut bound: Vec<&str> = Vec::new();
            for &pick in picks {
                if !bound.contains(&POOL[pick]) {
                    bound.push(POOL[pick]);
                }
            }
            for &parameter in &bound {
                let is_declared =
                    required.contains(&parameter) || optional.contains(&parameter);
                if *supplied && !is_declared {
                    expected.push(undeclared(&name, parameter));
                }
            }
            let bindings: Vec<(&str, &str)> =
                bound.iter().map(|&parameter| (parameter, "s")).collect();
            instances.push(instance(&name, &declared_type, &bindings));
        }

        let report = validate_wiring(&definition(node_types, instances, &["s"]));
        let found: Vec<WiringDefect> = report
            .into_iter()
            .filter(|defect| matches!(defect, WiringDefect::UndeclaredParameter { .. }))
            .collect();
        prop_assert_eq!(found, expected);
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

    /// A report that grows without saying more is what an author paying per
    /// round trip reads.
    ///
    /// Equality alone is not the whole assertion: a defect reported once per
    /// direction of a binding, or once per check that touched it, carries a
    /// different message about the same wire and would pass it. So no place may
    /// be named twice by defects of one class either.
    #[test]
    fn no_defect_is_reported_twice(workflow in any_definition()) {
        let report = validate_wiring(&workflow);

        for (position, defect) in report.iter().enumerate() {
            for other in &report[position + 1..] {
                prop_assert_ne!(defect, other, "the same defect twice");

                let same_class = std::mem::discriminant(defect) == std::mem::discriminant(other);
                let same_place = defect.instance() == other.instance()
                    && defect.parameter() == other.parameter();
                prop_assert!(
                    !(same_class && same_place),
                    "one place named twice by one class: {:?} and {:?}",
                    defect,
                    other
                );
            }
        }
    }

    /// The control the other cases are measured against: a validator that
    /// refuses every workflow satisfies every requirement that says what must be
    /// refused, and only this one notices.
    ///
    /// The shapes a strict validator refuses by accident are built into every
    /// generated definition rather than left to chance, because a generator
    /// producing only trees would pass against exactly such a validator: a cycle
    /// of two nodes, a pair of nodes no entry node reaches, an unbound optional
    /// parameter, a declared global that no binding carries, an output bound by
    /// several parameters, and an entry node whose required parameter is the
    /// workflow's own rather than a wire.
    #[test]
    fn well_formed_definitions_pass(workflow in well_formed_definition()) {
        prop_assert_eq!(validate_wiring(&workflow), Vec::new());
    }
}

/// Definitions that are well formed by construction, varying in what is wired
/// on top of a skeleton carrying every legal shape a strict validator refuses.
#[cfg(test)]
fn well_formed_definition() -> impl Strategy<Value = WorkflowDefinition> {
    vec((0..2usize, any::<bool>()), 0..=3).prop_map(|extras| {
        let node_types = vec![
            node_type("seed_note", &[], "note"),
            node_type("seed_diff", &[], "diff"),
            node_type("pass_note", &[("input", "note")], "note"),
            node_type("pass_diff", &[("input", "diff")], "diff"),
            node_type("lenient", &[], "note")
                .with_optional(&[("hint", "note")])
                .with_globals(&["policy"]),
        ];

        let mut instances = vec![
            instance("seed", "seed_note", &[]),
            instance("seed_diff", "seed_diff", &[]),
            // An entry node: its required parameter is the workflow's own.
            instance("entry", "pass_note", &[]).into_entry(),
            // A cycle of two, which no entry node reaches. Both are legal.
            instance("loop_a", "pass_note", &[("input", "loop_b")]),
            instance("loop_b", "pass_note", &[("input", "loop_a")]),
            // One output bound by several parameters.
            instance("fan_x", "pass_note", &[("input", "seed")]),
            instance("fan_y", "pass_note", &[("input", "seed")]),
            // An optional left unbound, and a declared global nothing carries.
            instance("lax", "lenient", &[]),
        ];

        for (extra, &(kind, bind_optional)) in extras.iter().enumerate() {
            instances.push(match kind {
                0 => instance(&format!("x{extra}"), "pass_note", &[("input", "seed")]),
                _ => instance(&format!("x{extra}"), "pass_diff", &[("input", "seed_diff")]),
            });
            if bind_optional {
                // The same optional parameter, this time wired, and agreeing.
                instances.push(instance(
                    &format!("l{extra}"),
                    "lenient",
                    &[("hint", "seed")],
                ));
            }
        }

        definition(node_types, instances, &["fan_x"])
    })
}

#[test]
fn all_four_classes_reported() {
    let types = vec![
        node_type("source", &[], "note"),
        node_type("differ", &[], "diff"),
        node_type("pair", &[("left", "note"), ("right", "note")], "note"),
        node_type("sink", &[("input", "note")], "note"),
    ];
    let instances = vec![
        instance("a", "differ", &[]),
        // Two defects on one instance: an unbound required parameter and a wire
        // whose ends disagree. A walk that moves on once an instance has a
        // defect reports three rather than four.
        instance("b", "pair", &[("left", "a")]),
        instance("c", "sink", &[("input", "ghost")]),
    ];
    // And no designated output, so a validator refusing the definition for its
    // signature before examining any wiring reports one rather than four.
    let broken = definition(types, instances, &[]);

    assert_eq!(
        validate_wiring(&broken),
        vec![
            unbound("b", "right"),
            disagreement("b", "left", "note", "diff"),
            unresolved("c", "input", "ghost"),
            signature_defect(0),
        ]
    );
}

#[test]
fn malformed_definition_still_reports() {
    let types = vec![
        node_type("source", &[], "note"),
        node_type("sink", &[("input", "note")], "note"),
    ];

    let mut declared_both_ways = node_type("both", &[("input", "note")], "note");
    declared_both_ways.optional = crate::workflow::parameters(&[("input", "diff")]);

    let malformed = [
        // Two node types sharing one name within the definition.
        definition(
            vec![
                node_type("source", &[], "note"),
                node_type("sink", &[("input", "note")], "note"),
                node_type("sink", &[("other", "diff")], "diff"),
            ],
            vec![
                instance("a", "source", &[]),
                instance("b", "sink", &[("input", "a")]),
            ],
            &["b"],
        ),
        // One parameter declared as both required and optional, bound and
        // unbound.
        definition(
            vec![node_type("source", &[], "note"), declared_both_ways],
            vec![
                instance("a", "source", &[]),
                instance("b", "both", &[("input", "a")]),
                instance("c", "both", &[]),
            ],
            &["b"],
        ),
        // Two instances sharing one name, so that a binding to it resolves to
        // both.
        definition(
            types.clone(),
            vec![
                instance("a", "source", &[]),
                instance("a", "sink", &[("input", "a")]),
            ],
            &["a"],
        ),
        // A binding naming an empty instance name.
        definition(
            types.clone(),
            vec![instance("b", "sink", &[("input", "")])],
            &["b"],
        ),
        // Node types the definition has no instances of.
        definition(types, Vec::new(), &[]),
    ];

    for workflow in &malformed {
        // Not ending the run is the whole assertion, and deliberately the whole
        // of it: which defect the first two earn is recorded as still open, and
        // asserting a class here would pin behaviour no requirement asks for.
        // The report is read rather than discarded, so a defect that cannot say
        // anything fails this.
        for defect in validate_wiring(workflow) {
            assert!(!defect.to_string().is_empty(), "{defect:?} says nothing");
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
fn undeclared_parameter_is_reported() {
    let types = vec![
        node_type("source", &[], "note"),
        node_type("differ", &[], "diff"),
        node_type("lenient", &[("input", "note")], "note")
            .with_optional(&[("hint", "note")])
            .with_globals(&["policy"]),
    ];
    let instances = vec![
        instance("a", "source", &[]),
        // What the undeclared bindings are wired from: an output no declared
        // parameter shares, so comparing one against a stand-in type would
        // report a disagreement nobody could fix.
        instance("z", "differ", &[]),
        // A typo of the optional parameter, and a binding spelled like the
        // requested global. Nothing else about b is wrong.
        instance(
            "b",
            "lenient",
            &[("input", "a"), ("hnit", "z"), ("policy", "z")],
        ),
        // A typo of the required parameter, which leaves it unbound.
        instance("c", "lenient", &[("inptu", "z")]),
        // An undeclared parameter wired from an instance that is not there.
        instance("d", "lenient", &[("input", "a"), ("stray", "ghost")]),
    ];

    // c and d are the two where one defect hides another: the unbound
    // parameter says what is missing and not which binding was meant for it,
    // and the unresolved source is a second fix beside the renamed parameter.
    assert_eq!(
        validate_wiring(&definition(types, instances, &["b"])),
        vec![
            undeclared("b", "hnit"),
            undeclared("b", "policy"),
            unbound("c", "input"),
            undeclared("c", "inptu"),
            undeclared("d", "stray"),
            unresolved("d", "stray", "ghost"),
        ]
    );
}

#[test]
fn declared_parameters_pass() {
    // The optional parameter bound beside the required one: a check searching
    // the required list alone reports it, and one searching the optional list
    // alone reports the other.
    let fed = definition(
        vec![
            node_type("source", &[], "note"),
            node_type("lenient", &[("input", "note")], "note").with_optional(&[("hint", "note")]),
        ],
        vec![
            instance("a", "source", &[]),
            instance("b", "lenient", &[("input", "a"), ("hint", "a")]),
        ],
        &["b"],
    );
    assert_eq!(validate_wiring(&fed), Vec::new());
}

#[test]
fn output_naming_nothing_is_reported() {
    let types = vec![
        node_type("source", &[], "note"),
        node_type("sink", &[("input", "note")], "note"),
    ];
    let instances = vec![
        instance("a", "source", &[]),
        instance("b", "sink", &[("input", "a")]),
    ];

    // Sound wiring and one designation, so a count finds nothing wrong: the
    // defect is the name, and it is echoed rather than counted as nought.
    let renamed = definition(types.clone(), instances.clone(), &["nowhere"]);
    assert_eq!(
        validate_wiring(&renamed),
        vec![unresolved_output("nowhere")]
    );

    // Two designations are refused for being two, and neither is resolved:
    // resolving each would name the definition once per name.
    let one_of_two = definition(types.clone(), instances.clone(), &["nowhere", "b"]);
    assert_eq!(validate_wiring(&one_of_two), vec![signature_defect(2)]);
    let neither = definition(types, instances, &["nowhere", "elsewhere"]);
    assert_eq!(validate_wiring(&neither), vec![signature_defect(2)]);
}

#[test]
fn resolving_outputs_pass() {
    // An output naming an entry node.
    let entered = definition(
        vec![node_type("pass", &[("input", "note")], "note")],
        vec![instance("start", "pass", &[]).into_entry()],
        &["start"],
    );
    assert_eq!(validate_wiring(&entered), Vec::new());

    // An output naming an instance whose name is not its type's. A check
    // resolving the output against the node types passes every definition
    // whose instances are named after their types, which is how short examples
    // get written.
    let named_apart = definition(
        vec![
            node_type("source", &[], "note"),
            node_type("sink", &[("input", "note")], "note"),
        ],
        vec![
            instance("fetch", "source", &[]),
            instance("result", "sink", &[("input", "fetch")]),
        ],
        &["result"],
    );
    assert_eq!(validate_wiring(&named_apart), Vec::new());
}

#[test]
fn empty_definition_is_refused_for_its_signature() {
    let report = validate_wiring(&definition(Vec::new(), Vec::new(), &[]));

    // The count and the class, not merely that something was reported: an empty
    // definition has nothing to wire and nothing to designate, so it is the case
    // where two requirements could quietly both fire.
    assert_eq!(report, vec![signature_defect(0)]);
}
