//! The runner: a run started, resumed, answered or checked from a project and
//! a model mapping, with its records kept in a file.

use std::fmt;
use std::path::Path;

use agconflo_core::{
    Arguments, Context, IdSource, WiringDefect, WorkflowDefinition, validate_wiring,
};
use agconflo_lua::{
    BehaviourFault, Outcome, ScriptedRefusal, answer_scripted, resume_scripted, run_scripted,
};

use crate::keeper::{KeeperRefusal, RecordKeeper, Unkept};
use crate::model_map::{ModelMap, ModelsFault, read_models};
use crate::project::{Project, ProjectFault, read_project};

/// The files a run is read from: its manifest, and the model mapping when it
/// has one.
#[derive(Clone, Copy, Debug)]
pub struct Sources<'a> {
    /// The manifest.
    pub manifest: &'a Path,
    /// The model mapping, or `None` for a run whose scripts call no model.
    pub models: Option<&'a Path>,
}

/// One argument of a run: the text for one entry instance's parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Argument {
    /// The entry instance.
    pub instance: String,
    /// The parameter it fills.
    pub parameter: String,
    /// The text, kept exactly.
    pub text: String,
}

/// Where a run stopped, and which record its file holds when that is not the
/// latest the run handed over.
#[derive(Clone, Debug)]
pub struct Stopped {
    /// The run's ending, or the step it awaits a person for.
    pub outcome: Outcome,
    /// The record file behind the run, or `None` when it holds the latest.
    pub unkept: Option<Unkept>,
}

/// Why a run was not started, resumed or answered. Nothing ran for any of them.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Refusal {
    /// The manifest or a file it names cannot be read.
    Project(ProjectFault),
    /// The model mapping cannot be read.
    Models(ModelsFault),
    /// Text was given for a parameter no entry instance of the workflow
    /// declares.
    Argument {
        /// The instance, as given.
        instance: String,
        /// The parameter, as given.
        parameter: String,
    },
    /// The record file was not held for the run.
    Record(KeeperRefusal),
    /// The scripted run refused, as it refused.
    Run(ScriptedRefusal),
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Project(fault) => fault.fmt(f),
            Self::Models(fault) => fault.fmt(f),
            Self::Argument {
                instance,
                parameter,
            } => write!(
                f,
                "no entry instance {instance} of the workflow declares a parameter {parameter}"
            ),
            Self::Record(refusal) => refusal.fmt(f),
            Self::Run(refusal) => refusal.fmt(f),
        }
    }
}

impl std::error::Error for Refusal {}

/// One reason a run of a project would be refused, found without running it.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Finding {
    /// The manifest or a file it names cannot be read.
    Project(ProjectFault),
    /// The model mapping cannot be read.
    Models(ModelsFault),
    /// The workflow's wiring is defective.
    Wiring(WiringDefect),
    /// A script cannot run.
    Script(BehaviourFault),
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Project(fault) => fault.fmt(f),
            Self::Models(fault) => fault.fmt(f),
            Self::Wiring(defect) => defect.fmt(f),
            Self::Script(fault) => fault.fmt(f),
        }
    }
}

/// A new run of the project `sources` names, started with `arguments`, its
/// records kept in `record` - or why it was not started.
///
/// Nothing is written and the record file is not taken until the project, the
/// model mapping and the arguments are read.
// @A run started from what was read for it,IMPL_RUNNER_START,impl,[CREQ_RUNNER_STARTS, CREQ_RUNNER_REFUSED_START_TAKES_NOTHING, CREQ_RUNNER_HANDS_BACK_HOW],[DEC_RUNNER_ON_ONE_THREAD]
pub async fn start(
    sources: Sources<'_>,
    record: &Path,
    arguments: &[Argument],
) -> Result<Stopped, Refusal> {
    let (project, models) = read(sources)?;
    let mut source = IdSource::new();
    let supplied = supplied(project.definition(), arguments, &mut source)?;
    let mut keeper = RecordKeeper::new_run(record).map_err(Refusal::Record)?;
    let outcome = run_scripted(
        project.definition(),
        &project.behaviours(),
        models.roster(),
        supplied,
        &mut source,
        project.budget(),
        project.limits(),
        |record| keeper.keep(&record),
    )
    .await;
    stopped(keeper, outcome)
}

