//! `agconflo`: checks, runs, resumes and answers a workflow from the documents
//! that describe it.
//!
//! A completed run's result, or the step a run awaits, is printed to standard
//! output and nothing else is; everything else goes to standard error. Exit
//! status: 0 completed, or checked with nothing found; 2 a command line that
//! cannot be read; 3 awaiting a person; 4 refused before anything ran, or a
//! check that found something; 5 a node failed; 6 the budget ran out; 7 no node
//! can make further progress; 8 the record not kept, whatever else happened.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use agconflo_core::{Activation, RunEnding};
use agconflo_lua::Outcome;
use agconflo_runner::{Argument, Finding, Refusal, Sources, Stopped};
use clap::{Args, Parser, Subcommand};

/// Checks, runs, resumes and answers a workflow from the documents that
/// describe it.
#[derive(Parser)]
#[command(name = "agconflo", version)]
struct Line {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Reports whatever would refuse a run of the manifest's workflow, and runs
    /// nothing.
    Check {
        /// The manifest naming the workflow's documents.
        manifest: PathBuf,
        /// The model mapping, for a workflow whose scripts call models.
        #[arg(long)]
        models: Option<PathBuf>,
        /// The grants file, for a workflow whose manifest names tools.
        #[arg(long)]
        grants: Option<PathBuf>,
    },
    /// Starts a new run, keeping its record in a file that must not exist yet.
    Run {
        #[command(flatten)]
        files: Files,
        /// Text for an instance's parameter that nothing binds.
        #[arg(long = "arg", num_args = 3, value_names = ["INSTANCE", "PARAMETER", "TEXT"])]
        arguments: Vec<String>,
        /// A file whose text is for an instance's parameter that nothing binds.
        #[arg(long = "arg-file", num_args = 3, value_names = ["INSTANCE", "PARAMETER", "FILE"])]
        argument_files: Vec<String>,
    },
    /// Resumes the run whose record a file holds.
    Resume {
        #[command(flatten)]
        files: Files,
    },
    /// Answers the step the run whose record a file holds awaits a person for.
    Answer {
        #[command(flatten)]
        files: Files,
        /// The instance whose step is answered.
        #[arg(long)]
        instance: String,
        #[command(flatten)]
        text: Text,
        /// For a router's step, an instance its run goes on to; once for each,
        /// and `--route ""` for none.
        #[arg(long = "route", value_name = "INSTANCE")]
        route: Vec<String>,
    },
}

/// The files every run is read from and kept in.
#[derive(Args)]
struct Files {
    /// The manifest naming the workflow's documents.
    manifest: PathBuf,
    /// The file the run's record is kept in.
    #[arg(long)]
    record: PathBuf,
    /// The model mapping, for a workflow whose scripts call models.
    #[arg(long)]
    models: Option<PathBuf>,
    /// The grants file, for a workflow whose manifest names tools.
    #[arg(long)]
    grants: Option<PathBuf>,
}

impl Files {
    fn sources(&self) -> Sources<'_> {
        Sources {
            manifest: &self.manifest,
            models: self.models.as_deref(),
            grants: self.grants.as_deref(),
        }
    }
}

/// The answer's text, given on the command line or in a file.
#[derive(Args)]
#[group(required = true, multiple = false)]
struct Text {
    /// The answer.
    #[arg(long)]
    text: Option<String>,
    /// A file holding the answer.
    #[arg(long)]
    text_file: Option<PathBuf>,
}

// @A command line that cannot be read refused with its usage,IMPL_MAIN_MISUSE,impl,[CREQ_COMMAND_MISUSE],[DEC_COMMAND_LINE_THROUGH_CLAP]
fn main() -> ExitCode {
    let line = match Line::try_parse() {
        Ok(line) => line,
        Err(error) => error.exit(),
    };
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("agconflo: no runtime to run on: {error}");
            return ExitCode::from(REFUSED);
        }
    };
    ExitCode::from(runtime.block_on(perform(line.command)))
}

