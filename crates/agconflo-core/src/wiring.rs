//! The wiring validator: one workflow definition compared against the node
//! types it names, and everything wrong with it reported at once.

use std::collections::{HashMap, HashSet};

use crate::defect::WiringDefect;
use crate::workflow::{Binding, NodeInstance, NodeType, Parameter, WorkflowDefinition};

/// Every wiring defect `definition` carries, and nothing at all for one that
/// carries none, in the definition's own order: each instance in turn, then the
/// signature.
///
/// Nothing here returns early, and the signature is checked with everything
/// else. An instance whose name another carries too, and the wires of a
/// parameter bound more than once, are reported and not looked behind.
// @Every instance walked and everything collected,IMPL_WIRING_WALK,impl,[CREQ_VALIDATOR_EVERY_DEFECT, CREQ_VALIDATOR_ACCEPTS_WELL_FORMED]
pub fn validate_wiring(definition: &WorkflowDefinition) -> Vec<WiringDefect> {
    let mut defects = Vec::new();
    let declarations = by_name(&definition.node_types, |declared| &declared.name);
    let instances = by_name(&definition.instances, |instance| &instance.name);
    let shared = repeated(
        definition
            .instances
            .iter()
            .map(|instance| instance.name.as_str()),
    );

    let mut reported = HashSet::new();
    for instance in &definition.instances {
        if shared.contains(instance.name.as_str()) {
            report_shared_name(instance, &mut reported, &mut defects);
            continue;
        }
        check_instance(instance, &declarations, &instances, &shared, &mut defects);
        check_calls(instance, &declarations, &mut defects);
    }
    check_signature(definition, &mut defects);
    check_output_resolves(definition, &instances, &mut defects);

    defects
}

/// A lookup from a name to the first thing carrying it.
fn by_name<'a, T>(items: &'a [T], name: impl Fn(&'a T) -> &'a str) -> HashMap<&'a str, &'a T> {
    let mut by_name = HashMap::new();
    for item in items {
        by_name.entry(name(item)).or_insert(item);
    }
    by_name
}

/// The names that occur more than once among `names`.
fn repeated<'a>(names: impl IntoIterator<Item = &'a str>) -> HashSet<&'a str> {
    let mut seen = HashSet::new();
    names
        .into_iter()
        .filter(|&name| !seen.insert(name))
        .collect()
}

/// An instance whose name another instance carries too: the name is reported
/// the first time it is met, once however many instances carry it, and the
/// instance is not looked at further.
// @A shared name reported once and nothing behind it checked,IMPL_WIRING_INSTANCE_NAMED_ONCE,impl,[CREQ_VALIDATOR_INSTANCE_NAMED_ONCE]
fn report_shared_name<'a>(
    instance: &'a NodeInstance,
    reported: &mut HashSet<&'a str>,
    defects: &mut Vec<WiringDefect>,
) {
    if reported.insert(instance.name.as_str()) {
        defects.push(WiringDefect::RepeatedInstance {
            instance: instance.name.clone(),
        });
    }
}

/// One instance against the declaration it names. An instance without one is
/// reported and left, and the rest of the definition is still walked.
// @An instance checked against the type it names,IMPL_WIRING_INSTANCE_TYPE,impl,[CREQ_VALIDATOR_BINDING_RESOLVES]
fn check_instance(
    instance: &NodeInstance,
    declarations: &HashMap<&str, &NodeType>,
    instances: &HashMap<&str, &NodeInstance>,
    shared: &HashSet<&str>,
    defects: &mut Vec<WiringDefect>,
) {
    let Some(declaration) = declarations.get(instance.node_type.as_str()) else {
        defects.push(WiringDefect::UnresolvedNodeType {
            instance: instance.name.clone(),
            unresolved: instance.node_type.clone(),
        });
        return;
    };

    check_bindings(
        instance,
        declaration,
        declarations,
        instances,
        shared,
        defects,
    );
}

