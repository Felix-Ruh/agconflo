//! The runner: a run started, resumed, answered or checked from a project, a
//! model mapping and grants, with its records kept in a file and its tool
//! steps performed in a container.

use std::fmt;
use std::path::Path;

use agconflo_core::{
    Arguments, Context, IdSource, WiringDefect, WorkflowDefinition, validate_wiring,
};
use agconflo_lua::{
    BehaviourFault, Outcome, ScriptedRefusal, answer_scripted, resume_scripted, run_scripted,
};

use crate::grants::{Action, Grants, GrantsFault, read_grants};
use crate::keeper::{KeeperRefusal, RecordKeeper, Unkept};
use crate::model_map::{ModelMap, ModelsFault, read_models};
use crate::performer::{Container, parameters, perform};
use crate::project::{Project, ProjectFault, read_project};
use crate::sandbox::{EngineFailure, Environment, NotReady, Sandbox};

/// The files a run is read from: its manifest, the model mapping when it has
/// one, and the grants when it has them.
#[derive(Clone, Copy, Debug)]
pub struct Sources<'a> {
    /// The manifest.
    pub manifest: &'a Path,
    /// The model mapping, or `None` for a run whose scripts call no model.
    pub models: Option<&'a Path>,
    /// The grants file, or `None` for a run whose manifest names no tool.
    pub grants: Option<&'a Path>,
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

/// Where a run stopped, which record its file holds when that is not the
/// latest the run handed over, and why a tool's step it awaits was not
/// performed.
#[derive(Clone, Debug)]
pub struct Stopped {
    /// The run's ending, or the step it awaits a person for.
    pub outcome: Outcome,
    /// The record file behind the run, or `None` when it holds the latest.
    pub unkept: Option<Unkept>,
    /// The container engine's failure the awaited tool step was left for, or
    /// `None`.
    pub engine: Option<EngineFailure>,
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
    /// The manifest names a tool, and no grants were given.
    NoGrants,
    /// The grants cannot be read.
    Grants(GrantsFault),
    /// The grants do not provide for the run's tools: every reason.
    Ungranted(Vec<ToolFault>),
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
            Self::NoGrants => f.write_str(NO_GRANTS),
            Self::Grants(fault) => fault.fmt(f),
            Self::Ungranted(faults) => {
                let faults: Vec<String> = faults.iter().map(ToString::to_string).collect();
                f.write_str(&faults.join("; "))
            }
        }
    }
}

impl std::error::Error for Refusal {}

/// What a person is told when a manifest naming a tool comes without grants.
const NO_GRANTS: &str = "the manifest names a tool, and no grants file says what its tools may do";

/// One reason the grants do not provide for a run's tools.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ToolFault {
    /// The grants do not allow the tool's action.
    NotGranted {
        /// The tool's node type.
        node_type: String,
        /// Its action.
        action: Action,
    },
    /// The tool's node type declares no parameter its action takes.
    MissingParameter {
        /// The tool's node type.
        node_type: String,
        /// Its action.
        action: Action,
        /// The parameter it lacks.
        parameter: &'static str,
    },
    /// The tool names an image the grants do not list.
    ImageNotGranted {
        /// The tool's node type.
        node_type: String,
        /// The image it names.
        image: String,
    },
    /// An image the run's tools run in cannot be used.
    NotReady(NotReady),
}

impl fmt::Display for ToolFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotGranted { node_type, action } => write!(
                f,
                "the tool {node_type} would {action}, which the grants do not allow"
            ),
            Self::MissingParameter {
                node_type,
                action,
                parameter,
            } => write!(
                f,
                "the tool {node_type} would {action}, and its node type declares no parameter {parameter}"
            ),
            Self::ImageNotGranted { node_type, image } => write!(
                f,
                "the tool {node_type} names the image {image}, which the grants do not list"
            ),
            Self::NotReady(why) => why.fmt(f),
        }
    }
}

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
    /// The manifest names a tool, and no grants were given.
    NoGrants,
    /// The grants cannot be read.
    Grants(GrantsFault),
    /// The grants do not provide for a tool.
    Tool(ToolFault),
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Project(fault) => fault.fmt(f),
            Self::Models(fault) => fault.fmt(f),
            Self::Wiring(defect) => defect.fmt(f),
            Self::Script(fault) => fault.fmt(f),
            Self::NoGrants => f.write_str(NO_GRANTS),
            Self::Grants(fault) => fault.fmt(f),
            Self::Tool(fault) => fault.fmt(f),
        }
    }
}

/// A new run of the project `sources` names, started with `arguments`, its
/// records kept in `record` and its tool steps performed in a sandbox - or why
/// it was not started.
///
/// Nothing is written and the record file is not taken until the project, the
/// model mapping, the grants, the run's tools and the arguments are read.
pub async fn start(
    sources: Sources<'_>,
    record: &Path,
    arguments: &[Argument],
) -> Result<Stopped, Refusal> {
    start_in(sources, record, arguments, Sandbox::new).await
}

/// [`start`], performing tool steps in the container `make` makes.
// @A run started from what was read for it,IMPL_RUNNER_START,impl,[CREQ_RUNNER_STARTS, CREQ_RUNNER_REFUSED_START_TAKES_NOTHING, CREQ_RUNNER_HANDS_BACK_HOW],[DEC_RUNNER_ON_ONE_THREAD]
async fn start_in<C: Container>(
    sources: Sources<'_>,
    record: &Path,
    arguments: &[Argument],
    make: impl FnOnce(&Grants, &Path) -> C,
) -> Result<Stopped, Refusal> {
    let (project, models) = read(sources)?;
    let mut prepared = prepared(sources, &project, record, make, Unreachable::GoesOn)?;
    let mut source = IdSource::new();
    let supplied = supplied(project.definition(), arguments, &mut source)?;
    let mut kept = Kept::new(RecordKeeper::new_run(record).map_err(Refusal::Record)?);
    let behaviours = project.behaviours();
    let outcome = run_scripted(
        project.definition(),
        &behaviours,
        models.roster(),
        supplied,
        &mut source,
        project.budget(),
        project.limits(),
        |record| kept.keep(record),
    )
    .await;
    let run = Tooled {
        project: &project,
        behaviours: &behaviours,
        models: &models,
    };
    let (outcome, engine) = run.tools(prepared.as_mut(), &mut kept, outcome).await;
    stopped(kept, outcome, engine)
}

/// The run whose record `record` holds, resumed against the project `sources`
/// names, its later records kept in the same file and its tool steps
/// performed in a sandbox - or why it was not resumed.
pub async fn resume(sources: Sources<'_>, record: &Path) -> Result<Stopped, Refusal> {
    resume_in(sources, record, Sandbox::new).await
}

/// [`resume`], performing tool steps in the container `make` makes.
// @A run resumed from the record its file holds,IMPL_RUNNER_RESUME,impl,[CREQ_RUNNER_RESUMES, CREQ_RUNNER_HANDS_BACK_HOW],[DEC_RUNNER_ON_ONE_THREAD]
async fn resume_in<C: Container>(
    sources: Sources<'_>,
    record: &Path,
    make: impl FnOnce(&Grants, &Path) -> C,
) -> Result<Stopped, Refusal> {
    let (project, models) = read(sources)?;
    let mut prepared = prepared(sources, &project, record, make, Unreachable::GoesOn)?;
    let (keeper, text) = RecordKeeper::resume(record).map_err(Refusal::Record)?;
    let mut kept = Kept::new(keeper);
    let behaviours = project.behaviours();
    let outcome = resume_scripted(
        project.definition(),
        &behaviours,
        models.roster(),
        &text,
        project.limits(),
        |record| kept.keep(record),
    )
    .await
    .map(|(outcome, _)| outcome);
    let run = Tooled {
        project: &project,
        behaviours: &behaviours,
        models: &models,
    };
    let (outcome, engine) = run.tools(prepared.as_mut(), &mut kept, outcome).await;
    stopped(kept, outcome, engine)
}

