//! The behaviour set: what performs each node type a run is started with - a
//! script, or a person - and whether the scripts can run at all, answered
//! before the run starts without running any of them.

use std::fmt;

use agconflo_core::WorkflowDefinition;

use crate::host;

/// What performs each node type a run is started with: a script, with the name
/// of the document it came from, or a person. Every script supplied is kept,
/// a second one for the same node type included.
// @Every script kept for the check,IMPL_BEHAVIOURS_KEEP_ALL,impl,[CREQ_BEHAVIOURS_REFUSE_TWICE],[DEC_PERSON_NAMED_BY_CALLER]
#[derive(Clone, Debug, Default)]
pub struct Behaviours {
    scripts: Vec<Script>,
    /// The node types a person performs, each as often as it was named.
    people: Vec<String>,
}

/// One script: the node type it is the behaviour of, the document it came from,
/// and its text.
#[derive(Clone, Debug)]
pub(crate) struct Script {
    pub(crate) node_type: String,
    pub(crate) document: String,
    pub(crate) source: String,
}

impl Behaviours {
    /// No scripts at all.
    pub fn new() -> Self {
        Self::default()
    }

    /// The same scripts, also giving `node_type` the script `source`, which came
    /// from `document`. A fault in the script, and the compiler's own messages,
    /// name `document`.
    pub fn define(mut self, node_type: &str, document: &str, source: &str) -> Self {
        self.scripts.push(Script {
            node_type: node_type.to_owned(),
            document: document.to_owned(),
            source: source.to_owned(),
        });
        self
    }

    /// The same behaviours, with `node_type` performed by a person rather than
    /// by a script: a scripted run hands each of its activations to its caller
    /// and runs nothing for it.
    ///
    /// Naming a type twice is not a fault. Naming it and giving it a script is.
    // @A person named by the caller,TRACE_BEHAVIOURS_PERSON,trace,[],[DEC_PERSON_NAMED_BY_CALLER, DEC_SCRIPTED_RUN_RETURNS_TO_AWAIT]
    pub fn person(mut self, node_type: &str) -> Self {
        self.people.push(node_type.to_owned());
        self
    }

    /// Whether a person performs `node_type`.
    pub(crate) fn performed_by_person(&self, node_type: &str) -> bool {
        self.people.iter().any(|named| named == node_type)
    }

    /// The one script for `node_type`, or `None` when it has none or several.
    pub(crate) fn script_for(&self, node_type: &str) -> Option<&Script> {
        let mut scripts = self.scripts.iter().filter(|s| s.node_type == node_type);
        let first = scripts.next()?;
        scripts.next().is_none().then_some(first)
    }

    /// Every fault in the scripts for the node types `definition`'s instances
    /// name, as their own or as ones they may call: at most one per type, in the
    /// order the definition first names each type.
    ///
    /// A type no instance names is not asked about, and a script for a type the
    /// definition lacks is ignored. A type a person performs must have no script.
    // @Every instantiated type has one script that compiles or a person,IMPL_BEHAVIOURS_CHECK,impl,[CREQ_BEHAVIOURS_REFUSE_MISSING, CREQ_BEHAVIOURS_REFUSE_TWICE, CREQ_BEHAVIOURS_EVERY_FAULT, CREQ_BEHAVIOURS_PERSON_OR_SCRIPT]
    pub(crate) fn faults(&self, definition: &WorkflowDefinition) -> Vec<BehaviourFault> {
        let mut asked: Vec<&str> = Vec::new();
        let mut faults = Vec::new();

        let named = definition
            .instances
            .iter()
            .flat_map(|instance| std::iter::once(&instance.node_type).chain(&instance.calls));
        for node_type in named {
            let node_type = node_type.as_str();
            if asked.contains(&node_type) {
                continue;
            }
            asked.push(node_type);

            let scripts: Vec<&Script> = self
                .scripts
                .iter()
                .filter(|s| s.node_type == node_type)
                .collect();
            let person = self.performed_by_person(node_type);
            let fault = match scripts.as_slice() {
                [] if person => None,
                [] => Some(BehaviourFault::Missing {
                    node_type: node_type.to_owned(),
                }),
                given if person => Some(BehaviourFault::PersonAndScript {
                    node_type: node_type.to_owned(),
                    documents: given.iter().map(|s| s.document.clone()).collect(),
                }),
                [script] => compile(script)
                    .err()
                    .map(|message| BehaviourFault::DoesNotCompile {
                        node_type: node_type.to_owned(),
                        document: script.document.clone(),
                        message,
                    }),
                several => Some(BehaviourFault::DefinedTwice {
                    node_type: node_type.to_owned(),
                    documents: several.iter().map(|s| s.document.clone()).collect(),
                }),
            };
            faults.extend(fault);
        }

        faults
    }
}