/// Every binding fills a parameter its instance's type declares, and names an
/// instance the definition carries: one defect per binding, both ends looked at
/// whatever the other is. A parameter is looked at once, at its first binding,
/// and a source several instances share has its type left uncompared.
// @Every binding's source resolved,IMPL_WIRING_BINDING_SOURCE,impl,[CREQ_VALIDATOR_BINDING_RESOLVES]
fn check_bindings(
    instance: &NodeInstance,
    declaration: &NodeType,
    declarations: &HashMap<&str, &NodeType>,
    instances: &HashMap<&str, &NodeInstance>,
    shared: &HashSet<&str>,
    defects: &mut Vec<WiringDefect>,
) {
    let bound_twice = repeated(
        instance
            .bindings
            .iter()
            .map(|binding| binding.parameter.as_str()),
    );
    let mut looked_at = HashSet::new();

    for binding in &instance.bindings {
        if !looked_at.insert(binding.parameter.as_str()) {
            continue;
        }
        let parameter = declared_parameter(instance, declaration, binding, defects);
        if bound_more_than_once(instance, binding, &bound_twice, defects) {
            continue;
        }
        let Some(source) = instances.get(binding.source.as_str()) else {
            defects.push(WiringDefect::UnresolvedInstance {
                instance: instance.name.clone(),
                parameter: binding.parameter.clone(),
                unresolved: binding.source.clone(),
            });
            continue;
        };
        if shared.contains(binding.source.as_str()) {
            continue;
        }
        if let Some(parameter) = parameter {
            check_binding_type(instance, parameter, binding, source, declarations, defects);
        }
    }
}

/// Whether the parameter `binding` fills is bound more than once on its
/// instance - and if it is, a defect saying so. Its caller then checks none of
/// that parameter's bindings for a source or a type.
// @A parameter bound twice reported once and none of its wires checked,IMPL_WIRING_PARAMETER_BOUND_ONCE,impl,[CREQ_VALIDATOR_PARAMETER_BOUND_ONCE]
fn bound_more_than_once(
    instance: &NodeInstance,
    binding: &Binding,
    bound_twice: &HashSet<&str>,
    defects: &mut Vec<WiringDefect>,
) -> bool {
    let twice = bound_twice.contains(binding.parameter.as_str());
    if twice {
        defects.push(WiringDefect::RepeatedBinding {
            instance: instance.name.clone(),
            parameter: binding.parameter.clone(),
        });
    }
    twice
}

/// The parameter `binding` fills, as its instance's type declares it in either
/// list - or nothing, and a defect saying so. A requested global is not a
/// parameter. Only an instance whose declaration resolved gets here.
// @Every bound parameter declared by its instance's type,IMPL_WIRING_PARAMETER_DECLARED,impl,[CREQ_VALIDATOR_PARAMETER_DECLARED],[DEC_EVERY_INPUT_REQUIRED]
fn declared_parameter<'d>(
    instance: &NodeInstance,
    declaration: &'d NodeType,
    binding: &Binding,
    defects: &mut Vec<WiringDefect>,
) -> Option<&'d Parameter> {
    let declared = declaration
        .required
        .iter()
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
/// to it are the same name, exactly. Reached for every binding whose wire can be
/// compared; a parameter the type does not declare, and a producer of a node
/// type the definition does not carry, are never compared.
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

/// Every node type an instance declares a call to is one the definition
/// carries, and has a name matching `^[a-zA-Z0-9_-]{1,64}$`, whole and counted
/// in characters. Checked whatever the instance's own node type turned out to
/// be, the name whether or not it resolves; a name listed twice is looked at
/// once.
// @Every call resolved and its name one every provider accepts,IMPL_WIRING_CALLS,impl,[CREQ_VALIDATOR_CALL_RESOLVES, CREQ_VALIDATOR_CALL_NAME],[DEC_TOOL_NAMES_PORTABLE]
fn check_calls(
    instance: &NodeInstance,
    declarations: &HashMap<&str, &NodeType>,
    defects: &mut Vec<WiringDefect>,
) {
    let mut looked_at = HashSet::new();
    for call in &instance.calls {
        if !looked_at.insert(call.as_str()) {
            continue;
        }
        if !declarations.contains_key(call.as_str()) {
            defects.push(WiringDefect::UnresolvedCall {
                instance: instance.name.clone(),
                unresolved: call.clone(),
            });
        }
        if !portable(call) {
            defects.push(WiringDefect::UnportableCallName {
                instance: instance.name.clone(),
                name: call.clone(),
            });
        }
    }
}