/// The run whose record `record` holds, continued with `text` as the output of
/// the step it awaits for `instance`, its later records kept in the same file
/// and its tool steps performed in a sandbox - or why it was not. A refused
/// answer leaves the file as it was.
pub async fn answer(
    sources: Sources<'_>,
    record: &Path,
    instance: &str,
    text: &str,
) -> Result<Stopped, Refusal> {
    answer_in(sources, record, instance, text, Sandbox::new).await
}

/// [`answer`], performing tool steps in the container `make` makes.
// @A person's text answering the record its file holds,IMPL_RUNNER_ANSWER,impl,[CREQ_RUNNER_ANSWERS, CREQ_RUNNER_HANDS_BACK_HOW],[DEC_ANSWER_ONCE_BY_THE_KEEPER, DEC_RUNNER_ON_ONE_THREAD]
async fn answer_in<C: Container>(
    sources: Sources<'_>,
    record: &Path,
    instance: &str,
    text: &str,
    make: impl FnOnce(&Grants, &Path) -> C,
) -> Result<Stopped, Refusal> {
    let (project, models) = read(sources)?;
    let mut prepared = prepared(sources, &project, record, make, Unreachable::GoesOn)?;
    let (keeper, held) = RecordKeeper::resume(record).map_err(Refusal::Record)?;
    let mut kept = Kept::new(keeper);
    kept.latest = held.clone();
    let behaviours = project.behaviours();
    let outcome = answer_scripted(
        project.definition(),
        &behaviours,
        models.roster(),
        &held,
        instance,
        text,
        project.limits(),
        |record| kept.keep(record),
    )
    .await
    .map(|(outcome, _)| outcome);
    let run = Tooled {
        project: &project,
        behaviours: &behaviours,
        models: &models,
    };
    let (outcome, engine) = run.tools(prepared.as_mut(), &mut kept, outcome).await;
    stopped(kept, outcome, engine)
}

/// Every reason a run of the project `sources` names would be refused that
/// does not depend on its arguments or a record: the project's fault or the
/// model mapping's, then every wiring defect and every script's fault, then
/// the grants' absence or fault and every tool they do not provide for. Empty
/// when there is none. Nothing runs, no file is written, and the engine is
/// asked whether the image is ready and nothing else.
pub fn check(sources: Sources<'_>) -> Vec<Finding> {
    check_in(sources, Sandbox::new)
}

/// [`check`], asking the container `make` makes whether its image is ready.
// @What would refuse a run found without running it,IMPL_RUNNER_CHECK,impl,[CREQ_RUNNER_CHECKS, CREQ_RUNNER_CHECKS_TOOLS, CREQ_RUNNER_CHECKS_TOOL_ENVIRONMENT],[DEC_UNREACHABLE_ENGINE_REFUSES_NO_RUN]
fn check_in<C: Container>(
    sources: Sources<'_>,
    make: impl FnOnce(&Grants, &Path) -> C,
) -> Vec<Finding> {
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
        match prepared(sources, &project, Path::new(""), make, Unreachable::Refuses) {
            Ok(_) => {}
            Err(Refusal::NoGrants) => findings.push(Finding::NoGrants),
            Err(Refusal::Grants(fault)) => findings.push(Finding::Grants(fault)),
            Err(Refusal::Ungranted(faults)) => {
                findings.extend(faults.into_iter().map(Finding::Tool));
            }
            Err(other) => unreachable!("preparing refuses for the grants alone, not {other}"),
        }
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

/// Whether a container engine that cannot be reached refuses what is asked.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Unreachable {
    /// It is reported: a check.
    Refuses,
    /// The run goes on, and its first tool step meets it: a start, a resume or
    /// an answer.
    GoesOn,
}

/// The containers a run's tool steps are performed in, and the grants' image
/// a tool naming none runs in.
struct Prepared<C> {
    container: C,
    image: String,
}

/// The containers a run's tool steps are performed in, made by `make` from the
/// grants `sources` names for the run whose record file is `record` - or
/// `None` for a run whose manifest names no tool and that was given no
/// grants. Refused for the grants' absence when the manifest names a tool, for
/// grants that cannot be read, and for every tool they do not provide for; an
/// engine out of reach refuses only when `unreachable` says it does.
// @Grants read and the run's tools provided for before anything else,IMPL_RUNNER_PREPARED,impl,[CREQ_RUNNER_REFUSES_WITHOUT_GRANTS, CREQ_RUNNER_REFUSES_UNGRANTED, CREQ_RUNNER_REFUSES_UNGRANTED_IMAGE],[DEC_GRANTS_IN_A_FILE_OF_THEIR_OWN, DEC_IMAGE_BY_DIGEST_NEVER_PULLED, DEC_GRANTS_LIST_IMAGES, DEC_UNREACHABLE_ENGINE_REFUSES_NO_RUN]
fn prepared<C: Container>(
    sources: Sources<'_>,
    project: &Project,
    record: &Path,
    make: impl FnOnce(&Grants, &Path) -> C,
    unreachable: Unreachable,
) -> Result<Option<Prepared<C>>, Refusal> {
    let grants = match sources.grants {
        Some(grants) => read_grants(grants).map_err(Refusal::Grants)?,
        None if project.tools().is_empty() => return Ok(None),
        None => return Err(Refusal::NoGrants),
    };
    let container = make(&grants, record);
    let mut faults = ungranted(project, &grants, &container);
    if unreachable == Unreachable::GoesOn {
        faults.retain(|fault| !matches!(fault, ToolFault::NotReady(NotReady::Engine(_))));
    }
    if faults.is_empty() {
        Ok(Some(Prepared {
            container,
            image: grants.image().to_owned(),
        }))
    } else {
        Err(Refusal::Ungranted(faults))
    }
}

/// Every reason `grants` do not provide for `project`'s tools: each tool
/// whose action they do not allow, each parameter a tool's node type does not
/// declare that its action takes, each tool naming an image they do not list,
/// and each image the tools run in not ready in `container`, asked once per
/// image and no more once the engine is out of reach. Nothing is asked of
/// `container` for a project naming no tool.
fn ungranted(project: &Project, grants: &Grants, container: &impl Container) -> Vec<ToolFault> {
    if project.tools().is_empty() {
        return Vec::new();
    }
    let mut faults = Vec::new();
    let mut images: Vec<&str> = Vec::new();
    for tool in project.tools() {
        if !grants.allows(tool.action) {
            faults.push(ToolFault::NotGranted {
                node_type: tool.node_type.clone(),
                action: tool.action,
            });
        }
        let declared = project
            .definition()
            .node_types
            .iter()
            .find(|node_type| node_type.name == tool.node_type);
        for parameter in parameters(tool.action) {
            let has = declared.is_none_or(|node_type| {
                node_type
                    .required
                    .iter()
                    .any(|declared| declared.name == *parameter)
            });
            if !has {
                faults.push(ToolFault::MissingParameter {
                    node_type: tool.node_type.clone(),
                    action: tool.action,
                    parameter,
                });
            }
        }
        let image = tool.image.as_deref().unwrap_or(grants.image());
        if !grants.allows_image(image) {
            faults.push(ToolFault::ImageNotGranted {
                node_type: tool.node_type.clone(),
                image: image.to_owned(),
            });
        } else if !images.contains(&image) {
            images.push(image);
        }
    }
    for image in images {
        match container.ready(image) {
            Ok(()) => {}
            Err(why @ NotReady::Engine(_)) => {
                faults.push(ToolFault::NotReady(why));
                break;
            }
            Err(why) => faults.push(ToolFault::NotReady(why)),
        }
    }
    faults
}