const COMPLETED: u8 = 0;
const AWAITING: u8 = 3;
const REFUSED: u8 = 4;
const NODE_FAILED: u8 = 5;
const BUDGET_EXCEEDED: u8 = 6;
const STUCK: u8 = 7;
const NOT_KEPT: u8 = 8;

/// `command` asked of the runner, reported, and the status to exit with.
// @Each command asked of the runner with what was typed,IMPL_MAIN_COMMANDS,impl,[CREQ_COMMAND_READS_THE_COMMAND, CREQ_COMMAND_READS_GRANTS],[DEC_COMMAND_LINE_THROUGH_CLAP, DEC_RUNNER_ON_ONE_THREAD, DEC_GRANTS_IN_A_FILE_OF_THEIR_OWN]
async fn perform(command: Command) -> u8 {
    let stopped = match command {
        Command::Check {
            manifest,
            models,
            grants,
        } => {
            return checked(Sources {
                manifest: &manifest,
                models: models.as_deref(),
                grants: grants.as_deref(),
            });
        }
        Command::Run {
            files,
            arguments,
            argument_files,
        } => {
            let arguments = match given(&arguments, &argument_files) {
                Ok(arguments) => arguments,
                Err(message) => return refused(&message),
            };
            agconflo_runner::start(files.sources(), &files.record, &arguments).await
        }
        Command::Resume { files } => agconflo_runner::resume(files.sources(), &files.record).await,
        Command::Answer {
            files,
            instance,
            text,
            route,
        } => {
            let route = named(&route);
            let text = match (text.text, text.text_file) {
                (Some(text), _) => text,
                (None, file) => match read(&file.unwrap_or_default()) {
                    Ok(text) => text,
                    Err(message) => return refused(&message),
                },
            };
            agconflo_runner::answer(
                files.sources(),
                &files.record,
                &instance,
                &text,
                route.as_deref(),
            )
            .await
        }
    };
    match stopped {
        Ok(stopped) => report(&stopped),
        Err(refusal @ Refusal::NoGrants) => refused(&format!("{refusal}; name it with --grants")),
        Err(refusal) => refused(&refusal.to_string()),
    }
}

/// The route `--route` gave: none when it was not given, and the names given
/// otherwise, an empty one naming nothing so that `--route ""` is a route to
/// nowhere.
// @A person's route read from the command,IMPL_MAIN_ROUTE,impl,[CREQ_COMMAND_READS_ROUTE],[DEC_PERSON_ROUTE_IN_THE_ANSWER]
fn named(route: &[String]) -> Option<Vec<String>> {
    (!route.is_empty()).then(|| {
        route
            .iter()
            .filter(|name| !name.is_empty())
            .cloned()
            .collect()
    })
}

/// The arguments given as text and in files, in that order, or why a file
/// could not be read.
fn given(texts: &[String], files: &[String]) -> Result<Vec<Argument>, String> {
    let argument = |triple: &[String], text: String| Argument {
        instance: triple[0].clone(),
        parameter: triple[1].clone(),
        text,
    };
    let mut arguments: Vec<Argument> = texts
        .chunks(3)
        .map(|triple| argument(triple, triple[2].clone()))
        .collect();
    for triple in files.chunks(3) {
        arguments.push(argument(triple, read(Path::new(&triple[2]))?));
    }
    Ok(arguments)
}

/// The text of the file at `path`, exactly, or why it cannot be read.
fn read(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("{}: cannot be read: {error}", path.display()))?;
    String::from_utf8(bytes).map_err(|_| format!("{}: is not UTF-8 text", path.display()))
}

/// What a check found reported, and its status.
fn checked(sources: Sources<'_>) -> u8 {
    let findings = agconflo_runner::check(sources);
    for finding in &findings {
        match finding {
            Finding::NoGrants => eprintln!("agconflo: {finding}; name it with --grants"),
            _ => eprintln!("agconflo: {finding}"),
        }
    }
    if findings.is_empty() {
        eprintln!("agconflo: nothing found that would refuse a run");
        COMPLETED
    } else {
        REFUSED
    }
}

/// A refusal reported, and its status.
fn refused(message: &str) -> u8 {
    eprintln!("agconflo: refused: {message}");
    REFUSED
}