/// Whether `name` matches `^[a-zA-Z0-9_-]{1,64}$`.
pub(crate) fn portable(name: &str) -> bool {
    (1..=64).contains(&name.len())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// The definition designates exactly one output: counted rather than tested for
/// absence, and checked in the walk with everything else.
// @Exactly one designated output,IMPL_WIRING_SIGNATURE,impl,[CREQ_VALIDATOR_ONE_OUTPUT],[DEC_SIGNATURE_IS_WHAT_NOTHING_BINDS]
fn check_signature(definition: &WorkflowDefinition, defects: &mut Vec<WiringDefect>) {
    let designated = definition.designated_outputs.len();
    if designated != 1 {
        defects.push(WiringDefect::SignatureOutputs {
            definition: definition.name.clone(),
            designated,
        });
    }
}

/// The one output a definition designates names one of its instances, checked
/// only when there is exactly one, and reported with the name as written.
// @The one designated output resolved,IMPL_WIRING_OUTPUT_RESOLVES,impl,[CREQ_VALIDATOR_OUTPUT_RESOLVES],[DEC_ONE_OUTPUT_KEY]
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
// Bare functions named after their test cases.

#[cfg(test)]
use crate::workflow::{DEFINITION_NAME, definition, instance, node_type};
#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::prelude::*;

/// Definitions of every shape, sound and broken alike: node types that may or
/// may not be declared, bindings that may or may not resolve, parameters that
/// may or may not be declared or agree on a type, instance names that may be
/// shared, parameters that may be bound more than once, and nought, one or two
/// designated outputs that may or may not name an instance.
#[cfg(test)]
pub(crate) fn any_definition() -> impl Strategy<Value = WorkflowDefinition> {
    const CONTEXT_TYPES: [&str; 2] = ["note", "diff"];
    const PARAMETERS: [&str; 3] = ["p0", "p1", "q0"];
    // Calls to node types that may or may not be declared, and to one no
    // provider would accept as a tool's name, which is also declared nowhere.
    const CALLS: [&str; 4] = ["t0", "t2", "t9", "a.b"];

    let declarations = vec((0..=2usize, 0..=1usize, 0..2usize), 1..=3);
    // Per node: its type, its bindings, whether a
    // parameter may be bound more than once, and an earlier node whose name it
    // takes - an index at or past its own position keeps a name of its own.
    // @The range a shared name is drawn from,TRACE_WIRING_GENERATOR_RANGE,trace,[],[NOTE_WIRING_GENERATOR_RANGE]
    let nodes = vec(
        (
            0..4usize,
            vec((0..3usize, 0..5usize), 0..=3),
            any::<bool>(),
            0..10usize,
            vec(0..CALLS.len(), 0..=2),
        ),
        1..=4,
    );
    let designated = vec(0..5usize, 0..=2);

    (declarations, nodes, designated).prop_map(|(declarations, nodes, designated)| {
        let node_types: Vec<NodeType> = declarations
            .iter()
            .enumerate()
            .map(|(declared, &(required, globals, output))| {
                let required: Vec<(&str, &str)> = PARAMETERS[..required]
                    .iter()
                    .enumerate()
                    .map(|(p, &name)| (name, CONTEXT_TYPES[(declared + p) % 2]))
                    .collect();
                let read: Vec<&str> = ["policy"][..globals].to_vec();
                node_type(&format!("t{declared}"), &required, CONTEXT_TYPES[output])
                    .with_globals(&read)
            })
            .collect();

        // A node taking an earlier node's name shares it, with that node and
        // with any other that took it too.
        let mut names: Vec<String> = Vec::new();
        for (node, &(_, _, _, earlier, _)) in nodes.iter().enumerate() {
            let name = if earlier < node {
                names[earlier].clone()
            } else {
                format!("n{node}")
            };
            names.push(name);
        }

        let instances: Vec<NodeInstance> = nodes
            .iter()
            .zip(&names)
            .map(|((declared, bindings, repeats, _, calls), name)| {
                // A type index past the declarations names a type nobody
                // supplied; a source index past the instances names a node the
                // definition does not carry, and so may one naming a node whose
                // own name was taken from an earlier one.
                let declared = if *declared < declarations.len() {
                    format!("t{declared}")
                } else {
                    "not-supplied".to_owned()
                };
                let mut bound: Vec<(&str, String)> = Vec::new();
                for &(parameter, source) in bindings {
                    let parameter = PARAMETERS[parameter];
                    if !repeats && bound.iter().any(|(filled, _)| *filled == parameter) {
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
                let calls: Vec<&str> = calls.iter().map(|&call| CALLS[call]).collect();
                instance(name, &declared, &bound).with_calls(&calls)
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

/// A name several instances share, as the report names it.
#[cfg(test)]
fn shared(name: &str) -> WiringDefect {
    WiringDefect::RepeatedInstance {
        instance: name.to_owned(),
    }
}

/// A parameter bound more than once on one instance, as the report names it.
#[cfg(test)]
fn bound_twice(instance: &str, parameter: &str) -> WiringDefect {
    WiringDefect::RepeatedBinding {
        instance: instance.to_owned(),
        parameter: parameter.to_owned(),
    }
}

#[cfg(test)]
proptest! {
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

    /// Each instance is of a type of its own declaring parameters and requested
    /// globals in any number, or of a type nobody supplied, and binds names from
    /// one pool: every name it could declare, names spelled like the globals, and
    /// names nothing declares. Only undeclared-parameter defects are compared.
    #[test]
    fn every_undeclared_parameter_is_reported(
        shapes in vec(
            (0..=2usize, 0..=2usize, any::<bool>(), vec(0..7usize, 0..=5)),
            1..=4,
        )
    ) {
        const POOL: [&str; 7] = ["r0", "r1", "o0", "o1", "g0", "g1", "stray"];
        let mut node_types = vec![node_type("source", &[], "note")];
        let mut instances = vec![instance("s", "source", &[])];
        let mut expected = Vec::new();

        for (node, (required, globals, supplied, picks)) in shapes.iter().enumerate() {
            let name = format!("n{node}");
            let required = &POOL[..*required];
            let declared_type = if *supplied {
                let typed = |names: &[&'static str]| -> Vec<(&str, &str)> {
                    names.iter().map(|&name| (name, "note")).collect()
                };
                let name = format!("t{node}");
                node_types.push(
                    node_type(&name, &typed(required), "note").with_globals(&POOL[4..4 + globals]),
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
                if *supplied && !required.contains(&parameter) {
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
    /// first by each way a comparison gets loosened: the same name, upper-cased,
    /// with a leading or a trailing space, and with a character appended.
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

    /// No defect is reported twice, and no place is named twice by defects of one
    /// class.
    #[test]
    fn no_defect_is_reported_twice(workflow in any_definition()) {
        let report = validate_wiring(&workflow);

        for (position, defect) in report.iter().enumerate() {
            for other in &report[position + 1..] {
                prop_assert_ne!(defect, other, "the same defect twice");

                let same_class = std::mem::discriminant(defect) == std::mem::discriminant(other);
                let same_place = defect.instance() == other.instance()
                    && defect.parameter() == other.parameter()
                    && defect.call() == other.call();
                prop_assert!(
                    !(same_class && same_place),
                    "one place named twice by one class: {:?} and {:?}",
                    defect,
                    other
                );
            }
        }
    }

    /// The control: generated sound definitions, each holding a cycle of two
    /// nodes, a pair nothing reaches, an unbound global, an output bound by
    /// several parameters, and an instance whose parameter nothing binds, are
    /// reported clean.
    #[test]
    fn well_formed_definitions_pass(workflow in well_formed_definition()) {
        prop_assert_eq!(validate_wiring(&workflow), Vec::new());
    }
}

/// Definitions that are well formed by construction, varying in what is wired
/// on top of a skeleton carrying every legal shape a strict validator refuses.
#[cfg(test)]
pub(crate) fn well_formed_definition() -> impl Strategy<Value = WorkflowDefinition> {
    vec(0..2usize, 0..=3).prop_map(|extras| {
        let node_types = vec![
            node_type("seed_note", &[], "note"),
            node_type("seed_diff", &[], "diff"),
            node_type("pass_note", &[("input", "note")], "note"),
            node_type("pass_diff", &[("input", "diff")], "diff"),
            node_type("lenient", &[], "note").with_globals(&["policy"]),
        ];

        let mut instances = vec![
            instance("seed", "seed_note", &[]),
            instance("seed_diff", "seed_diff", &[]),
            // A parameter nothing binds, which is the workflow's own.
            instance("given", "pass_note", &[]),
            // A cycle of two, which nothing reaches. Both are legal.
            instance("loop_a", "pass_note", &[("input", "loop_b")]),
            instance("loop_b", "pass_note", &[("input", "loop_a")]),
            // One output bound by several parameters.
            instance("fan_x", "pass_note", &[("input", "seed")]),
            instance("fan_y", "pass_note", &[("input", "seed")]),
            // A declared global nothing carries - and calls, one of them listed
            // twice, to node types it carries.
            instance("lax", "lenient", &[]).with_calls(&["pass_note", "seed_diff", "pass_note"]),
        ];

        for (extra, &kind) in extras.iter().enumerate() {
            instances.push(match kind {
                0 => instance(&format!("x{extra}"), "pass_note", &[("input", "seed")]),
                _ => instance(&format!("x{extra}"), "pass_diff", &[("input", "seed_diff")]),
            });
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
        // Two defects on one instance: a binding to a parameter its type does
        // not declare and a wire whose ends disagree.
        instance("b", "pair", &[("left", "a"), ("centre", "a")]),
        instance("c", "sink", &[("input", "ghost")]),
    ];
    // And no designated output, so a validator refusing the definition for its
    // signature before examining any wiring reports one rather than four.
    let broken = definition(types, instances, &[]);

    assert_eq!(
        validate_wiring(&broken),
        vec![
            disagreement("b", "left", "note", "diff"),
            undeclared("b", "centre"),
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
        // Returning a report whose every defect says something is the whole
        // assertion.
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

    // Both names are asserted on.
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
            // instances.
            instance("b", "pair", &[("left", "a"), ("right", "a")]),
            instance("c", "pair", &[("left", "a"), ("right", "b")]),
        ],
        &["c"],
    );

    assert_eq!(validate_wiring(&fanned), Vec::new());
}

#[test]
fn degenerate_declarations_pass() {
    let types = vec![
        // A global that no binding carries.
        node_type("lenient", &[], "note").with_globals(&["policy"]),
        // A type declaring no parameters at all.
        node_type("bare", &[], "note"),
        // Parameters nothing binds, which the run is given when it starts.
        node_type("taking", &[("first", "note"), ("second", "note")], "note"),
    ];
    let instances = vec![
        instance("a", "lenient", &[]),
        instance("b", "bare", &[]),
        instance("c", "taking", &[]),
        instance("d", "taking", &[("first", "b")]),
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

    // Nothing but the wire to nowhere is reported.
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
        instance("c", "sink", &[("input", "gone")]),
    ];
    let report = validate_wiring(&definition(types, instances, &["b"]));

    // Nothing else about that instance is reported, and c's own defect is in the
    // same report.
    assert_eq!(
        report,
        vec![
            WiringDefect::UnresolvedNodeType {
                instance: "a".to_owned(),
                unresolved: "not-supplied".to_owned(),
            },
            unresolved("c", "input", "gone"),
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
    // else.
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
    // A workflow whose every parameter is bound, which is legal.
    let closed = definition(
        vec![node_type("source", &[], "note")],
        vec![instance("a", "source", &[])],
        &["a"],
    );
    assert_eq!(validate_wiring(&closed), Vec::new());

    // A designated output whose node also feeds another node, the ordinary shape
    // of a loop.
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
        node_type("lenient", &[("input", "note")], "note").with_globals(&["policy"]),
    ];
    let instances = vec![
        instance("a", "source", &[]),
        // What the undeclared bindings are wired from: an output no declared
        // parameter shares.
        instance("z", "differ", &[]),
        // A name no parameter has, and a binding spelled like the requested
        // global. Nothing else about b is wrong.
        instance(
            "b",
            "lenient",
            &[("input", "a"), ("hnit", "z"), ("policy", "z")],
        ),
        // A typo of the required parameter.
        instance("c", "lenient", &[("inptu", "z")]),
        // An undeclared parameter wired from an instance that is not there.
        instance("d", "lenient", &[("input", "a"), ("stray", "ghost")]),
    ];

    // c and d are the two where one defect hides another.
    assert_eq!(
        validate_wiring(&definition(types, instances, &["b"])),
        vec![
            undeclared("b", "hnit"),
            undeclared("b", "policy"),
            undeclared("c", "inptu"),
            undeclared("d", "stray"),
            unresolved("d", "stray", "ghost"),
        ]
    );
}

#[test]
fn declared_parameters_pass() {
    // Both parameters bound, each to a source of its type.
    let fed = definition(
        vec![
            node_type("source", &[], "note"),
            node_type("lenient", &[("input", "note"), ("hint", "note")], "note"),
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
    // An output naming an instance with a parameter nothing binds.
    let entered = definition(
        vec![node_type("pass", &[("input", "note")], "note")],
        vec![instance("start", "pass", &[])],
        &["start"],
    );
    assert_eq!(validate_wiring(&entered), Vec::new());

    // An output naming an instance whose name is not its type's.
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
fn shared_instance_name_is_reported() {
    let types = vec![
        node_type("source", &[], "note"),
        node_type("differ", &[], "diff"),
        node_type("sink", &[("input", "note")], "note"),
    ];
    // Four instances named a, of the types given: two producing different
    // context types, one with a required parameter nothing binds, one of a type
    // nobody supplied. Then a wire from a into a parameter declared for one of
    // those types, the output designating a, and c, whose own wire to nowhere
    // shows the walk goes on.
    let sharing = |copies: [&str; 4]| {
        let mut instances: Vec<NodeInstance> = copies
            .iter()
            .map(|&node_type| instance("a", node_type, &[]))
            .collect();
        instances.push(instance("b", "sink", &[("input", "a")]));
        instances.push(instance("c", "sink", &[("input", "gone")]));
        definition(types.clone(), instances, &["a"])
    };

    // Two orders, the producers swapped between them and a different instance
    // first in each.
    for copies in [
        ["differ", "source", "sink", "not-supplied"],
        ["not-supplied", "sink", "source", "differ"],
    ] {
        assert_eq!(
            validate_wiring(&sharing(copies)),
            vec![shared("a"), unresolved("c", "input", "gone")],
            "with the instances named a in the order {copies:?}"
        );
    }
}

#[test]
fn names_of_different_kinds_pass() {
    let named_alike = definition(
        vec![
            node_type("source", &[], "note"),
            node_type("sink", &[("input", "note")], "note"),
        ],
        vec![
            // Named like its own node type.
            instance("source", "source", &[]),
            // Named like a parameter - the very one it is bound through.
            instance("input", "source", &[]),
            instance("sink", "sink", &[("input", "input")]),
            // So each node type has two instances, under different names.
            instance("other", "sink", &[("input", "source")]),
        ],
        &["sink"],
    );

    // Names shared across kinds and never within one: reported clean.
    assert_eq!(validate_wiring(&named_alike), Vec::new());
}

#[test]
fn parameter_bound_twice_is_reported() {
    let types = vec![
        node_type("source", &[], "note"),
        node_type("differ", &[], "diff"),
        node_type("sink", &[("input", "note")], "note"),
    ];
    let instances = vec![
        instance("a", "source", &[]),
        instance("z", "differ", &[]),
        // Bound to two instances that are not there.
        instance("b", "sink", &[("input", "g1"), ("input", "g2")]),
        // Bound three times, to one sound source.
        instance(
            "c",
            "sink",
            &[("input", "a"), ("input", "a"), ("input", "a")],
        ),
        // Bound to two sound sources, the second disagreeing on its type.
        instance("d", "sink", &[("input", "a"), ("input", "z")]),
        // An undeclared parameter bound twice, beside a sound binding.
        instance(
            "e",
            "sink",
            &[("stray", "a"), ("input", "a"), ("stray", "a")],
        ),
    ];

    // b, c and d each once, and e's undeclared parameter still reported.
    assert_eq!(
        validate_wiring(&definition(types, instances, &["a"])),
        vec![
            bound_twice("b", "input"),
            bound_twice("c", "input"),
            bound_twice("d", "input"),
            undeclared("e", "stray"),
            bound_twice("e", "stray"),
        ]
    );
}

#[test]
fn singly_bound_parameters_pass() {
    let fanned = definition(
        vec![
            node_type("source", &[], "note"),
            node_type("sink", &[("input", "note")], "note"),
            node_type("pair", &[("left", "note"), ("right", "note")], "note"),
        ],
        vec![
            instance("a", "source", &[]),
            // One parameter name, bound once on each of several instances.
            instance("b", "sink", &[("input", "a")]),
            instance("c", "sink", &[("input", "a")]),
            // One output bound by two parameters of one instance.
            instance("d", "pair", &[("left", "a"), ("right", "a")]),
        ],
        &["d"],
    );

    // A check counting parameter names across the definition refuses b and c,
    // and one counting sources within an instance refuses d.
    assert_eq!(validate_wiring(&fanned), Vec::new());
}

#[test]
fn name_defects_reported_together() {
    let types = vec![
        node_type("source", &[], "note"),
        node_type("lenient", &[("input", "note")], "note"),
    ];
    let instances = vec![
        // A name two instances share, first in the definition.
        instance("a", "source", &[]),
        instance("a", "source", &[]),
        instance("s", "source", &[]),
        // On one instance: a name it does not declare, bound twice.
        instance("b", "lenient", &[("hnit", "s"), ("hnit", "s")]),
    ];
    // And an output naming no instance.
    let broken = definition(types, instances, &["nowhere"]);

    // The shared name, then two defects for b, then the output.
    assert_eq!(
        validate_wiring(&broken),
        vec![
            shared("a"),
            undeclared("b", "hnit"),
            bound_twice("b", "hnit"),
            unresolved_output("nowhere"),
        ]
    );
}

#[test]
fn empty_definition_is_refused_for_its_signature() {
    let report = validate_wiring(&definition(Vec::new(), Vec::new(), &[]));

    // The count and the class: exactly one signature defect.
    assert_eq!(report, vec![signature_defect(0)]);
}

#[cfg(test)]
fn unresolved_call(instance: &str, unresolved: &str) -> WiringDefect {
    WiringDefect::UnresolvedCall {
        instance: instance.to_owned(),
        unresolved: unresolved.to_owned(),
    }
}

#[cfg(test)]
fn unportable(instance: &str, name: &str) -> WiringDefect {
    WiringDefect::UnportableCallName {
        instance: instance.to_owned(),
        name: name.to_owned(),
    }
}

#[test]
fn call_to_missing_type_is_reported() {
    let types = vec![
        node_type("source", &[], "note"),
        node_type("sink", &[("input", "note")], "note"),
    ];
    let instances = vec![
        // Two calls to node types nobody declares, beside one that resolves.
        instance("a", "source", &[]).with_calls(&["ghost", "sink", "phantom"]),
        // One more, on another instance: one defect per call, not per workflow.
        instance("b", "sink", &[("input", "a")]).with_calls(&["ghost"]),
        // An unrelated binding defect, in the same report.
        instance("c", "sink", &[("input", "nowhere")]),
        // An instance whose own node type is missing, and which calls nothing:
        // its defect is the one it always was, and nothing is added for calls.
        instance("d", "missing", &[]),
    ];
    let workflow = definition(types, instances, &["b"]);
    let report = validate_wiring(&workflow);

    assert_eq!(
        report,
        vec![
            unresolved_call("a", "ghost"),
            unresolved_call("a", "phantom"),
            unresolved_call("b", "ghost"),
            unresolved("c", "input", "nowhere"),
            WiringDefect::UnresolvedNodeType {
                instance: "d".to_owned(),
                unresolved: "missing".to_owned(),
            },
        ]
    );
    // The place is values, the call among them.
    assert_eq!(
        (
            report[1].instance(),
            report[1].parameter(),
            report[1].call()
        ),
        (Some("a"), None, Some("phantom"))
    );

    // A run of it is refused before anything is offered, carrying all five.
    match crate::Run::<std::convert::Infallible>::start(&workflow, crate::Arguments::new(), 10) {
        Err(crate::StartRefusal::Wiring(defects)) => assert_eq!(defects, report),
        other => panic!("a workflow calling a missing node type does not start: {other:?}"),
    }
}

#[test]
fn unportable_call_name_is_reported() {
    let long_legal = "z".repeat(64);
    let too_long = "x".repeat(65);
    // Legal under Anthropic's rule alone, which allows 128.
    let anthropic_only = "y".repeat(100);
    let names: Vec<&str> = vec![
        "look up",
        "a.b",
        "\u{e9}",
        &too_long,
        &anthropic_only,
        &long_legal,
        "a-Z_9",
    ];

    let mut types: Vec<NodeType> = names
        .iter()
        .map(|&name| node_type(name, &[], "note"))
        .collect();
    types.push(node_type("source", &[], "note"));
    // A node type with an illegal name that no instance calls is never offered.
    types.push(node_type("never called", &[], "note"));

    let workflow = definition(
        types,
        vec![instance("a", "source", &[]).with_calls(&names)],
        &["a"],
    );
    let report = validate_wiring(&workflow);

    // Each declared, so only the name is wrong; searched rather than matched
    // whole, every one of them holds a legal character and would pass.
    assert_eq!(
        report,
        vec![
            unportable("a", "look up"),
            unportable("a", "a.b"),
            unportable("a", "\u{e9}"),
            unportable("a", &too_long),
            unportable("a", &anthropic_only),
        ]
    );
    assert!(matches!(
        crate::Run::<std::convert::Infallible>::start(&workflow, crate::Arguments::new(), 10),
        Err(crate::StartRefusal::Wiring(defects)) if defects == report
    ));
}

#[cfg(test)]
proptest! {
    /// For any name a call gives, the validator reports it exactly when it does
    /// not match `^[a-zA-Z0-9_-]{1,64}$`, the verdict coming from how the name
    /// was built, with lengths near the edges.
    #[test]
    fn call_names_match_the_rule(
        length in prop_oneof![Just(0usize), Just(1), Just(63), Just(64), Just(65), 0..130usize],
        legal in "[a-zA-Z0-9_-]{130}",
        stranger in proptest::option::of((prop_oneof![
            Just(' '), Just('.'), Just('/'), Just('\u{e9}'), Just('\u{1F600}'), Just('\n')
        ], any::<usize>())),
    ) {
        let mut name: String = legal.chars().take(length).collect();
        if let Some((stranger, at)) = stranger {
            let at = at % (name.len() + 1);
            name.insert(at, stranger);
        }
        let accepted = stranger.is_none() && (1..=64).contains(&length);

        let workflow = definition(
            vec![node_type(&name, &[], "note"), node_type("source", &[], "note")],
            vec![instance("a", "source", &[]).with_calls(&[&name])],
            &["a"],
        );
        let report = validate_wiring(&workflow);
        let reported = report
            .iter()
            .any(|defect| matches!(defect, WiringDefect::UnportableCallName { .. }));
        prop_assert_eq!(reported, !accepted, "{:?}: {:?}", name, report);
    }
}