/// A record keeper, and the latest record the run handed it.
struct Kept {
    keeper: RecordKeeper,
    latest: String,
}

impl Kept {
    fn new(keeper: RecordKeeper) -> Self {
        Self {
            keeper,
            latest: String::new(),
        }
    }

    /// `record` kept in the file, and held as the latest.
    fn keep(&mut self, record: String) {
        self.keeper.keep(&record);
        self.latest = record;
    }
}

/// What a run's tool steps are answered against: its project, the behaviours
/// the scripted run was given, and its models.
struct Tooled<'r> {
    project: &'r Project,
    behaviours: &'r agconflo_lua::Behaviours,
    models: &'r ModelMap,
}

impl Tooled<'_> {
    /// `outcome`, or - while it awaits a step of a node type the manifest
    /// names as a tool - where the run stopped once each such step was
    /// performed in its tool's container and image and answered with its
    /// text, each record kept.
    /// A step the engine failed to perform is handed back as awaited, with the
    /// engine's failure.
    // @Each awaited tool step performed in its tool's environment and answered with its text,IMPL_RUNNER_TOOLS,impl,[CREQ_RUNNER_PERFORMS_TOOLS, CREQ_RUNNER_ENGINE_FAILURE_STOPS, CREQ_RUNNER_STEPS_IN_THEIR_ENVIRONMENT],[DEC_TOOL_PERFORMED_BY_THE_RUNNER, DEC_ENGINE_FAILURE_LEAVES_THE_STEP, DEC_ONE_CONTAINER_PER_NAME, DEC_TOOL_NAMES_ITS_ENVIRONMENT]
    async fn tools<C: Container>(
        &self,
        mut prepared: Option<&mut Prepared<C>>,
        kept: &mut Kept,
        mut outcome: Result<Outcome, ScriptedRefusal>,
    ) -> (Result<Outcome, ScriptedRefusal>, Option<EngineFailure>) {
        loop {
            let Ok(Outcome::Awaiting(activation)) = &outcome else {
                return (outcome, None);
            };
            let (Some(tool), Some(Prepared { container, image })) = (
                self.project.tool(activation.node_type()),
                prepared.as_deref_mut(),
            ) else {
                return (outcome, None);
            };
            let environment = Environment {
                container: &tool.container,
                image: tool.image.as_deref().unwrap_or(image.as_str()),
            };
            let text = match perform(tool.action, environment, activation.inputs(), container) {
                Ok(text) => text,
                Err(failure) => return (outcome, Some(failure)),
            };
            let instance = activation.instance().to_owned();
            let held = kept.latest.clone();
            outcome = answer_scripted(
                self.project.definition(),
                self.behaviours,
                self.models.roster(),
                &held,
                &instance,
                &text,
                self.project.limits(),
                |record| kept.keep(record),
            )
            .await
            .map(|(outcome, _)| outcome);
        }
    }
}