/// How a run stopped reported - its result or the step it awaits on standard
/// output, anything else on standard error - and the status for it. A tool's
/// step the container engine could not perform is reported as the step the
/// run awaits, with the engine's failure.
// @The result or the awaited step alone on standard output,IMPL_MAIN_REPORT,impl,[CREQ_COMMAND_RESULT_ON_STDOUT, CREQ_COMMAND_FAILURE_ON_STDERR, CREQ_COMMAND_UNKEPT_RECORD_TOLD, CREQ_COMMAND_TELLS_ENGINE_FAILURE],[DEC_RESULT_ON_STANDARD_OUTPUT, DEC_ENGINE_FAILURE_LEAVES_THE_STEP]
fn report(stopped: &Stopped) -> u8 {
    match &stopped.outcome {
        Outcome::Ended(RunEnding::Completed(result)) => print(&result.render()),
        Outcome::Awaiting(activation) => {
            print(&awaited(activation, stopped.routes.as_deref()));
            match &stopped.engine {
                Some(failure) => eprintln!(
                    "agconflo: the tool step for {} was not performed - {failure}; `agconflo resume` performs it again once the engine is back, or answer it with `agconflo answer`",
                    activation.instance()
                ),
                None => eprintln!(
                    "agconflo: the run awaits a person for {}; answer it with `agconflo answer`",
                    activation.instance()
                ),
            }
        }
        Outcome::Ended(RunEnding::NodeFailed { instance, failure }) => {
            eprintln!("agconflo: the node {instance} failed: {failure}");
        }
        Outcome::Ended(RunEnding::BudgetExceeded { budget }) => {
            eprintln!("agconflo: the run stopped at its budget of {budget} activations");
        }
        Outcome::Ended(RunEnding::Quiescent { waiting }) => {
            eprintln!(
                "agconflo: no node can make further progress; nothing came from {}",
                waiting.join(", ")
            );
        }
    }
    if let Some(unkept) = &stopped.unkept {
        eprintln!("agconflo: the record was not kept: {unkept}");
    }
    status(stopped)
}

/// The status to exit with for how a run stopped.
// @A status of its own for each way a run stops,IMPL_MAIN_EXIT_STATUS,impl,[CREQ_COMMAND_EXIT_STATUS],[DEC_EXIT_STATUS_PER_ENDING]
fn status(stopped: &Stopped) -> u8 {
    if stopped.unkept.is_some() {
        return NOT_KEPT;
    }
    match &stopped.outcome {
        Outcome::Ended(RunEnding::Completed(_)) => COMPLETED,
        Outcome::Awaiting(_) => AWAITING,
        Outcome::Ended(RunEnding::NodeFailed { .. }) => NODE_FAILED,
        Outcome::Ended(RunEnding::BudgetExceeded { .. }) => BUDGET_EXCEEDED,
        Outcome::Ended(RunEnding::Quiescent { .. }) => STUCK,
    }
}

/// The step a run awaits, as a person is shown it: the instance, the type of
/// context it produces, for a router's step the instances it may name, and
/// each input by parameter with its rendering.
// @A router's choices printed with its awaited step,IMPL_MAIN_ROUTES_SHOWN,impl,[CREQ_COMMAND_READS_ROUTE],[DEC_PERSON_ROUTE_IN_THE_ANSWER]
fn awaited(activation: &Activation, routes: Option<&[String]>) -> String {
    let mut shown = format!(
        "instance: {}\nproduces: {}\n",
        activation.instance(),
        activation.output().as_str()
    );
    if let Some(routes) = routes {
        shown.push_str(&format!("routes to: {}\n", routes.join(" ")));
    }
    for (parameter, context) in activation.inputs() {
        shown.push_str(&format!(
            "input {parameter} ({}):\n{}\n",
            context.declared_type().as_str(),
            context.render()
        ));
    }
    shown
}

/// `text` printed to standard output exactly.
fn print(text: &str) {
    let mut out = std::io::stdout().lock();
    if let Err(error) = out.write_all(text.as_bytes()).and_then(|()| out.flush()) {
        eprintln!("agconflo: cannot write to standard output: {error}");
    }
}