/// Whether `script` compiles, and the compiler's own account when it does not,
/// naming the document and the line as a failure at run time would. Nothing in
/// the script runs, and a call to a function that exists nowhere compiles.
// @Compiled and never called,IMPL_BEHAVIOURS_COMPILE,impl,[CREQ_BEHAVIOURS_REFUSE_UNCOMPILABLE, CREQ_BEHAVIOURS_NOTHING_RUN],[DEC_SCRIPT_IS_THE_BODY]
fn compile(script: &Script) -> Result<(), String> {
    let lua = host::sandbox().map_err(|error| error.to_string())?;
    lua.load(&script.source)
        .set_name(host::chunk_name(&script.document))
        .into_function()
        .map(drop)
        .map_err(|error| error.to_string())
}

/// One reason the scripts a run was given cannot run. Every variant names the
/// node type.
// @Script faults as values,IMPL_BEHAVIOURS_FAULT,impl,[CREQ_BEHAVIOURS_EVERY_FAULT],[DEC_FAILURES_NON_EXHAUSTIVE]
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BehaviourFault {
    /// An instance names this node type, no script was supplied for it, and it
    /// was not named as performed by a person.
    Missing {
        /// The node type with no script.
        node_type: String,
    },
    /// The script supplied for this node type does not compile.
    DoesNotCompile {
        /// The node type whose script does not compile.
        node_type: String,
        /// The document the script came from.
        document: String,
        /// The compiler's own account, which names the document and the line.
        message: String,
    },
    /// More than one script was supplied for this node type.
    DefinedTwice {
        /// The node type given several scripts.
        node_type: String,
        /// The documents they came from, in the order they were supplied.
        documents: Vec<String>,
    },
    /// This node type was named as performed by a person and given a script
    /// as well.
    PersonAndScript {
        /// The node type given both.
        node_type: String,
        /// The documents its scripts came from, in the order they were
        /// supplied.
        documents: Vec<String>,
    },
}

impl fmt::Display for BehaviourFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { node_type } => write!(f, "{node_type} has no script"),
            Self::DoesNotCompile {
                node_type,
                document,
                message,
            } => write!(
                f,
                "the script for {node_type} in {document} does not compile: {message}"
            ),
            Self::DefinedTwice {
                node_type,
                documents,
            } => write!(
                f,
                "{node_type} was given {} scripts: {}",
                documents.len(),
                documents.join(", ")
            ),
            Self::PersonAndScript {
                node_type,
                documents,
            } => write!(
                f,
                "{node_type} is performed by a person and was given a script too: {}",
                documents.join(", ")
            ),
        }
    }
}

impl std::error::Error for BehaviourFault {}

#[cfg(test)]
use crate::ScriptFailure;
#[cfg(test)]
use crate::scripted::{ScriptedRefusal, failed, rendered, run_with, workflow};

/// The script faults a refusal carries, or a panic naming what came instead.
#[cfg(test)]
fn faults_of(ending: Result<crate::Outcome, ScriptedRefusal>) -> Vec<BehaviourFault> {
    match ending {
        Err(ScriptedRefusal::Behaviours(faults)) => faults,
        other => panic!("expected the scripts to be refused, got {other:?}"),
    }
}

/// Three instances: two of `twin` and one of `lone`, each an entry node.
#[cfg(test)]
const TWINS_TYPES: &str = "\
[types.twin]
output = \"note\"

[types.lone]
output = \"note\"

[types.unused]
output = \"note\"
";

#[cfg(test)]
const TWINS: &str = "\
name = \"twins\"
output = \"c\"

[instances.a]
node_type = \"twin\"

[instances.b]
node_type = \"twin\"

[instances.c]
node_type = \"lone\"
";

/// A script making its output from nothing.
#[cfg(test)]
const MAKES: &str = "local given, host = ...\nreturn host.text(host.output, 'made')";

#[cfg(test)]
#[test]
fn missing_script_is_refused() {
    let definition = workflow(TWINS_TYPES, TWINS);
    // `lone` has a script; `twin` has none, only one keyed by a misspelling.
    let behaviours = Behaviours::new()
        .define("lone", "lone.lua", MAKES)
        .define("twinn", "twin.lua", MAKES);

    // Refused before starting, with one fault naming the type once although two
    // instances name it.
    assert_eq!(
        faults_of(run_with(&definition, &behaviours, None)),
        vec![BehaviourFault::Missing {
            node_type: "twin".to_owned()
        }]
    );
}