/// How a run stopped, with its record keeper let go, its report beside it,
/// and the engine's failure an awaited tool step was left for.
fn stopped(
    kept: Kept,
    outcome: Result<Outcome, ScriptedRefusal>,
    engine: Option<EngineFailure>,
) -> Result<Stopped, Refusal> {
    let unkept = kept.keeper.release();
    let outcome = outcome.map_err(Refusal::Run)?;
    Ok(Stopped {
        outcome,
        unkept,
        engine,
    })
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

/// Every node type the tests' workflows use: an entry taking a brief, and four
/// types taking what came before - one asking a
/// model, one adding to it, one a person performs, and one whose script fails.
#[cfg(test)]
const TYPES: &str = "[types.begin]\nrequired = { brief = \"note\" }\noutput = \"note\"\n\n[types.ask]\nrequired = { before = \"note\" }\noutput = \"note\"\n\n[types.add]\nrequired = { before = \"note\" }\noutput = \"note\"\n\n[types.review]\nrequired = { before = \"note\" }\noutput = \"note\"\n\n[types.broken]\nrequired = { before = \"note\" }\noutput = \"note\"\n";

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
            engine: None,
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
        grants: None,
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
        grants: None,
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
            "types.toml",
            TYPES.replace("required = { brief = \"note\" }", "required = { brief = \"note\", extra = \"other\" }"),
        );
        scratch.write(
            "begin.lua",
            "local given, host = ...\nreturn host.compose(host.output, {given.brief, given.extra}, '|')\n",
        );
        let arguments = [
            Argument { instance: "first".to_owned(), parameter: "brief".to_owned(), text: wanted.clone() },
            Argument { instance: "first".to_owned(), parameter: "extra".to_owned(), text: extra.clone() },
        ];
        let sources = Sources { manifest: &manifest, models: None, grants: None };

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
        grants: None,
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
        grants: None,
    };
    runtime().block_on(interrupted(start(sources, &record, &brief("x")), &holding));

    // Resumed against a provider that does not answer, the file is held for as
    // long as the resume waits.
    let waiting = Stub::holding();
    let sources = Sources {
        manifest: &manifest,
        models: Some(&models(&scratch, "waiting.toml", &waiting)),
        grants: None,
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
        grants: None,
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
        grants: None,
    };
    let record = scratch.path("run.toml");

    match runtime().block_on(start(sources, &record, &brief("x"))) {
        Ok(Stopped {
            outcome: Outcome::Awaiting(activation),
            unkept: None,
            engine: None,
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
        grants: None,
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
        grants: None,
    };
    let findings = check(sources);
    assert!(findings.is_empty(), "{findings:?}");

    scratch.write(
        "flow.toml",
        "name = \"broken\"\noutput = \"third\"\n\n[instances.first]\nnode_type = \"begin\"\nentry = true\n\n[instances.second]\nnode_type = \"add\"\nbindings = { before = \"nobody\" }\n\n[instances.third]\nnode_type = \"add\"\nbindings = { before = \"gone\" }\n",
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
        grants: None,
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
        grants: None,
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
        grants: None,
    };
    assert!(matches!(
        runtime().block_on(start(sources, &record, &brief("x"))),
        Err(Refusal::Models(_))
    ));
    assert!(!record.exists() && !lock.exists());

    let sources = Sources {
        manifest: &manifest,
        models: None,
        grants: None,
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
            grants: None,
        };
        runtime().block_on(start(sources, &scratch.path(file), &brief("x")))
    };

    assert_eq!(completed(run("add", 5, "completes.toml")), "x+a+c+c");
    assert!(matches!(
        run("review", 5, "awaits.toml"),
        Ok(Stopped {
            outcome: Outcome::Awaiting(_),
            unkept: None,
            engine: None,
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
            engine: None,
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
            unkept: None,
            engine: None,
        })
    ));

    scratch.write(
        "flow.toml",
        "name = \"cycle\"\noutput = \"c1\"\n\n[instances.c1]\nnode_type = \"add\"\nbindings = { before = \"c2\" }\n\n[instances.c2]\nnode_type = \"add\"\nbindings = { before = \"c1\" }\n",
    );
    let sources = Sources {
        manifest: &scratch.path("manifest.toml"),
        models: None,
        grants: None,
    };
    match runtime().block_on(start(sources, &scratch.path("stuck.toml"), &[])) {
        Ok(Stopped {
            outcome: Outcome::Ended(RunEnding::Quiescent { waiting }),
            unkept: None,
            engine: None,
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
        grants: None,
    };
    match runtime().block_on(start(sources, &record, &brief("x"))) {
        Ok(Stopped {
            outcome: Outcome::Ended(RunEnding::Completed(result)),
            unkept: Some(unkept),
            engine: None,
        }) => {
            assert_eq!(result.render(), "x+a+c+c");
            assert_eq!(unkept.holds, 0);
            assert!(unkept.handed > 0);
        }
        other => panic!("expected the run completed with its record not kept, got {other:?}"),
    }
}

#[cfg(test)]
use crate::testing::{Asked, IMAGE, PYTHON, StandIn, labelled, needs_docker, needs_image};
#[cfg(test)]
use agconflo_core::Activation;

/// The node types a tool performs in the tests' workflows: `fetch` reads a
/// path, `exec` runs a command.
#[cfg(test)]
const TOOL_TYPES: &str = "[types.fetch]\nrequired = { path = \"note\" }\noutput = \"note\"\n\n[types.exec]\nrequired = { command = \"note\" }\noutput = \"note\"\n";

/// A chain from the entry through `fetch` reading what the entry made, `exec`
/// running what that read, `add`, and a person's review.
#[cfg(test)]
const TOOL_FLOW: &str = "name = \"tooled\"\noutput = \"reviewed\"\n\n[instances.first]\nnode_type = \"begin\"\nentry = true\n\n[instances.fetched]\nnode_type = \"fetch\"\nbindings = { path = \"first\" }\n\n[instances.executed]\nnode_type = \"exec\"\nbindings = { command = \"fetched\" }\n\n[instances.joined]\nnode_type = \"add\"\nbindings = { before = \"executed\" }\n\n[instances.reviewed]\nnode_type = \"review\"\nbindings = { before = \"joined\" }\n";

/// A project in `scratch` whose workflow is `flow`, with the tests' scripts,
/// `persons` performing what they name, `tools` naming each tool's node type
/// and action, and `extra` at the manifest's end. Returns the manifest.
#[cfg(test)]
fn tooled(
    scratch: &Scratch,
    flow: &str,
    persons: &[&str],
    tools: &[(&str, &str)],
    extra: &str,
) -> std::path::PathBuf {
    project(scratch, "add", 20);
    scratch.write("tools.toml", TOOL_TYPES);
    scratch.write("flow.toml", flow);
    let persons: Vec<String> = persons.iter().map(|p| format!("\"{p}\"")).collect();
    let tools: String = tools
        .iter()
        .map(|(node_type, action)| match action.starts_with('{') {
            true => format!("{node_type} = {action}\n"),
            false => format!("{node_type} = \"{action}\"\n"),
        })
        .collect();
    scratch.write(
        "manifest.toml",
        format!(
            "workflow = \"flow.toml\"\ntypes = [\"types.toml\", \"tools.toml\"]\nbudget = 20\npersons = [{}]\n\n[scripts]\nbegin = \"begin.lua\"\nask = \"ask.lua\"\nadd = \"add.lua\"\nbroken = \"broken.lua\"\n\n[tools]\n{tools}{extra}",
            persons.join(", ")
        ),
    )
}

/// A grants file in `scratch` of the test image, allowing `actions`, written
/// as a TOML array's items.
#[cfg(test)]
fn granting(scratch: &Scratch, actions: &str) -> std::path::PathBuf {
    scratch.write(
        "grants.toml",
        format!("image = \"{IMAGE}\"\nimages = [\"{PYTHON}\"]\nactions = [{actions}]\n"),
    )
}

/// A stand-in answering a read of a path with `read <path>` and a run of a
/// command with `ran <command>`.
#[cfg(test)]
fn echoing() -> StandIn {
    StandIn::with(|asked| {
        let output = match asked {
            Asked::Read(path) => format!("read {path}"),
            Asked::Run(command) => format!("ran {command}"),
            other => format!("{other:?}"),
        };
        Ok(crate::Done { status: 0, output })
    })
}

/// The steps a stand-in was asked to perform, leaving out whether the image
/// is ready.
#[cfg(test)]
fn steps(stand_in: &StandIn) -> Vec<Asked> {
    stand_in
        .asked()
        .into_iter()
        .filter(|asked| *asked != Asked::Ready)
        .collect()
}

/// The step a run awaits, its record kept and no engine failure beside it, or
/// a panic naming how it stopped.
#[cfg(test)]
fn awaiting(stopped: Result<Stopped, Refusal>) -> Activation {
    match stopped {
        Ok(Stopped {
            outcome: Outcome::Awaiting(activation),
            unkept: None,
            engine: None,
        }) => activation,
        other => panic!("expected the run to await a step, got {other:?}"),
    }
}

/// The rendering of `activation`'s input for `parameter`.
#[cfg(test)]
fn input(activation: &Activation, parameter: &str) -> String {
    activation
        .inputs()
        .iter()
        .find(|(name, _)| name == parameter)
        .map(|(_, context)| context.render().into_owned())
        .unwrap_or_else(|| panic!("no input {parameter}"))
}

#[cfg(test)]
#[test]
fn performs_tool_steps() {
    let scratch = Scratch::new("performs_tool_steps");
    let grants = granting(&scratch, "\"read\", \"run\"");
    let manifest = tooled(
        &scratch,
        TOOL_FLOW,
        &["review"],
        &[("fetch", "read"), ("exec", "run")],
        "",
    );
    let sources = Sources {
        manifest: &manifest,
        models: None,
        grants: Some(&grants),
    };
    let stand_in = echoing();

    let awaited = awaiting(runtime().block_on(start_in(
        sources,
        &scratch.path("run.toml"),
        &brief("x"),
        |_, _| stand_in.clone(),
    )));

    assert_eq!(awaited.instance(), "reviewed");
    assert_eq!(input(&awaited, "before"), "exit status 0\nran read x+a+c");
    assert_eq!(
        steps(&stand_in),
        [
            Asked::Read("x+a".to_owned()),
            Asked::Run("read x+a".to_owned())
        ]
    );

    // A record awaiting the run step: exec performed by a person when it was
    // made, and by a tool when it is resumed.
    let as_person = tooled(
        &scratch,
        TOOL_FLOW,
        &["review", "exec"],
        &[("fetch", "read")],
        "",
    );
    let record = scratch.path("awaiting-exec.toml");
    let stand_in = echoing();
    let awaited = awaiting(runtime().block_on(start_in(
        Sources {
            manifest: &as_person,
            models: None,
            grants: Some(&grants),
        },
        &record,
        &brief("y"),
        |_, _| stand_in.clone(),
    )));
    assert_eq!(awaited.instance(), "executed");

    let as_tool = tooled(
        &scratch,
        TOOL_FLOW,
        &["review"],
        &[("fetch", "read"), ("exec", "run")],
        "",
    );
    let stand_in = echoing();
    let awaited = awaiting(runtime().block_on(resume_in(
        Sources {
            manifest: &as_tool,
            models: None,
            grants: Some(&grants),
        },
        &record,
        |_, _| stand_in.clone(),
    )));
    assert_eq!(awaited.instance(), "reviewed");
    assert_eq!(input(&awaited, "before"), "exit status 0\nran read y+a+c");
    assert_eq!(steps(&stand_in), [Asked::Run("read y+a".to_owned())]);
}

/// Text of ASCII and wider characters with every kind of line ending, empty
/// among them.
#[cfg(test)]
fn any_answer() -> impl Strategy<Value = String> {
    let piece = prop_oneof![
        Just("\n".to_owned()),
        Just("\r\n".to_owned()),
        Just("\r".to_owned()),
        Just("\u{e9}".to_owned()),
        Just("\u{1f600}".to_owned()),
        "[ -~]{0,6}",
    ];
    prop::collection::vec(piece, 0..12).prop_map(|pieces| pieces.concat())
}

#[cfg(test)]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]
    // @Few cases for runs that each build a model client,TRACE_RUNNER_TOOL_PROPERTY_CASES,trace,[],[NOTE_RUNNER_PROPERTY_CASES]
    #[test]
    fn tool_text_is_the_output(text in any_answer()) {
        let scratch = Scratch::new("tool_text_is_the_output");
        let grants = granting(&scratch, "\"read\"");
        let manifest = tooled(
            &scratch,
            "name = \"fetching\"\noutput = \"fetched\"\n\n[instances.first]\nnode_type = \"begin\"\nentry = true\n\n[instances.fetched]\nnode_type = \"fetch\"\nbindings = { path = \"first\" }\n",
            &["review"],
            &[("fetch", "read")],
            "",
        );
        let answer = text.clone();
        let stand_in = StandIn::with(move |_| Ok(crate::Done { status: 0, output: answer.clone() }));
        let sources = Sources { manifest: &manifest, models: None, grants: Some(&grants) };

        let stopped = runtime().block_on(start_in(sources, &scratch.path("run.toml"), &brief("x"), |_, _| stand_in.clone()));

        match stopped {
            Ok(Stopped { outcome: Outcome::Ended(RunEnding::Completed(result)), unkept: None, engine: None }) => {
                prop_assert_eq!(result.render(), text);
                prop_assert_eq!(result.declared_type().as_str(), "note");
            }
            other => panic!("expected the run completed, got {other:?}"),
        }
    }
}

#[cfg(test)]
#[test]
fn recorded_tool_step_not_repeated() {
    let scratch = Scratch::new("recorded_tool_step_not_repeated");
    let grants = granting(&scratch, "\"read\", \"run\"");
    let manifest = tooled(
        &scratch,
        TOOL_FLOW,
        &["review"],
        &[("fetch", "read"), ("exec", "run")],
        "",
    );
    let sources = Sources {
        manifest: &manifest,
        models: None,
        grants: Some(&grants),
    };
    let record = scratch.path("run.toml");
    let first = echoing();
    awaiting(
        runtime().block_on(start_in(sources, &record, &brief("x"), |_, _| {
            first.clone()
        })),
    );
    assert_eq!(steps(&first).len(), 2);

    let fresh = echoing();
    let awaited = awaiting(runtime().block_on(resume_in(sources, &record, |_, _| fresh.clone())));

    assert_eq!(awaited.instance(), "reviewed");
    assert_eq!(steps(&fresh), []);
}

/// A workflow of the entry and an asker whose model may call `fetch` `calls`
/// times, the asker designated.
#[cfg(test)]
fn asking_flow(calls: usize) -> String {
    let calls = vec!["\"fetch\""; calls].join(", ");
    format!(
        "name = \"asking\"\noutput = \"asker\"\n\n[instances.first]\nnode_type = \"begin\"\nentry = true\n\n[instances.asker]\nnode_type = \"ask\"\nbindings = {{ before = \"first\" }}\ncalls = [{calls}]\n"
    )
}

/// A stub reply calling `fetch` under `id` with `path`.
#[cfg(test)]
fn fetching(id: &str, path: &str) -> (String, Vec<crate::testing::Call>) {
    (
        String::new(),
        vec![(
            id.to_owned(),
            "fetch".to_owned(),
            format!("{{\"path\": \"{path}\"}}"),
        )],
    )
}

#[cfg(test)]
#[test]
fn tool_steps_count_against_budget() {
    let scratch = Scratch::new("tool_steps_count_against_budget");
    let grants = granting(&scratch, "\"read\"");
    let manifest = tooled(
        &scratch,
        &asking_flow(12),
        &["review"],
        &[("fetch", "read")],
        "\n[limits]\nmodel_calls = 20\n",
    );
    let text = std::fs::read_to_string(&manifest)
        .expect("the manifest")
        .replace("budget = 20", "budget = 5");
    std::fs::write(&manifest, text).expect("the manifest");
    let stub = Stub::replying((0..12).map(|i| fetching(&format!("c{i}"), "a")).collect());
    let models = models(&scratch, "models.toml", &stub);
    let stand_in = echoing();

    let stopped = runtime().block_on(start_in(
        Sources {
            manifest: &manifest,
            models: Some(&models),
            grants: Some(&grants),
        },
        &scratch.path("run.toml"),
        &brief("x"),
        |_, _| stand_in.clone(),
    ));

    assert!(
        matches!(
            stopped,
            Ok(Stopped {
                outcome: Outcome::Ended(RunEnding::BudgetExceeded { budget: 5 }),
                unkept: None,
                engine: None,
            })
        ),
        "{stopped:?}"
    );
    let performed = steps(&stand_in).len();
    assert!((1..5).contains(&performed), "{performed}");
}

#[cfg(test)]
#[test]
fn model_calls_a_tool() {
    let scratch = Scratch::new("model_calls_a_tool");
    let grants = granting(&scratch, "\"read\"");
    let manifest = tooled(
        &scratch,
        &asking_flow(2),
        &["review"],
        &[("fetch", "read")],
        "\n[limits]\nmodel_calls = 8\n",
    );
    let stub = Stub::replying(vec![
        fetching("c1", "notes/one"),
        fetching("c2", "notes/two"),
        ("It says one and two.".to_owned(), Vec::new()),
    ]);
    let models = models(&scratch, "models.toml", &stub);
    let stand_in = echoing();

    let stopped = runtime().block_on(start_in(
        Sources {
            manifest: &manifest,
            models: Some(&models),
            grants: Some(&grants),
        },
        &scratch.path("run.toml"),
        &brief("x"),
        |_, _| stand_in.clone(),
    ));

    assert_eq!(completed(stopped), "It says one and two.+b");
    assert_eq!(
        steps(&stand_in),
        [
            Asked::Read("notes/one".to_owned()),
            Asked::Read("notes/two".to_owned())
        ]
    );
    let bodies = stub.bodies();
    assert_eq!(bodies.len(), 3);
    assert!(
        bodies[2].contains("read notes/one") && bodies[2].contains("read notes/two"),
        "{}",
        bodies[2]
    );
}

#[cfg(test)]
#[test]
fn refuses_without_grants() {
    let scratch = Scratch::new("refuses_without_grants");
    let manifest = tooled(
        &scratch,
        TOOL_FLOW,
        &["review"],
        &[("fetch", "read"), ("exec", "run")],
        "",
    );
    let record = scratch.path("run.toml");
    let lock = scratch.path("run.toml.lock");
    let stand_in = echoing();
    let without = Sources {
        manifest: &manifest,
        models: None,
        grants: None,
    };

    let refusals = [
        runtime().block_on(start_in(without, &record, &brief("x"), |_, _| {
            stand_in.clone()
        })),
        runtime().block_on(resume_in(without, &record, |_, _| stand_in.clone())),
        runtime().block_on(answer_in(without, &record, "fetched", "t", |_, _| {
            stand_in.clone()
        })),
    ];
    for refused in refusals {
        assert!(matches!(refused, Err(Refusal::NoGrants)), "{refused:?}");
    }
    let absent = scratch.path("absent-grants.toml");
    let unreadable = runtime().block_on(start_in(
        Sources {
            grants: Some(&absent),
            ..without
        },
        &record,
        &brief("x"),
        |_, _| stand_in.clone(),
    ));
    assert!(
        matches!(unreadable, Err(Refusal::Grants(GrantsFault::File(_)))),
        "{unreadable:?}"
    );
    assert!(!record.exists() && !lock.exists());
    assert_eq!(stand_in.asked(), []);

    // The same manifest without its tools needs no grants.
    let untooled = project(&scratch, "add", 5);
    assert_eq!(
        completed(runtime().block_on(start_in(
            Sources {
                manifest: &untooled,
                models: None,
                grants: None,
            },
            &record,
            &brief("x"),
            |_, _| stand_in.clone(),
        ))),
        "x+a+c+c"
    );
}

#[cfg(test)]
#[test]
fn refuses_ungranted() {
    let scratch = Scratch::new("refuses_ungranted");
    let record = scratch.path("run.toml");
    let taken = || record.exists() || scratch.path("run.toml.lock").exists();
    let refused = |manifest: &std::path::Path, grants: &std::path::Path, stand_in: StandIn| {
        let stopped = runtime().block_on(start_in(
            Sources {
                manifest,
                models: None,
                grants: Some(grants),
            },
            &record,
            &brief("x"),
            |_, _| stand_in,
        ));
        match stopped {
            Err(Refusal::Ungranted(faults)) => faults,
            other => panic!("expected the run refused for its tools, got {other:?}"),
        }
    };

    let manifest = tooled(
        &scratch,
        TOOL_FLOW,
        &["review"],
        &[("fetch", "read"), ("exec", "run")],
        "",
    );
    let writing = granting(&scratch, "\"write\"");
    assert_eq!(
        refused(&manifest, &writing, echoing()),
        [
            ToolFault::NotGranted {
                node_type: "fetch".to_owned(),
                action: Action::Read,
            },
            ToolFault::NotGranted {
                node_type: "exec".to_owned(),
                action: Action::Run,
            },
        ]
    );
    assert!(!taken());

    let all = granting(&scratch, "\"read\", \"run\"");
    scratch.write(
        "tools.toml",
        format!("{TOOL_TYPES}\n[types.bare]\nrequired = {{ other = \"note\" }}\noutput = \"note\"\n\n[types.loose]\nrequired = {{ path = \"note\" }}\noutput = \"note\"\n"),
    );
    let text = std::fs::read_to_string(&manifest).expect("the manifest");
    std::fs::write(&manifest, format!("{text}bare = \"read\"\n")).expect("the manifest");
    assert_eq!(
        refused(&manifest, &all, echoing()),
        [ToolFault::MissingParameter {
            node_type: "bare".to_owned(),
            action: Action::Read,
            parameter: "path",
        }]
    );
    assert!(!taken());

    std::fs::write(&manifest, format!("{text}loose = \"read\"\n")).expect("the manifest");
    let absent = crate::NotReady::Absent {
        image: IMAGE.to_owned(),
    };
    assert_eq!(
        refused(&manifest, &all, echoing().not_ready(absent.clone())),
        [ToolFault::NotReady(absent)]
    );
    assert!(!taken());

    // Its path declared, the image the grants name ready: the same manifest
    // starts.
    let stand_in = echoing();
    awaiting(runtime().block_on(start_in(
        Sources {
            manifest: &manifest,
            models: None,
            grants: Some(&all),
        },
        &record,
        &brief("x"),
        |_, _| stand_in.clone(),
    )));
}

#[cfg(test)]
#[test]
fn engine_failure_leaves_the_step() {
    let scratch = Scratch::new("engine_failure_leaves_the_step");
    let grants = granting(&scratch, "\"read\", \"run\"");
    let manifest = tooled(
        &scratch,
        TOOL_FLOW,
        &["review"],
        &[("fetch", "read"), ("exec", "run")],
        "",
    );
    let sources = Sources {
        manifest: &manifest,
        models: None,
        grants: Some(&grants),
    };
    let record = scratch.path("run.toml");
    let failing = StandIn::with(|asked| match asked {
        Asked::Run(_) => Err(EngineFailure {
            message: "the engine went away".to_owned(),
        }),
        _ => Ok(crate::Done {
            status: 0,
            output: "R".to_owned(),
        }),
    });

    match runtime().block_on(start_in(sources, &record, &brief("x"), |_, _| {
        failing.clone()
    })) {
        Ok(Stopped {
            outcome: Outcome::Awaiting(activation),
            unkept: None,
            engine: Some(EngineFailure { message }),
        }) => {
            assert_eq!(activation.instance(), "executed");
            assert_eq!(message, "the engine went away");
        }
        other => panic!("expected the run left awaiting the step, got {other:?}"),
    }
    let copy = scratch.write("copy.toml", std::fs::read(&record).expect("the record"));

    let answering = echoing();
    let awaited =
        awaiting(runtime().block_on(resume_in(sources, &record, |_, _| answering.clone())));
    assert_eq!(awaited.instance(), "reviewed");
    assert_eq!(input(&awaited, "before"), "exit status 0\nran R+c");

    let untouched = echoing();
    let awaited =
        awaiting(
            runtime().block_on(answer_in(sources, &copy, "executed", "typed", |_, _| {
                untouched.clone()
            })),
        );
    assert_eq!(input(&awaited, "before"), "typed+c");
    assert_eq!(steps(&untouched), []);
}

#[cfg(test)]
#[test]
fn check_reports_tools() {
    let scratch = Scratch::new("check_reports_tools");
    let manifest = tooled(
        &scratch,
        TOOL_FLOW,
        &["review"],
        &[("fetch", "read"), ("exec", "run")],
        "",
    );
    let stand_in = echoing();
    let checked = |grants: Option<&std::path::Path>, stand_in: StandIn| {
        check_in(
            Sources {
                manifest: &manifest,
                models: None,
                grants,
            },
            |_, _| stand_in,
        )
    };

    let findings = checked(None, stand_in.clone());
    assert!(
        matches!(findings.as_slice(), [Finding::NoGrants]),
        "{findings:?}"
    );

    let reading = granting(&scratch, "\"read\"");
    let findings = checked(Some(&reading), stand_in.clone());
    assert!(
        matches!(
            findings.as_slice(),
            [Finding::Tool(ToolFault::NotGranted { node_type, action: Action::Run })] if node_type == "exec"
        ),
        "{findings:?}"
    );

    let all = granting(&scratch, "\"read\", \"run\"");
    let absent = crate::NotReady::Absent {
        image: IMAGE.to_owned(),
    };
    let not_ready = echoing().not_ready(absent.clone());
    let findings = checked(Some(&all), not_ready.clone());
    assert!(
        matches!(findings.as_slice(), [Finding::Tool(ToolFault::NotReady(why))] if *why == absent),
        "{findings:?}"
    );

    // Everything provided for: nothing found, and still no container asked for.
    let findings = checked(Some(&all), stand_in.clone());
    assert!(findings.is_empty(), "{findings:?}");

    assert_eq!(stand_in.asked(), [Asked::Ready, Asked::Ready]);
    assert_eq!(not_ready.asked(), [Asked::Ready]);
}

#[cfg(test)]
#[test]
fn tool_steps_end_to_end() {
    needs_docker();
    let scratch = Scratch::new("tool_steps_end_to_end");
    scratch.write(
        "tools.toml",
        "[types.name]\nrequired = { before = \"note\" }\noutput = \"note\"\n\n[types.save]\nrequired = { path = \"note\", text = \"note\" }\noutput = \"note\"\n\n[types.command]\nrequired = { before = \"note\" }\noutput = \"note\"\n\n[types.exec]\nrequired = { command = \"note\" }\noutput = \"note\"\n",
    );
    project(&scratch, "add", 5);
    scratch.write(
        "name.lua",
        "local given, host = ...\nreturn host.text(host.output, 'w/made.txt')\n",
    );
    scratch.write(
        "command.lua",
        "local given, host = ...\nreturn host.text(host.output, 'cat w/made.txt')\n",
    );
    scratch.write(
        "flow.toml",
        "name = \"end-to-end\"\noutput = \"reviewed\"\n\n[instances.first]\nnode_type = \"begin\"\nentry = true\n\n[instances.named]\nnode_type = \"name\"\nbindings = { before = \"first\" }\n\n[instances.saved]\nnode_type = \"save\"\nbindings = { path = \"named\", text = \"first\" }\n\n[instances.commanded]\nnode_type = \"command\"\nbindings = { before = \"saved\" }\n\n[instances.catted]\nnode_type = \"exec\"\nbindings = { command = \"commanded\" }\n\n[instances.reviewed]\nnode_type = \"review\"\nbindings = { before = \"catted\" }\n",
    );
    let manifest = scratch.write(
        "manifest.toml",
        "workflow = \"flow.toml\"\ntypes = [\"types.toml\", \"tools.toml\"]\nbudget = 10\npersons = [\"review\"]\n\n[scripts]\nbegin = \"begin.lua\"\nname = \"name.lua\"\ncommand = \"command.lua\"\n\n[tools]\nsave = \"write\"\nexec = \"run\"\n",
    );
    scratch.write("w/.keep", "");
    let grants = scratch.write(
        "grants.toml",
        format!(
            "image = \"{IMAGE}\"\nactions = [\"write\", \"run\"]\n\n[folders.w]\npath = \"w\"\nwritable = true\n"
        ),
    );
    let record = scratch.path("run.toml");

    let awaited = awaiting(runtime().block_on(start(
        Sources {
            manifest: &manifest,
            models: None,
            grants: Some(&grants),
        },
        &record,
        &brief("x"),
    )));

    assert_eq!(
        std::fs::read_to_string(scratch.path("w/made.txt")).expect("the file"),
        "x+a"
    );
    assert_eq!(awaited.instance(), "reviewed");
    assert_eq!(input(&awaited, "before"), "exit status 0\nx+a");
    assert_eq!(labelled(&record), Vec::<String>::new());
}

#[cfg(test)]
#[test]
fn steps_in_their_environment() {
    let scratch = Scratch::new("steps_in_their_environment");
    let grants = granting(&scratch, "\"read\", \"run\"");
    let build = format!("{{ action = \"run\", container = \"build\", image = \"{PYTHON}\" }}");
    let manifest = tooled(
        &scratch,
        TOOL_FLOW,
        &["review"],
        &[("fetch", "read"), ("exec", &build)],
        "",
    );
    let stand_in = echoing();
    awaiting(runtime().block_on(start_in(
        Sources {
            manifest: &manifest,
            models: None,
            grants: Some(&grants),
        },
        &scratch.path("run.toml"),
        &brief("x"),
        |_, _| stand_in.clone(),
    )));
    assert_eq!(
        stand_in.environments(),
        [
            ("tools".to_owned(), IMAGE.to_owned()),
            ("build".to_owned(), PYTHON.to_owned())
        ]
    );

    // A model's call to a tool is performed in the called tool's environment.
    let manifest = tooled(
        &scratch,
        &asking_flow(1),
        &["review"],
        &[("fetch", "{ action = \"read\", container = \"other\" }")],
        "\n[limits]\nmodel_calls = 4\n",
    );
    let stub = Stub::replying(vec![
        fetching("c1", "notes/one"),
        ("Read it.".to_owned(), Vec::new()),
    ]);
    let models = models(&scratch, "models.toml", &stub);
    let stand_in = echoing();
    let stopped = runtime().block_on(start_in(
        Sources {
            manifest: &manifest,
            models: Some(&models),
            grants: Some(&grants),
        },
        &scratch.path("called.toml"),
        &brief("x"),
        |_, _| stand_in.clone(),
    ));
    assert_eq!(completed(stopped), "Read it.+b");
    assert_eq!(
        stand_in.environments(),
        [("other".to_owned(), IMAGE.to_owned())]
    );
}

#[cfg(test)]
#[test]
fn refuses_ungranted_image() {
    let scratch = Scratch::new("refuses_ungranted_image");
    let grants = granting(&scratch, "\"read\", \"run\"");
    let record = scratch.path("run.toml");
    let unlisted = format!("python@sha256:{}", "9".repeat(64));
    let unlisted_too = format!("python@sha256:{}", "8".repeat(64));
    let start = |manifest: &std::path::Path, stand_in: StandIn| {
        runtime().block_on(start_in(
            Sources {
                manifest,
                models: None,
                grants: Some(&grants),
            },
            &record,
            &brief("x"),
            |_, _| stand_in,
        ))
    };

    let manifest = tooled(
        &scratch,
        TOOL_FLOW,
        &["review"],
        &[
            (
                "fetch",
                &format!("{{ action = \"read\", container = \"f\", image = \"{unlisted}\" }}"),
            ),
            (
                "exec",
                &format!("{{ action = \"run\", container = \"e\", image = \"{unlisted_too}\" }}"),
            ),
        ],
        "",
    );
    match start(&manifest, echoing()) {
        Err(Refusal::Ungranted(faults)) => assert_eq!(
            faults,
            [
                ToolFault::ImageNotGranted {
                    node_type: "fetch".to_owned(),
                    image: unlisted.clone(),
                },
                ToolFault::ImageNotGranted {
                    node_type: "exec".to_owned(),
                    image: unlisted_too.clone(),
                },
            ]
        ),
        other => panic!("expected both images refused, got {other:?}"),
    }
    assert!(!record.exists());

    let python = format!("{{ action = \"run\", container = \"py\", image = \"{PYTHON}\" }}");
    let manifest = tooled(
        &scratch,
        TOOL_FLOW,
        &["review"],
        &[("fetch", "read"), ("exec", &python)],
        "",
    );
    let absent = crate::NotReady::Absent {
        image: PYTHON.to_owned(),
    };
    let stand_in = echoing().not_ready_for(PYTHON, absent.clone());
    match start(&manifest, stand_in.clone()) {
        Err(Refusal::Ungranted(faults)) => assert_eq!(faults, [ToolFault::NotReady(absent)]),
        other => panic!("expected the absent image refused, got {other:?}"),
    }
    assert_eq!(stand_in.images(), [IMAGE.to_owned(), PYTHON.to_owned()]);
    assert!(!record.exists());

    // The grants' own image, named by its digest, is granted.
    let own = format!("{{ action = \"run\", container = \"x\", image = \"{IMAGE}\" }}");
    let manifest = tooled(
        &scratch,
        TOOL_FLOW,
        &["review"],
        &[("fetch", "read"), ("exec", &own)],
        "",
    );
    awaiting(start(&manifest, echoing()));
}

#[cfg(test)]
#[test]
fn unreachable_engine_not_refused() {
    let scratch = Scratch::new("unreachable_engine_not_refused");
    let grants = granting(&scratch, "\"read\", \"run\"");
    let manifest = tooled(
        &scratch,
        TOOL_FLOW,
        &["review"],
        &[("fetch", "read"), ("exec", "run")],
        "",
    );
    let sources = Sources {
        manifest: &manifest,
        models: None,
        grants: Some(&grants),
    };
    let record = scratch.path("run.toml");
    let failure = EngineFailure {
        message: "error during connect".to_owned(),
    };
    let unreachable =
        || StandIn::failing(&failure.message).not_ready(NotReady::Engine(failure.clone()));
    let left = |stopped: Result<Stopped, Refusal>| match stopped {
        Ok(Stopped {
            outcome: Outcome::Awaiting(activation),
            unkept: None,
            engine: Some(engine),
        }) => {
            assert_eq!(engine, failure);
            activation.instance().to_owned()
        }
        other => panic!("expected the step left for the engine, got {other:?}"),
    };

    let started = runtime().block_on(start_in(sources, &record, &brief("x"), |_, _| {
        unreachable()
    }));
    assert_eq!(left(started), "fetched");

    let answered = runtime().block_on(answer_in(sources, &record, "fetched", "typed", |_, _| {
        unreachable()
    }));
    assert_eq!(left(answered), "executed");

    let findings = check_in(sources, |_, _| unreachable());
    assert!(
        matches!(
            findings.as_slice(),
            [Finding::Tool(ToolFault::NotReady(NotReady::Engine(_)))]
        ),
        "{findings:?}"
    );
}

#[cfg(test)]
#[test]
fn check_reports_tool_environment() {
    let scratch = Scratch::new("check_reports_tool_environment");
    let grants = granting(&scratch, "\"read\", \"run\"");
    let unlisted = format!("python@sha256:{}", "9".repeat(64));
    let checked = |exec: &str, stand_in: StandIn| {
        let manifest = tooled(
            &scratch,
            TOOL_FLOW,
            &["review"],
            &[("fetch", "read"), ("exec", exec)],
            "",
        );
        check_in(
            Sources {
                manifest: &manifest,
                models: None,
                grants: Some(&grants),
            },
            |_, _| stand_in,
        )
    };

    let stand_in = echoing();
    let findings = checked(
        &format!("{{ action = \"run\", container = \"e\", image = \"{unlisted}\" }}"),
        stand_in.clone(),
    );
    assert!(
        matches!(
            findings.as_slice(),
            [Finding::Tool(ToolFault::ImageNotGranted { node_type, image })] if node_type == "exec" && *image == unlisted
        ),
        "{findings:?}"
    );

    let absent = crate::NotReady::Absent {
        image: PYTHON.to_owned(),
    };
    let findings = checked(
        &format!("{{ action = \"run\", container = \"py\", image = \"{PYTHON}\" }}"),
        stand_in.clone().not_ready_for(PYTHON, absent.clone()),
    );
    assert!(
        matches!(findings.as_slice(), [Finding::Tool(ToolFault::NotReady(why))] if *why == absent),
        "{findings:?}"
    );
    assert_eq!(steps(&stand_in), []);
}

/// A project in `scratch` whose entry is followed by each of `steps` - a
/// command a script names, performed by a tool of its own - and then by
/// `after`, from the tests' types beside a `pair` joining two notes. Each
/// step is `(tool, entry, command)`: the tool's node type, its manifest
/// entry, and the command its script gives. `after` is the instances that
/// follow, as the workflow document writes them, and `output` the designated
/// instance.
#[cfg(test)]
fn stepping(
    scratch: &Scratch,
    steps: &[(&str, &str, &str)],
    after: &str,
    output: &str,
    grants: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
    project(scratch, "add", 5);
    let mut types = String::from(
        "[types.pair]\nrequired = { a = \"note\", b = \"note\" }\noutput = \"note\"\n",
    );
    let mut flow = format!(
        "name = \"stepping\"\noutput = \"{output}\"\n\n[instances.first]\nnode_type = \"begin\"\nentry = true\n"
    );
    let mut scripts = String::from("begin = \"begin.lua\"\npair = \"pair.lua\"\n");
    let mut tools = String::new();
    scratch.write(
        "pair.lua",
        "local given, host = ...\nreturn host.compose(host.output, {given.a, given.b}, '|')\n",
    );
    for (tool, entry, command) in steps {
        types.push_str(&format!(
            "\n[types.{tool}_says]\nrequired = {{ before = \"note\" }}\noutput = \"note\"\n\n[types.{tool}]\nrequired = {{ command = \"note\" }}\noutput = \"note\"\n"
        ));
        flow.push_str(&format!(
            "\n[instances.{tool}_said]\nnode_type = \"{tool}_says\"\nbindings = {{ before = \"first\" }}\n\n[instances.{tool}_done]\nnode_type = \"{tool}\"\nbindings = {{ command = \"{tool}_said\" }}\n"
        ));
        scratch.write(
            &format!("{tool}_says.lua"),
            format!("local given, host = ...\nreturn host.text(host.output, [==[{command}]==])\n"),
        );
        scripts.push_str(&format!("{tool}_says = \"{tool}_says.lua\"\n"));
        tools.push_str(&format!("{tool} = {entry}\n"));
    }
    flow.push_str(after);
    scratch.write("steps.toml", types);
    scratch.write("flow.toml", flow);
    let manifest = scratch.write(
        "manifest.toml",
        format!(
            "workflow = \"flow.toml\"\ntypes = [\"types.toml\", \"steps.toml\"]\nbudget = 20\npersons = [\"review\"]\n\n[scripts]\n{scripts}\n[tools]\n{tools}"
        ),
    );
    let grants = scratch.write("grants.toml", grants);
    (manifest, grants)
}

#[cfg(test)]
#[test]
fn environments_end_to_end() {
    needs_docker();
    needs_image(PYTHON);
    let scratch = Scratch::new("environments_end_to_end");
    let python = format!("{{ action = \"run\", container = \"py\", image = \"{PYTHON}\" }}");
    let (manifest, grants) = stepping(
        &scratch,
        &[
            (
                "inpy",
                &python,
                "python3 -c 'print(2 + 3)'; cd ~ && touch here && echo home",
            ),
            (
                "plain",
                "\"run\"",
                "command -v python3 || echo no-python; cd ~ && touch here && echo home",
            ),
        ],
        "\n[instances.both]\nnode_type = \"pair\"\nbindings = { a = \"inpy_done\", b = \"plain_done\" }\n",
        "both",
        &format!("image = \"{IMAGE}\"\nimages = [\"{PYTHON}\"]\nactions = [\"run\"]\n"),
    );
    let record = scratch.path("run.toml");

    let result = completed(runtime().block_on(start(
        Sources {
            manifest: &manifest,
            models: None,
            grants: Some(&grants),
        },
        &record,
        &brief("x"),
    )));

    assert_eq!(
        result,
        "exit status 0\n5\nhome\n|exit status 0\nno-python\nhome\n"
    );
    assert_eq!(labelled(&record), Vec::<String>::new());
}

#[cfg(test)]
#[test]
fn environment_shared_end_to_end() {
    needs_docker();
    let scratch = Scratch::new("environment_shared_end_to_end");
    let build = "{ action = \"run\", container = \"build\" }";
    let (manifest, grants) = stepping(
        &scratch,
        &[
            ("make", build, "echo made > /tmp/shared && echo ok"),
            ("usemade", build, "cat /tmp/shared"),
            (
                "peek",
                "{ action = \"run\", container = \"other\" }",
                "cat /tmp/shared 2>/dev/null || echo none",
            ),
        ],
        "\n[instances.seen]\nnode_type = \"pair\"\nbindings = { a = \"usemade_done\", b = \"peek_done\" }\n\n[instances.reviewed]\nnode_type = \"review\"\nbindings = { before = \"seen\" }\n",
        "reviewed",
        &format!("image = \"{IMAGE}\"\nactions = [\"run\"]\n"),
    );
    // make runs before usemade: usemade's command waits on make's output.
    let flow = std::fs::read_to_string(scratch.path("flow.toml"))
        .expect("the flow")
        .replace(
            "[instances.usemade_said]\nnode_type = \"usemade_says\"\nbindings = { before = \"first\" }",
            "[instances.usemade_said]\nnode_type = \"usemade_says\"\nbindings = { before = \"make_done\" }",
        )
        .replace(
            "[instances.peek_said]\nnode_type = \"peek_says\"\nbindings = { before = \"first\" }",
            "[instances.peek_said]\nnode_type = \"peek_says\"\nbindings = { before = \"make_done\" }",
        );
    scratch.write("flow.toml", flow);
    let record = scratch.path("run.toml");

    let awaited = awaiting(runtime().block_on(start(
        Sources {
            manifest: &manifest,
            models: None,
            grants: Some(&grants),
        },
        &record,
        &brief("x"),
    )));

    assert_eq!(awaited.instance(), "reviewed");
    assert_eq!(
        input(&awaited, "before"),
        "exit status 0\nmade\n|exit status 0\nnone\n"
    );
    assert_eq!(labelled(&record), Vec::<String>::new());
}
