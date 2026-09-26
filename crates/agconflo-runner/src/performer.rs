//! The tool performer: one tool step performed in a container, and the text
//! the step is answered with.
//!
//! `read` takes `path` and answers with the file's text; `write` takes `path`
//! and `text`, replaces the file with the text, and answers with a line naming
//! the file and its bytes; `run` takes `command` and answers with a line giving
//! its exit status, then its output. A path is a granted folder's name and a
//! place inside it, and one that is absolute or names a parent folder is
//! refused. Whatever goes wrong with the action is the answer's text; only the
//! container engine's own failure is not.

use agconflo_core::Context;

use crate::grants::Action;
use crate::sandbox::{Done, EngineFailure, Environment, NotReady, Sandbox};

/// What a tool step's action is performed in: the sandbox, or a stand-in for
/// it.
pub trait Container {
    /// Nothing, or why `image` cannot be used.
    fn ready(&self, image: &str) -> Result<(), NotReady>;
    /// The file at `path`, read in `environment`.
    fn read(&mut self, environment: Environment<'_>, path: &str) -> Result<Done, EngineFailure>;
    /// The file at `path`, replaced by `text` in `environment`.
    fn write(
        &mut self,
        environment: Environment<'_>,
        path: &str,
        text: &str,
    ) -> Result<Done, EngineFailure>;
    /// `command`, run in `environment`.
    fn run(&mut self, environment: Environment<'_>, command: &str) -> Result<Done, EngineFailure>;
}

// @The sandbox as what a step is performed in,TRACE_PERFORMER_SANDBOX,trace,[],[DEC_TESTS_NEED_DOCKER]
impl Container for Sandbox {
    fn ready(&self, image: &str) -> Result<(), NotReady> {
        Sandbox::ready_image(self, image)
    }

    fn read(&mut self, environment: Environment<'_>, path: &str) -> Result<Done, EngineFailure> {
        Sandbox::read_in(self, environment, path)
    }

    fn write(
        &mut self,
        environment: Environment<'_>,
        path: &str,
        text: &str,
    ) -> Result<Done, EngineFailure> {
        Sandbox::write_in(self, environment, path, text)
    }

    fn run(&mut self, environment: Environment<'_>, command: &str) -> Result<Done, EngineFailure> {
        Sandbox::run_in(self, environment, command)
    }
}

/// The parameters `action` takes, in the order its step's inputs are read.
pub fn parameters(action: Action) -> &'static [&'static str] {
    match action {
        Action::Read => &["path"],
        Action::Write => &["path", "text"],
        Action::Run => &["command"],
    }
}

/// The text a step of `action`, given `inputs` by parameter, is answered with,
/// performed in `container` in `environment` - or the container engine's
/// failure, with no text.
///
/// A step lacking an input its action takes, given a path or a command
/// holding a NUL character, or given a path that is absolute or names a
/// parent folder, asks `container` for nothing.
// @A tool step's action performed and its answer made,IMPL_PERFORMER_PERFORM,impl,[CREQ_PERFORMER_PERFORMS_THE_ACTION, CREQ_PERFORMER_FAILURE_AS_TEXT, CREQ_PERFORMER_PASSES_ENGINE_FAILURE],[DEC_THREE_TOOL_ACTIONS, DEC_TOOL_FAILURE_IS_OUTPUT, DEC_ENGINE_FAILURE_LEAVES_THE_STEP]
pub fn perform(
    action: Action,
    environment: Environment<'_>,
    inputs: &[(String, Context)],
    container: &mut impl Container,
) -> Result<String, EngineFailure> {
    let mut given = Vec::new();
    for parameter in parameters(action) {
        match inputs.iter().find(|(name, _)| name == parameter) {
            Some((_, context)) => given.push(context.render().into_owned()),
            None => {
                return Ok(format!(
                    "the {action} step was not given its {parameter}, so nothing was done"
                ));
            }
        }
    }

    if given[0].contains('\0') {
        return Ok(format!(
            "the {} {:?} holds a NUL character, which no path or command can, so nothing was done",
            parameters(action)[0],
            given[0]
        ));
    }
    if matches!(action, Action::Read | Action::Write) && refused(&given[0]) {
        return Ok(format!(
            "the path {} was refused: a path is a granted folder's name and a place inside it, and may not be absolute or name a parent folder",
            given[0]
        ));
    }

    Ok(match action {
        Action::Read => {
            let Done { status, output } = container.read(environment, &given[0])?;
            if status == 0 {
                output
            } else {
                format!("the read of {} failed: {output}", given[0])
            }
        }
        Action::Write => {
            let Done { status, output } = container.write(environment, &given[0], &given[1])?;
            if status == 0 {
                format!("wrote {} bytes to {}", given[1].len(), given[0])
            } else {
                format!("the write of {} failed: {output}", given[0])
            }
        }
        Action::Run => {
            let Done { status, output } = container.run(environment, &given[0])?;
            format!("exit status {status}\n{output}")
        }
    })
}

/// Whether `path` is absolute - from a root, a drive, or either separator -
/// or has a part, split at either separator, that names a parent folder.
// @An absolute path or one naming a parent folder refused,IMPL_PERFORMER_PATH,impl,[CREQ_PERFORMER_REFUSES_PATH],[DEC_PATHS_IN_GRANTED_FOLDERS]
fn refused(path: &str) -> bool {
    let bytes = path.as_bytes();
    let rooted = path.starts_with(['/', '\\']);
    let drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    rooted || drive || path.split(['/', '\\']).any(|part| part == "..")
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases.

#[cfg(test)]
use crate::grants::CommandLimits;
#[cfg(test)]
use crate::testing::{Asked, IMAGE, Scratch, StandIn, grants, needs_docker};

/// The shared container, of the tests' image.
#[cfg(test)]
const HERE: Environment<'static> = Environment {
    container: crate::sandbox::SHARED,
    image: IMAGE,
};
#[cfg(test)]
use agconflo_core::{ContextType, IdSource};
#[cfg(test)]
use proptest::prelude::*;

/// Inputs by parameter, each a note of its text, in the order given.
#[cfg(test)]
fn inputs(given: &[(&str, &str)]) -> Vec<(String, Context)> {
    let mut source = IdSource::new();
    given
        .iter()
        .map(|(parameter, text)| {
            let note = ContextType::new("note").expect("a type");
            let context = Context::text(&mut source, note, *text).expect("a context");
            ((*parameter).to_owned(), context)
        })
        .collect()
}

/// A sandbox for a run whose record file is in `scratch`, granted each of
/// `folders` by name, writable or not.
#[cfg(test)]
fn sandbox(scratch: &Scratch, folders: &[(&str, bool)]) -> Sandbox {
    needs_docker();
    Sandbox::new(
        &grants(scratch, folders, false, CommandLimits::default()),
        &scratch.path("run.toml"),
    )
}

#[cfg(test)]
#[test]
fn actions_performed() {
    let scratch = Scratch::new("performer_actions_performed");
    let mut sandbox = sandbox(&scratch, &[("work", true)]);

    // Declared text first, path second: taken by name, not by position.
    let written = perform(
        Action::Write,
        HERE,
        &inputs(&[("text", "one\ntwo\n"), ("path", "work/new/dir/a.txt")]),
        &mut sandbox,
    );
    assert_eq!(
        written.as_deref(),
        Ok("wrote 8 bytes to work/new/dir/a.txt")
    );
    assert_eq!(
        std::fs::read_to_string(scratch.path("work/new/dir/a.txt")).expect("the file"),
        "one\ntwo\n"
    );

    let read = perform(
        Action::Read,
        HERE,
        &inputs(&[("path", "work/new/dir/a.txt")]),
        &mut sandbox,
    );
    assert_eq!(read.as_deref(), Ok("one\ntwo\n"));

    let ran = perform(
        Action::Run,
        HERE,
        &inputs(&[("command", "ls work/new/dir; exit 3")]),
        &mut sandbox,
    );
    assert_eq!(ran.as_deref(), Ok("exit status 3\na.txt\n"));
}

/// Text of ASCII and wider characters, quotes, dollar signs and backticks,
/// with every kind of line ending, ending in one or not.
#[cfg(test)]
fn any_text() -> impl Strategy<Value = String> {
    let piece = prop_oneof![
        Just("\n".to_owned()),
        Just("\r\n".to_owned()),
        Just("\r".to_owned()),
        Just("\u{e9}".to_owned()),
        Just("\u{1f600}".to_owned()),
        Just("'\"$`\\".to_owned()),
        "[ -~]{0,8}",
    ];
    prop::collection::vec(piece, 0..16).prop_map(|pieces| pieces.concat())
}

#[cfg(test)]
#[test]
fn text_kept_exactly() {
    let scratch = Scratch::new("performer_text_kept_exactly");
    let sandbox = std::cell::RefCell::new(sandbox(&scratch, &[("work", true)]));
    let mut runner =
        proptest::test_runner::TestRunner::new(proptest::test_runner::Config::with_cases(8));

    let kept = runner.run(&any_text(), |text| {
        let mut sandbox = sandbox.borrow_mut();
        perform(
            Action::Write,
            HERE,
            &inputs(&[("path", "work/t.txt"), ("text", &text)]),
            &mut *sandbox,
        )
        .expect("the write performed");
        prop_assert_eq!(
            std::fs::read(scratch.path("work/t.txt")).expect("the file"),
            text.as_bytes()
        );
        let read = perform(
            Action::Read,
            HERE,
            &inputs(&[("path", "work/t.txt")]),
            &mut *sandbox,
        )
        .expect("the read performed");
        prop_assert_eq!(read, text);
        Ok(())
    });

    if let Err(failure) = kept {
        panic!("{failure}");
    }
}

#[cfg(test)]
#[test]
fn path_refused() {
    for action in [Action::Read, Action::Write] {
        for path in [
            "/etc/passwd",
            "\\etc",
            "C:\\x",
            "c:x",
            "work/../../etc",
            "work/..",
            "..\\x",
        ] {
            let mut stand_in = StandIn::answering("x");
            let answer = perform(
                action,
                HERE,
                &inputs(&[("path", path), ("text", "t")]),
                &mut stand_in,
            )
            .expect("an answer");
            assert!(
                answer.starts_with(&format!("the path {path} was refused")),
                "{answer}"
            );
            assert_eq!(stand_in.asked(), []);
        }

        for path in ["work/a..b", "work/.hidden"] {
            let mut stand_in = StandIn::answering("x");
            perform(
                action,
                HERE,
                &inputs(&[("path", path), ("text", "t")]),
                &mut stand_in,
            )
            .expect("an answer");
            let expected = match action {
                Action::Read => Asked::Read(path.to_owned()),
                _ => Asked::Write(path.to_owned(), "t".to_owned()),
            };
            assert_eq!(stand_in.asked(), [expected]);
        }
    }
}

#[cfg(test)]
#[test]
fn failure_answered_as_text() {
    let scratch = Scratch::new("performer_failure_answered_as_text");
    let mut sandbox = sandbox(&scratch, &[("kept", false)]);

    let read = perform(
        Action::Read,
        HERE,
        &inputs(&[("path", "kept/nowhere.txt")]),
        &mut sandbox,
    )
    .expect("an answer");
    assert!(
        read.starts_with("the read of kept/nowhere.txt failed: ") && read.contains("nowhere.txt"),
        "{read}"
    );
    assert!(
        read.len() > "the read of kept/nowhere.txt failed: ".len(),
        "{read}"
    );

    let write = perform(
        Action::Write,
        HERE,
        &inputs(&[("path", "kept/a.txt"), ("text", "t")]),
        &mut sandbox,
    )
    .expect("an answer");
    assert!(
        write.starts_with("the write of kept/a.txt failed: ") && write.contains("Read-only"),
        "{write}"
    );

    let mut stand_in = StandIn::answering("x");
    let unbound = perform(Action::Read, HERE, &inputs(&[]), &mut stand_in).expect("an answer");
    assert_eq!(
        unbound,
        "the read step was not given its path, so nothing was done"
    );
    for (action, parameter) in [(Action::Read, "path"), (Action::Run, "command")] {
        let nul = perform(
            action,
            HERE,
            &inputs(&[(parameter, "a\u{0}b")]),
            &mut stand_in,
        )
        .expect("an answer");
        assert!(nul.contains("holds a NUL character"), "{nul}");
    }
    assert_eq!(stand_in.asked(), []);
}

#[cfg(test)]
#[test]
fn engine_failure_passed() {
    for (action, given) in [
        (Action::Run, [("command", "true")]),
        (Action::Read, [("path", "work/a.txt")]),
    ] {
        let mut stand_in = StandIn::failing("the engine is gone");
        assert_eq!(
            perform(action, HERE, &inputs(&given), &mut stand_in),
            Err(EngineFailure {
                message: "the engine is gone".to_owned()
            })
        );
        assert_eq!(stand_in.asked().len(), 1);
    }
}