#[cfg(test)]
#[test]
fn uninstantiated_type_needs_no_script() {
    // `unused` is declared and never instantiated, and has no script; `ghost` is
    // a type the catalogue does not have at all, and has one.
    let definition = workflow(TWINS_TYPES, TWINS);
    let behaviours = Behaviours::new()
        .define("twin", "twin.lua", MAKES)
        .define("lone", "lone.lua", MAKES)
        .define("ghost", "ghost.lua", MAKES);

    assert_eq!(rendered(run_with(&definition, &behaviours, None)), "made");
}

#[cfg(test)]
#[test]
fn uncompilable_script_is_refused() {
    let definition = workflow(TWINS_TYPES, TWINS);
    let broken = "local given, host = ...\nreturn host.text(host.output, 'made'";
    let behaviours = Behaviours::new()
        .define("twin", "scripts/twin.lua", MAKES)
        .define("lone", "scripts/lone.lua", broken);

    let faults = faults_of(run_with(&definition, &behaviours, None));
    let [
        BehaviourFault::DoesNotCompile {
            node_type,
            document,
            message,
        },
    ] = faults.as_slice()
    else {
        panic!("expected one compile fault, got {faults:?}")
    };
    assert_eq!(node_type, "lone");
    assert_eq!(document, "scripts/lone.lua");
    // The compiler's own account, naming the document and the line.
    assert!(
        message.contains("scripts/lone.lua:2:"),
        "the message names the document and line 2: {message}"
    );
}

#[cfg(test)]
#[test]
fn uninstantiated_broken_script_passes() {
    let definition = workflow(TWINS_TYPES, TWINS);
    let behaviours = Behaviours::new()
        .define("twin", "twin.lua", MAKES)
        .define("lone", "lone.lua", MAKES)
        .define("unused", "unused.lua", "this is not Lua");

    assert_eq!(rendered(run_with(&definition, &behaviours, None)), "made");
}