/// The run whose record `record` holds, resumed against the project `sources`
/// names, its later records kept in the same file - or why it was not resumed.
// @A run resumed from the record its file holds,IMPL_RUNNER_RESUME,impl,[CREQ_RUNNER_RESUMES, CREQ_RUNNER_HANDS_BACK_HOW],[DEC_RUNNER_ON_ONE_THREAD]
pub async fn resume(sources: Sources<'_>, record: &Path) -> Result<Stopped, Refusal> {
    let (project, models) = read(sources)?;
    let (mut keeper, text) = RecordKeeper::resume(record).map_err(Refusal::Record)?;
    let outcome = resume_scripted(
        project.definition(),
        &project.behaviours(),
        models.roster(),
        &text,
        project.limits(),
        |record| keeper.keep(&record),
    )
    .await
    .map(|(outcome, _)| outcome);
    stopped(keeper, outcome)
}

/// The run whose record `record` holds, continued with `text` as the output of
/// the step it awaits for `instance`, its later records kept in the same file
/// - or why it was not. A refused answer leaves the file as it was.
// @A person's text answering the record its file holds,IMPL_RUNNER_ANSWER,impl,[CREQ_RUNNER_ANSWERS, CREQ_RUNNER_HANDS_BACK_HOW],[DEC_ANSWER_ONCE_BY_THE_KEEPER, DEC_RUNNER_ON_ONE_THREAD]
pub async fn answer(
    sources: Sources<'_>,
    record: &Path,
    instance: &str,
    text: &str,
) -> Result<Stopped, Refusal> {
    let (project, models) = read(sources)?;
    let (mut keeper, held) = RecordKeeper::resume(record).map_err(Refusal::Record)?;
    let outcome = answer_scripted(
        project.definition(),
        &project.behaviours(),
        models.roster(),
        &held,
        instance,
        text,
        project.limits(),
        |record| keeper.keep(&record),
    )
    .await
    .map(|(outcome, _)| outcome);
    stopped(keeper, outcome)
}

/// Every reason a run of the project `sources` names would be refused that
/// does not depend on its arguments or a record: the project's fault or the
/// model mapping's, then every wiring defect and every script's fault. Empty
/// when there is none. Nothing runs and no file is written.
// @What would refuse a run found without running it,IMPL_RUNNER_CHECK,impl,[CREQ_RUNNER_CHECKS]
pub fn check(sources: Sources<'_>) -> Vec<Finding> {
    let mut findings = Vec::new();
    let project = read_project(sources.manifest);
    if let Err(fault) = &project {
        findings.push(Finding::Project(fault.clone()));
    }
    if let Some(models) = sources.models
        && let Err(fault) = read_models(models, environment)
    {
        findings.push(Finding::Models(fault));
    }
    if let Ok(project) = project {
        let definition = project.definition();
        findings.extend(validate_wiring(definition).into_iter().map(Finding::Wiring));
        findings.extend(
            project
                .behaviours()
                .faults(definition)
                .into_iter()
                .map(Finding::Script),
        );
    }
    findings
}

/// The project and the model mapping `sources` names.
fn read(sources: Sources<'_>) -> Result<(Project, ModelMap), Refusal> {
    let project = read_project(sources.manifest).map_err(Refusal::Project)?;
    let models = match sources.models {
        Some(models) => read_models(models, environment),
        None => ModelMap::none(),
    }
    .map_err(Refusal::Models)?;
    Ok((project, models))
}