#[cfg(test)]
#[test]
fn undefined_name_fails_when_run() {
    let definition = workflow(TWINS_TYPES, TWINS);
    let behaviours = Behaviours::new().define("twin", "twin.lua", MAKES).define(
        "lone",
        "lone.lua",
        "return nowhere_at_all()",
    );

    // It compiles, so the run starts; it fails when run, as the script's error.
    let (instance, failure) = failed(run_with(&definition, &behaviours, None));
    assert_eq!(instance, "c");
    match failure {
        ScriptFailure::Raised { message } => {
            assert!(message.contains("nowhere_at_all"), "{message}");
        }
        other => panic!("expected a script error, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn two_scripts_are_refused() {
    let definition = workflow(TWINS_TYPES, TWINS);
    let expected = vec![BehaviourFault::DefinedTwice {
        node_type: "twin".to_owned(),
        documents: vec!["one.lua".to_owned(), "two.lua".to_owned()],
    }];

    // Different texts.
    let behaviours = Behaviours::new()
        .define("twin", "one.lua", MAKES)
        .define("lone", "lone.lua", MAKES)
        .define("twin", "two.lua", "return 'other'");
    assert_eq!(
        faults_of(run_with(&definition, &behaviours, None)),
        expected
    );

    // Identical texts: still two documents claiming one type.
    let behaviours = Behaviours::new()
        .define("twin", "one.lua", MAKES)
        .define("lone", "lone.lua", MAKES)
        .define("twin", "two.lua", MAKES);
    assert_eq!(
        faults_of(run_with(&definition, &behaviours, None)),
        expected
    );
}

#[cfg(test)]
#[test]
fn every_fault_reported() {
    let types = "\
[types.nothing]
output = \"note\"

[types.broken]
output = \"note\"

[types.doubled]
output = \"note\"
";
    let flow = "\
name = \"faulty\"
output = \"c\"

[instances.a]
node_type = \"nothing\"

[instances.b]
node_type = \"broken\"

[instances.c]
node_type = \"doubled\"
";
    let definition = workflow(types, flow);
    let behaviours = Behaviours::new()
        .define("broken", "broken.lua", "return (")
        .define("doubled", "one.lua", MAKES)
        .define("doubled", "two.lua", MAKES);

    let faults = faults_of(run_with(&definition, &behaviours, None));
    assert_eq!(faults.len(), 3, "{faults:?}");
    assert_eq!(
        faults[0],
        BehaviourFault::Missing {
            node_type: "nothing".to_owned()
        }
    );
    assert!(
        matches!(&faults[1], BehaviourFault::DoesNotCompile { node_type, .. } if node_type == "broken"),
        "{faults:?}"
    );
    assert_eq!(
        faults[2],
        BehaviourFault::DefinedTwice {
            node_type: "doubled".to_owned(),
            documents: vec!["one.lua".to_owned(), "two.lua".to_owned()],
        }
    );
}

#[cfg(test)]
#[test]
fn checking_runs_nothing() {
    let definition = workflow(TWINS_TYPES, TWINS);
    // `lone`'s script fails on its first line if it runs at all; the only fault
    // is `twin`'s missing script.
    let behaviours = Behaviours::new().define("lone", "lone.lua", "error('ran during the check')");

    assert_eq!(
        faults_of(run_with(&definition, &behaviours, None)),
        vec![BehaviourFault::Missing {
            node_type: "twin".to_owned()
        }]
    );

    // The control: with `twin` given a script, the run starts and `lone`'s script
    // runs and fails.
    let behaviours = behaviours.define("twin", "twin.lua", MAKES);
    let (_, failure) = failed(run_with(&definition, &behaviours, None));
    assert!(
        matches!(&failure, ScriptFailure::Raised { message } if message.contains("ran during the check")),
        "{failure:?}"
    );
}

#[cfg(test)]
#[test]
fn person_needs_no_script() {
    let definition = workflow(TWINS_TYPES, TWINS);
    let scripted = Behaviours::new().define("twin", "twin.lua", MAKES);

    // `lone` has no script and a person performs it: the run starts, and stops
    // at its instance `c` to hand it over.
    let outcome = run_with(&definition, &scripted.clone().person("lone"), None);
    match outcome {
        Ok(crate::Outcome::Awaiting(activation)) => assert_eq!(activation.instance(), "c"),
        other => panic!("expected the person's step handed over, got {other:?}"),
    }

    // The control: the same scripts without the naming are refused for it.
    assert_eq!(
        faults_of(run_with(&definition, &scripted, None)),
        vec![BehaviourFault::Missing {
            node_type: "lone".to_owned()
        }]
    );
}

#[cfg(test)]
#[test]
fn person_and_script_refused() {
    let definition = workflow(TWINS_TYPES, TWINS);
    let ran = "error('ran')";

    // `twin` is named for a person and given two scripts, each of which fails
    // if it runs; `lone` has neither. Both faults, once each, in one refusal.
    let behaviours = Behaviours::new()
        .define("twin", "one.lua", ran)
        .person("twin")
        .define("twin", "two.lua", ran);
    assert_eq!(
        faults_of(run_with(&definition, &behaviours, None)),
        vec![
            BehaviourFault::PersonAndScript {
                node_type: "twin".to_owned(),
                documents: vec!["one.lua".to_owned(), "two.lua".to_owned()],
            },
            BehaviourFault::Missing {
                node_type: "lone".to_owned()
            },
        ]
    );

    // Controls that start: `lone` named for a person twice, and `unused` -
    // which no instance names - both named for a person and given a script.
    let behaviours = Behaviours::new()
        .define("twin", "twin.lua", MAKES)
        .person("lone")
        .person("lone")
        .person("unused")
        .define("unused", "unused.lua", ran);
    let outcome = run_with(&definition, &behaviours, None);
    assert!(
        matches!(&outcome, Ok(crate::Outcome::Awaiting(activation)) if activation.instance() == "c"),
        "{outcome:?}"
    );
}

#[cfg(test)]
#[test]
fn callee_without_behaviour_refused() {
    use crate::scripted::{YIELD_TYPES, YIELDING};
    // `ask` has a script; `lookup`, which `asker` declares a call to, has
    // neither a script nor a person - beside another type missing its own.
    let definition = workflow(
        YIELD_TYPES,
        &format!("{YIELDING}\n[instances.other]\nnode_type = \"search\"\n"),
    );
    let behaviours = Behaviours::new().define("ask", "ask.lua", MAKES);

    // Refused as a value before anything runs, naming the called type as it
    // names the instantiated one, in the order the definition first names each.
    assert_eq!(
        faults_of(run_with(&definition, &behaviours, None)),
        vec![
            BehaviourFault::Missing {
                node_type: "lookup".to_owned()
            },
            BehaviourFault::Missing {
                node_type: "search".to_owned()
            },
        ]
    );

    // Given a person, the called type needs no script.
    let behaviours = behaviours
        .person("lookup")
        .define("search", "search.lua", MAKES);
    assert!(run_with(&definition, &behaviours, None).is_ok());
}