/// A variable of this process's environment, when it is set to text.
fn environment(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Each argument as a context of the type its parameter declares, issued by
/// `source`, or a refusal for the first naming no entry instance's parameter.
// @Each argument made a context of the type its parameter declares,IMPL_RUNNER_ARGUMENTS,impl,[CREQ_RUNNER_ARGUMENTS_AS_TEXT, CREQ_RUNNER_REFUSES_UNKNOWN_ARGUMENT],[DEC_ARGUMENTS_AS_TEXT]
fn supplied(
    definition: &WorkflowDefinition,
    arguments: &[Argument],
    source: &mut IdSource,
) -> Result<Arguments, Refusal> {
    arguments
        .iter()
        .try_fold(Arguments::new(), |supplied, argument| {
            let declared = definition
                .instances
                .iter()
                .find(|instance| instance.entry && instance.name == argument.instance)
                .and_then(|instance| {
                    definition
                        .node_types
                        .iter()
                        .find(|node_type| node_type.name == instance.node_type)
                })
                .and_then(|node_type| {
                    node_type
                        .required
                        .iter()
                        .chain(&node_type.optional)
                        .find(|parameter| parameter.name == argument.parameter)
                })
                .ok_or_else(|| Refusal::Argument {
                    instance: argument.instance.clone(),
                    parameter: argument.parameter.clone(),
                })?;
            let context = Context::text(
                source,
                declared.context_type.clone(),
                argument.text.as_str(),
            )
            .expect("a new source has identifiers to issue");
            Ok(supplied.supply(&argument.instance, &argument.parameter, context))
        })
}

/// How a run stopped, with the record keeper let go and its report beside it.
fn stopped(
    keeper: RecordKeeper,
    outcome: Result<Outcome, ScriptedRefusal>,
) -> Result<Stopped, Refusal> {
    let unkept = keeper.release();
    let outcome = outcome.map_err(Refusal::Run)?;
    Ok(Stopped { outcome, unkept })
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases.

#[cfg(test)]
use crate::testing::{Scratch, Stub};
#[cfg(test)]
use agconflo_core::RunEnding;
#[cfg(test)]
use agconflo_lua::ScriptFailure;
#[cfg(test)]
use proptest::prelude::*;

/// Every node type the tests' workflows use: an entry taking a brief and an
/// optional extra, and four types taking what came before - one asking a
/// model, one adding to it, one a person performs, and one whose script fails.
#[cfg(test)]
const TYPES: &str = "[types.begin]\nrequired = { brief = \"note\" }\noptional = { extra = \"other\" }\noutput = \"note\"\n\n[types.ask]\nrequired = { before = \"note\" }\noutput = \"note\"\n\n[types.add]\nrequired = { before = \"note\" }\noutput = \"note\"\n\n[types.review]\nrequired = { before = \"note\" }\noutput = \"note\"\n\n[types.broken]\nrequired = { before = \"note\" }\noutput = \"note\"\n";

/// The scripts, by file.
#[cfg(test)]
const SCRIPTS: [(&str, &str); 4] = [
    (
        "begin.lua",
        "local given, host = ...\nreturn host.text(host.output, given.brief:render() .. '+a')\n",
    ),
    (
        "ask.lua",
        "local given, host = ...\nlocal answer = host.complete('helping', given.before)\nreturn host.text(host.output, answer:render() .. '+b')\n",
    ),
    (
        "add.lua",
        "local given, host = ...\nreturn host.text(host.output, given.before:render() .. '+c')\n",
    ),
    ("broken.lua", "error('broke here')\n"),
];

/// A project in `scratch`: `first` begins, `second` is of `middle`, `third`
/// adds; a person performs `review`; the budget is `budget`. Returns the
/// manifest's path.
#[cfg(test)]
fn project(scratch: &Scratch, middle: &str, budget: usize) -> std::path::PathBuf {
    scratch.write("types.toml", TYPES);
    for (file, text) in SCRIPTS {
        scratch.write(file, text);
    }
    scratch.write(
        "flow.toml",
        format!(
            "name = \"chain\"\noutput = \"third\"\n\n[instances.first]\nnode_type = \"begin\"\nentry = true\n\n[instances.second]\nnode_type = \"{middle}\"\nbindings = {{ before = \"first\" }}\n\n[instances.third]\nnode_type = \"add\"\nbindings = {{ before = \"second\" }}\n"
        ),
    );
    scratch.write(
        "manifest.toml",
        format!(
            "workflow = \"flow.toml\"\ntypes = [\"types.toml\"]\nbudget = {budget}\npersons = [\"review\"]\n\n[scripts]\nbegin = \"begin.lua\"\nask = \"ask.lua\"\nadd = \"add.lua\"\nbroken = \"broken.lua\"\n"
        ),
    )
}

/// A model mapping in `scratch` sending `helping` to `stub`, written as `file`.
#[cfg(test)]
fn models(scratch: &Scratch, file: &str, stub: &Stub) -> std::path::PathBuf {
    scratch.write(
        file,
        format!(
            "[roles]\nhelping = {{ model = \"openai::helper\", endpoint = \"{}\" }}\n",
            stub.base
        ),
    )
}

/// The brief `text` for the entry instance.
#[cfg(test)]
fn brief(text: &str) -> Vec<Argument> {
    vec![Argument {
        instance: "first".to_owned(),
        parameter: "brief".to_owned(),
        text: text.to_owned(),
    }]
}

/// A runtime for one test's run, on the test's own thread.
#[cfg(test)]
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a runtime")
}

/// `run`, dropped once `stub` has been sent a request: a run interrupted while
/// a model is asked.
#[cfg(test)]
async fn interrupted(
    run: impl std::future::Future<Output = Result<Stopped, Refusal>>,
    stub: &Stub,
) {
    tokio::select! {
        stopped = run => panic!("expected the run to wait on the model, got {stopped:?}"),
        () = async {
            while stub.requests().is_empty() {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        } => {}
    }
}

/// The rendering a run completed with, or a panic naming how it stopped.
#[cfg(test)]
fn completed(stopped: Result<Stopped, Refusal>) -> String {
    match stopped {
        Ok(Stopped {
            outcome: Outcome::Ended(RunEnding::Completed(result)),
            unkept: None,
        }) => result.render().into_owned(),
        other => panic!("expected the run to complete with its record kept, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn runs_to_completion() {
    let scratch = Scratch::new("runs_to_completion");
    let manifest = project(&scratch, "ask", 5);
    let holding = Stub::holding();
    let sources = Sources {
        manifest: &manifest,
        models: Some(&models(&scratch, "holding.toml", &holding)),
    };
    let record = scratch.path("interrupted.toml");

    // While the model is asked, the file already holds the record after the
    // first output.
    runtime().block_on(interrupted(start(sources, &record, &brief("x")), &holding));
    let kept = std::fs::read_to_string(&record).expect("a record kept before the model answered");
    assert!(kept.contains("x+a"), "{kept}");

    let answering = Stub::answering("M");
    let answered = models(&scratch, "answering.toml", &answering);
    let sources = Sources {
        manifest: &manifest,
        models: Some(&answered),
    };
    let record = scratch.path("run.toml");
    assert_eq!(
        completed(runtime().block_on(start(sources, &record, &brief("x")))),
        "M+b+c"
    );

    // The file holds the last record: resumed, it hands the same ending back.
    assert_eq!(
        completed(runtime().block_on(resume(sources, &record))),
        "M+b+c"
    );
    assert_eq!(answering.requests().len(), 1);

    match runtime().block_on(start(sources, &record, &brief("x"))) {
        Err(Refusal::Record(KeeperRefusal::Exists { .. })) => {}
        other => panic!("expected the existing record refused, got {other:?}"),
    }
    assert_eq!(answering.requests().len(), 1);
}

#[cfg(test)]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]
    // @Few cases for runs that each build a model client,TRACE_RUNNER_PROPERTY_CASES,trace,[],[NOTE_RUNNER_PROPERTY_CASES]
    #[test]
    fn arguments_kept_exactly(wanted in "( |\r\n|\n|[a-z])*", extra in "( |\r\n|\n|[A-Z])*") {
        let scratch = Scratch::new("arguments_kept_exactly");
        let manifest = project(&scratch, "add", 5);
        scratch.write(
            "begin.lua",
            "local given, host = ...\nreturn host.compose(host.output, {given.brief, given.extra}, '|')\n",
        );
        let arguments = [
            Argument { instance: "first".to_owned(), parameter: "brief".to_owned(), text: wanted.clone() },
            Argument { instance: "first".to_owned(), parameter: "extra".to_owned(), text: extra.clone() },
        ];
        let sources = Sources { manifest: &manifest, models: None };

        let stopped = runtime().block_on(start(sources, &scratch.path("run.toml"), &arguments));

        prop_assert_eq!(completed(stopped), format!("{wanted}|{extra}+c+c"));
    }
}

#[cfg(test)]
#[test]
fn unknown_argument_refused() {
    let scratch = Scratch::new("unknown_argument_refused");
    let manifest = project(&scratch, "add", 5);
    let sources = Sources {
        manifest: &manifest,
        models: None,
    };
    let record = scratch.path("run.toml");

    for (instance, parameter) in [("ghost", "brief"), ("first", "brif"), ("second", "before")] {
        let arguments = [Argument {
            instance: instance.to_owned(),
            parameter: parameter.to_owned(),
            text: "x".to_owned(),
        }];
        match runtime().block_on(start(sources, &record, &arguments)) {
            Err(Refusal::Argument {
                instance: named,
                parameter: which,
            }) => assert_eq!((named.as_str(), which.as_str()), (instance, parameter)),
            other => panic!("expected {instance}.{parameter} refused, got {other:?}"),
        }
        assert!(!record.exists());
        assert!(!scratch.path("run.toml.lock").exists());
    }
}

#[cfg(test)]
#[test]
fn resumes_an_interrupted_run() {
    let scratch = Scratch::new("resumes_an_interrupted_run");
    let manifest = project(&scratch, "ask", 5);
    let holding = Stub::holding();
    let held = models(&scratch, "holding.toml", &holding);
    let record = scratch.path("run.toml");
    let sources = Sources {
        manifest: &manifest,
        models: Some(&held),
    };
    runtime().block_on(interrupted(start(sources, &record, &brief("x")), &holding));

    // Resumed against a provider that does not answer, the file is held for as
    // long as the resume waits.
    let waiting = Stub::holding();
    let sources = Sources {
        manifest: &manifest,
        models: Some(&models(&scratch, "waiting.toml", &waiting)),
    };
    runtime().block_on(async {
        tokio::select! {
            stopped = resume(sources, &record) => panic!("expected the resume to wait, got {stopped:?}"),
            () = async {
                while waiting.requests().is_empty() {
                    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                }
                assert!(matches!(
                    RecordKeeper::resume(&record),
                    Err(KeeperRefusal::Held { .. })
                ));
            } => {}
        }
    });

    let answering = Stub::answering("M");
    let sources = Sources {
        manifest: &manifest,
        models: Some(&models(&scratch, "answering.toml", &answering)),
    };
    assert_eq!(
        completed(runtime().block_on(resume(sources, &record))),
        "M+b+c"
    );
    let (_, last) = RecordKeeper::resume(&record).expect("the file is free");
    assert!(last.contains("M+b+c"), "{last}");
}

#[cfg(test)]
#[test]
fn answers_the_awaited_step() {
    let scratch = Scratch::new("answers_the_awaited_step");
    let manifest = project(&scratch, "review", 5);
    let sources = Sources {
        manifest: &manifest,
        models: None,
    };
    let record = scratch.path("run.toml");

    match runtime().block_on(start(sources, &record, &brief("x"))) {
        Ok(Stopped {
            outcome: Outcome::Awaiting(activation),
            unkept: None,
        }) => assert_eq!(activation.instance(), "second"),
        other => panic!("expected the run to await a person, got {other:?}"),
    }
    let awaiting = std::fs::read_to_string(&record).expect("the record");

    let text = "ok\r\nfine\n";
    assert_eq!(
        completed(runtime().block_on(answer(sources, &record, "second", text))),
        format!("{text}+c")
    );
    assert_ne!(
        std::fs::read_to_string(&record).expect("the record"),
        awaiting
    );

    match runtime().block_on(answer(sources, &record, "second", text)) {
        Err(Refusal::Run(ScriptedRefusal::NotAwaited { .. })) => {}
        other => panic!("expected a second answer refused, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn answer_elsewhere_leaves_the_file() {
    let scratch = Scratch::new("answer_elsewhere_leaves_the_file");
    let manifest = project(&scratch, "review", 5);
    let sources = Sources {
        manifest: &manifest,
        models: None,
    };
    let record = scratch.path("run.toml");
    runtime()
        .block_on(start(sources, &record, &brief("x")))
        .expect("the run awaits a person");
    let before = std::fs::read(&record).expect("the record");

    match runtime().block_on(answer(sources, &record, "third", "text")) {
        Err(Refusal::Run(ScriptedRefusal::NotAwaited { .. })) => {}
        other => panic!("expected the answer refused, got {other:?}"),
    }

    assert_eq!(std::fs::read(&record).expect("the record"), before);
    RecordKeeper::resume(&record).expect("the file is free");
}

#[cfg(test)]
#[test]
fn check_runs_nothing() {
    let scratch = Scratch::new("check_runs_nothing");
    let manifest = project(&scratch, "add", 5);
    for (file, _) in SCRIPTS {
        scratch.write(file, "while true do end\n");
    }
    let limits = format!(
        "{}\n[limits]\ninstructions = {}\n",
        std::fs::read_to_string(&manifest).expect("the manifest"),
        i64::MAX
    );
    std::fs::write(&manifest, limits).expect("the manifest");
    let sources = Sources {
        manifest: &manifest,
        models: None,
    };
    let findings = check(sources);
    assert!(findings.is_empty(), "{findings:?}");

    scratch.write(
        "flow.toml",
        "name = \"broken\"\noutput = \"third\"\n\n[instances.first]\nnode_type = \"begin\"\nentry = true\n\n[instances.second]\nnode_type = \"add\"\nbindings = { before = \"nobody\" }\n\n[instances.third]\nnode_type = \"add\"\n",
    );
    scratch.write("add.lua", "local given, host = ...\nreturn (\n");
    let findings = check(sources);
    let wiring = findings
        .iter()
        .filter(|f| matches!(f, Finding::Wiring(_)))
        .count();
    let scripts = findings
        .iter()
        .filter(|f| matches!(f, Finding::Script(_)))
        .count();
    assert_eq!((wiring, scripts), (2, 1), "{findings:?}");

    let unset = scratch.write(
        "models.toml",
        "[roles]\nhelping = { model = \"openai::helper\", key_env = \"AGCONFLO_TEST_NEVER_SET_KEY\" }\n",
    );
    let findings = check(Sources {
        manifest: &manifest,
        models: Some(&unset),
    });
    assert!(
        findings
            .iter()
            .any(|f| matches!(f, Finding::Models(ModelsFault::UnsetVariable { .. }))),
        "{findings:?}"
    );
}

#[cfg(test)]
#[test]
fn refused_start_takes_nothing() {
    let scratch = Scratch::new("refused_start_takes_nothing");
    let manifest = project(&scratch, "add", 5);
    let record = scratch.path("run.toml");
    let lock = scratch.path("run.toml.lock");

    let absent = scratch.path("absent.toml");
    let sources = Sources {
        manifest: &absent,
        models: None,
    };
    assert!(matches!(
        runtime().block_on(start(sources, &record, &brief("x"))),
        Err(Refusal::Project(_))
    ));
    assert!(!record.exists() && !lock.exists());

    let broken = scratch.write("models.toml", "[roles\n");
    let sources = Sources {
        manifest: &manifest,
        models: Some(&broken),
    };
    assert!(matches!(
        runtime().block_on(start(sources, &record, &brief("x"))),
        Err(Refusal::Models(_))
    ));
    assert!(!record.exists() && !lock.exists());

    let sources = Sources {
        manifest: &manifest,
        models: None,
    };
    assert_eq!(
        completed(runtime().block_on(start(sources, &record, &brief("x")))),
        "x+a+c+c"
    );
}

#[cfg(test)]
#[test]
fn endings_handed_back() {
    let scratch = Scratch::new("endings_handed_back");
    let run = |middle: &str, budget: usize, file: &str| {
        let manifest = project(&scratch, middle, budget);
        let sources = Sources {
            manifest: &manifest,
            models: None,
        };
        runtime().block_on(start(sources, &scratch.path(file), &brief("x")))
    };

    assert_eq!(completed(run("add", 5, "completes.toml")), "x+a+c+c");
    assert!(matches!(
        run("review", 5, "awaits.toml"),
        Ok(Stopped {
            outcome: Outcome::Awaiting(_),
            unkept: None
        })
    ));
    match run("broken", 5, "fails.toml") {
        Ok(Stopped {
            outcome:
                Outcome::Ended(RunEnding::NodeFailed {
                    instance,
                    failure: ScriptFailure::Raised { message },
                }),
            unkept: None,
        }) => {
            assert_eq!(instance, "second");
            assert!(message.contains("broke here"), "{message}");
        }
        other => panic!("expected the node to fail, got {other:?}"),
    }
    assert!(matches!(
        run("add", 1, "budget.toml"),
        Ok(Stopped {
            outcome: Outcome::Ended(RunEnding::BudgetExceeded { budget: 1 }),
            unkept: None
        })
    ));

    scratch.write(
        "flow.toml",
        "name = \"cycle\"\noutput = \"c1\"\n\n[instances.c1]\nnode_type = \"add\"\nbindings = { before = \"c2\" }\n\n[instances.c2]\nnode_type = \"add\"\nbindings = { before = \"c1\" }\n",
    );
    let sources = Sources {
        manifest: &scratch.path("manifest.toml"),
        models: None,
    };
    match runtime().block_on(start(sources, &scratch.path("stuck.toml"), &[])) {
        Ok(Stopped {
            outcome: Outcome::Ended(RunEnding::Quiescent { waiting }),
            unkept: None,
        }) => assert_eq!(waiting, ["c1", "c2"]),
        other => panic!("expected the run stuck, got {other:?}"),
    }

    // Every write blocked by a directory where the new record goes.
    let record = scratch.path("unkept.toml");
    std::fs::create_dir_all(scratch.path("unkept.toml.new/inside")).expect("the block");
    let manifest = project(&scratch, "add", 5);
    let sources = Sources {
        manifest: &manifest,
        models: None,
    };
    match runtime().block_on(start(sources, &record, &brief("x"))) {
        Ok(Stopped {
            outcome: Outcome::Ended(RunEnding::Completed(result)),
            unkept: Some(unkept),
        }) => {
            assert_eq!(result.render(), "x+a+c+c");
            assert_eq!(unkept.holds, 0);
            assert!(unkept.handed > 0);
        }
        other => panic!("expected the run completed with its record not kept, got {other:?}"),
    }
}
