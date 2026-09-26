//! The sandbox: the container a run's tool steps are performed in, reached
//! through the `docker` command.
//!
//! One container is made at the first step it is asked for, from the grants'
//! image, locked down, with each granted folder mounted under `/work` by its
//! name, and labelled with the run's record file. Each step runs in it through
//! `docker exec` as an unprivileged user, and the container is removed when
//! the sandbox is dropped.

use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::grants::{CommandLimits, Folder, Grants};

/// The label every container a run makes carries, with the run's record file
/// as its value.
pub(crate) const LABEL: &str = "agconflo.record";

/// The script each step runs in: the command given as `$3`, with any further
/// arguments as its own, under `timeout` for `$1` seconds, its output and
/// errors to a file; then every process of the step's user killed, the output
/// written to standard output whole up to `$2` bytes or cut to its first and
/// last halves, and the command's status alone to standard error.
// @The command under timeout and its status alone on standard error,TRACE_SANDBOX_WRAPPER,trace,[],[DEC_COMMAND_WRAPPED, DEC_OUTPUT_KEEPS_BOTH_ENDS]
const WRAPPER: &str = r#"t=$1 l=$2 c=$3; shift 3
timeout -s KILL "$t" sh -c "$c" sh "$@" >/tmp/out 2>&1
s=$?
kill -KILL -1 2>/dev/null
n=$(wc -c </tmp/out)
if [ "$n" -le "$l" ]; then cat /tmp/out; else h=$((l / 2)); head -c "$h" /tmp/out; printf "\n[%d bytes cut]\n" $((n - l)); tail -c $((l - h)) /tmp/out; fi
rm -f /tmp/out
echo "$s" >&2"#;

/// A read step's command: the file named by its argument, to standard output.
const READ: &str = r#"cat -- "$1""#;

/// A write step's command: the folders the file named by its argument lies
/// in made, and the file replaced by standard input.
const WRITE: &str = r#"mkdir -p -- "$(dirname -- "$1")" && cat > "$1""#;

/// The container engine's own failure, told apart from any step's: its
/// message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineFailure {
    /// What the engine, or the `docker` command, said.
    pub message: String,
}

impl fmt::Display for EngineFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the container engine failed: {}", self.message)
    }
}

impl std::error::Error for EngineFailure {}

/// Why the grants' image cannot be used for a run's tool steps.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NotReady {
    /// The image is not present, and is not pulled.
    Absent {
        /// The image, as the grants name it.
        image: String,
    },
    /// The image holds no `sh`.
    NoShell {
        /// The image, as the grants name it.
        image: String,
    },
    /// The image holds no `timeout`.
    NoTimeout {
        /// The image, as the grants name it.
        image: String,
    },
    /// The engine could not be asked.
    Engine(EngineFailure),
}

impl fmt::Display for NotReady {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absent { image } => write!(
                f,
                "the image {image} is not present, and is never pulled; pull it with docker pull {image}"
            ),
            Self::NoShell { image } => {
                write!(f, "the image {image} holds no sh, which every step runs in")
            }
            Self::NoTimeout { image } => write!(
                f,
                "the image {image} holds no timeout, which every step's command runs under"
            ),
            Self::Engine(failure) => failure.fmt(f),
        }
    }
}

impl std::error::Error for NotReady {}

/// What a step's command came to: its exit status and its output, standard
/// output and errors together, cut to the grants' limit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Done {
    /// The command's exit status: 137 for one ended by its time limit.
    pub status: i32,
    /// What it printed, a character broken by a cut replaced.
    pub output: String,
}

/// The container a run's tool steps are performed in, made at the first step
/// and removed when this is dropped.
#[derive(Debug)]
pub struct Sandbox {
    image: String,
    folders: Vec<Folder>,
    network: bool,
    limits: CommandLimits,
    label: String,
    user: (u32, u32),
    env: Vec<(String, String)>,
    container: Option<String>,
}

impl Sandbox {
    /// A sandbox for the run whose record file is `record`, as `grants`
    /// grant it. Nothing is made until a step is asked for.
    pub fn new(grants: &Grants, record: &Path) -> Self {
        Self {
            image: grants.image().to_owned(),
            folders: grants.folders().to_vec(),
            network: grants.network(),
            limits: grants.limits(),
            label: label(record),
            user: step_ids(),
            env: Vec::new(),
            container: None,
        }
    }

    /// The same sandbox, running `docker` with `name` set to `value`.
    #[cfg(test)]
    pub(crate) fn with_env(mut self, name: &str, value: &str) -> Self {
        self.env.push((name.to_owned(), value.to_owned()));
        self
    }

    /// The container made for the run, if one has been.
    #[cfg(test)]
    pub(crate) fn container(&self) -> Option<&str> {
        self.container.as_deref()
    }

    /// Nothing, or why the grants' image cannot be used: absent, holding no
    /// `sh` or no `timeout`, or the engine not reached. The image is never
    /// pulled, and nothing is made but one short container of the image's own,
    /// removed as it ends.
    // @An image found present with sh and timeout and never pulled,IMPL_SANDBOX_READY,impl,[CREQ_SANDBOX_IMAGE_READY],[DEC_IMAGE_BY_DIGEST_NEVER_PULLED]
    pub fn ready(&self) -> Result<(), NotReady> {
        let image = || self.image.clone();
        let inspected = self
            .docker(
                &["image", "inspect", "--format", "{{.Id}}", &self.image],
                b"",
            )
            .map_err(NotReady::Engine)?;
        if !inspected.status.success() {
            let version = self
                .docker(&["version", "--format", "{{.Server.Version}}"], b"")
                .map_err(NotReady::Engine)?;
            return Err(if version.status.success() {
                NotReady::Absent { image: image() }
            } else {
                NotReady::Engine(failure(&version))
            });
        }

        let probed = self
            .docker(
                &[
                    "run",
                    "--rm",
                    "--pull",
                    "never",
                    "--network",
                    "none",
                    "--cap-drop",
                    "ALL",
                    "--security-opt",
                    "no-new-privileges",
                    "--entrypoint",
                    "sh",
                    &self.image,
                    "-c",
                    "command -v timeout >/dev/null && echo ready || echo no-timeout",
                ],
                b"",
            )
            .map_err(NotReady::Engine)?;
        match String::from_utf8_lossy(&probed.stdout).trim() {
            "ready" if probed.status.success() => Ok(()),
            "no-timeout" if probed.status.success() => Err(NotReady::NoTimeout { image: image() }),
            "" if matches!(probed.status.code(), Some(126 | 127)) => {
                Err(NotReady::NoShell { image: image() })
            }
            _ => Err(NotReady::Engine(failure(&probed))),
        }
    }

    /// The file at `path`, relative to `/work`: its text as the command's
    /// output, or what `cat` said and its status.
    pub fn read(&mut self, path: &str) -> Result<Done, EngineFailure> {
        self.step(READ, Some(path), b"")
    }

    /// The file at `path`, relative to `/work`, replaced by `text` exactly,
    /// with the folders it lies in made first.
    pub fn write(&mut self, path: &str, text: &str) -> Result<Done, EngineFailure> {
        self.step(WRITE, Some(path), text.as_bytes())
    }

    /// `command` run by `sh` in `/work`.
    pub fn run(&mut self, command: &str) -> Result<Done, EngineFailure> {
        self.step(command, None, b"")
    }

    /// `script` run in the container, with `argument` as its `$1` and `input`
    /// on its standard input: its status and output, or the engine's failure.
    ///
    /// The status is the last line the wrapper writes to standard error; any
    /// line before it, which the wrapper writes only when the command removed
    /// what it writes the output to, is added to the output. A step whose
    /// wrapper ended without a status is the step's own doing while its
    /// container still runs: whatever the step's user left is killed, and the
    /// step comes to `docker exec`'s status and what was printed. Otherwise it
    /// is the engine's failure.
    // @A path as an argument and text as input to the wrapped command,IMPL_SANDBOX_STEP,impl,[CREQ_SANDBOX_PATHS_AS_ARGUMENTS, CREQ_SANDBOX_STEP_USER, CREQ_SANDBOX_TIME_LIMIT, CREQ_SANDBOX_NOTHING_LEFT_RUNNING, CREQ_SANDBOX_OUTPUT_LIMIT, CREQ_SANDBOX_ENGINE_FAILURE_APART],[DEC_PATHS_AS_ARGUMENTS, DEC_COMMAND_WRAPPED, DEC_STEP_USER_IS_THE_PERSONS, DEC_ENGINE_THROUGH_ITS_COMMAND, DEC_KILLED_WRAPPER_TOLD_BY_ITS_CONTAINER]
    fn step(
        &mut self,
        script: &str,
        argument: Option<&str>,
        input: &[u8],
    ) -> Result<Done, EngineFailure> {
        let container = self.made()?;
        let user = format!("{}:{}", self.user.0, self.user.1);
        let seconds = self.limits.seconds.to_string();
        let output = self.limits.output.to_string();
        let mut args = vec![
            "exec", "-i", "-u", &user, "-w", "/work", &container, "sh", "-c", WRAPPER, "wrapper",
            &seconds, &output, script,
        ];
        args.extend(argument);
        let done = self.docker(&args, input)?;

        let mut output = String::from_utf8_lossy(&done.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&done.stderr).into_owned();
        let mut lines: Vec<&str> = stderr.lines().collect();
        let status = lines.pop().and_then(|status| status.parse().ok());
        match status {
            Some(status) if done.status.success() => {
                for line in lines {
                    output.push_str(line);
                    output.push('\n');
                }
                Ok(Done { status, output })
            }
            _ if done.status.code().is_some_and(|code| code > 128) && self.running(&container) => {
                let _ = self.docker(
                    &["exec", "-u", &user, &container, "sh", "-c", "kill -KILL -1"],
                    b"",
                );
                output.push_str(&stderr);
                Ok(Done {
                    status: done.status.code().unwrap_or_default(),
                    output,
                })
            }
            _ => Err(failure(&done)),
        }
    }

    /// Whether `container` is there and running.
    fn running(&self, container: &str) -> bool {
        self.docker(
            &["inspect", "--format", "{{.State.Running}}", container],
            b"",
        )
        .is_ok_and(|inspected| {
            inspected.status.success()
                && String::from_utf8_lossy(&inspected.stdout).trim() == "true"
        })
    }

    /// The run's container, made first if it has not been, with every
    /// container a killed run left under the same label removed before it.
    // @Leftovers removed before the run's container is made locked down,IMPL_SANDBOX_MAKE,impl,[CREQ_SANDBOX_LOCKED_DOWN, CREQ_SANDBOX_REMOVES_LEFTOVERS],[DEC_CONTAINER_LOCKED_DOWN, DEC_GRANTS_NARROW_BY_DEFAULT, DEC_LEFTOVER_CONTAINERS_REMOVED, DEC_ONE_CONTAINER_PER_CALL, DEC_PATHS_IN_GRANTED_FOLDERS]
    fn made(&mut self) -> Result<String, EngineFailure> {
        if let Some(container) = &self.container {
            return Ok(container.clone());
        }

        let filter = format!("label={LABEL}={}", self.label);
        let listed = self.docker(&["ps", "-aq", "--no-trunc", "--filter", &filter], b"")?;
        if !listed.status.success() {
            return Err(failure(&listed));
        }
        let left = String::from_utf8_lossy(&listed.stdout).into_owned();
        let left: Vec<&str> = left.split_whitespace().collect();
        if !left.is_empty() {
            let mut args = vec!["rm", "-f"];
            args.extend(&left);
            let removed = self.docker(&args, b"")?;
            if !removed.status.success() {
                return Err(failure(&removed));
            }
        }

        let label = format!("{LABEL}={}", self.label);
        let network = if self.network { "bridge" } else { "none" };
        let mounts: Vec<String> = self.folders.iter().map(mount).collect();
        let mut args = vec![
            "run",
            "-d",
            "--pull",
            "never",
            "--init",
            "--network",
            network,
            "--read-only",
            "--tmpfs",
            "/tmp:mode=1777",
            "--cap-drop",
            "ALL",
            "--security-opt",
            "no-new-privileges",
            "--label",
            &label,
            "--workdir",
            "/work",
        ];
        for mount in &mounts {
            args.extend(["--mount", mount]);
        }
        args.extend([self.image.as_str(), "sleep", "infinity"]);
        let made = self.docker(&args, b"")?;
        if !made.status.success() {
            return Err(failure(&made));
        }
        let container = String::from_utf8_lossy(&made.stdout).trim().to_owned();
        self.container = Some(container.clone());
        Ok(container)
    }

    /// `docker` run with `args` and `input` on its standard input, with this
    /// sandbox's variables set, and what it gave back - or, when it could not
    /// be run at all, that as the engine's failure.
    fn docker(&self, args: &[&str], input: &[u8]) -> Result<Output, EngineFailure> {
        let mut child = Command::new("docker")
            .args(args)
            .envs(self.env.iter().map(|(name, value)| (name, value)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| EngineFailure {
                message: format!("docker could not be run: {error}"),
            })?;
        let mut stdin = child.stdin.take().expect("a piped standard input");
        let input = input.to_vec();
        let writer = std::thread::spawn(move || {
            let _ = stdin.write_all(&input);
        });
        let output = child.wait_with_output().map_err(|error| EngineFailure {
            message: format!("docker could not be waited for: {error}"),
        })?;
        let _ = writer.join();
        Ok(output)
    }
}

impl Drop for Sandbox {
    /// The run's container removed, if one was made. A failure to remove it
    /// is left for the next call of the same run, which removes it first.
    // @The container removed however its call ends,IMPL_SANDBOX_REMOVE,impl,[CREQ_SANDBOX_REMOVED_AT_THE_END],[DEC_ONE_CONTAINER_PER_CALL]
    fn drop(&mut self) {
        if let Some(container) = self.container.take() {
            let _ = self.docker(&["rm", "-f", &container], b"");
        }
    }
}

/// The label value for the run whose record file is `record`: its path made
/// absolute.
pub(crate) fn label(record: &Path) -> String {
    std::path::absolute(record)
        .unwrap_or_else(|_| record.to_owned())
        .display()
        .to_string()
}

/// `folder` as `docker run`'s `--mount` value: its absolute path bound at
/// `/work` under its name, read-only unless it is writable, with each field
/// quoted and a comma in it kept.
fn mount(folder: &Folder) -> String {
    let source: PathBuf = std::path::absolute(&folder.path).unwrap_or_else(|_| folder.path.clone());
    let quoted = |field: String| format!("\"{}\"", field.replace('"', "\"\""));
    let mut mount = format!(
        "type=bind,{},{}",
        quoted(format!("source={}", source.display())),
        quoted(format!("target=/work/{}", folder.name))
    );
    if !folder.writable {
        mount.push_str(",readonly");
    }
    mount
}

/// `output`'s failure: what `docker` wrote to standard error, or its status
/// when it wrote nothing.
fn failure(output: &Output) -> EngineFailure {
    let said = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    EngineFailure {
        message: if said.is_empty() {
            format!("docker exited with {}", output.status)
        } else {
            said
        },
    }
}

/// The user and group a step runs as: on Linux the effective ones of this
/// process, unless that is root or cannot be read, and elsewhere 1000 and
/// 1000.
// @The person's user on Linux and never root,IMPL_SANDBOX_STEP_USER,impl,[CREQ_SANDBOX_STEP_USER],[DEC_STEP_USER_IS_THE_PERSONS, DEC_CONTAINER_LOCKED_DOWN]
fn step_ids() -> (u32, u32) {
    let own = if cfg!(target_os = "linux") {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|status| ids_from_status(&status))
    } else {
        None
    };
    match own {
        Some((user, group)) if user != 0 => (user, group),
        _ => (1000, 1000),
    }
}

/// The effective user and group a process's status file, as Linux writes
/// `/proc/<pid>/status`, gives, or `None` when it lacks either line.
fn ids_from_status(status: &str) -> Option<(u32, u32)> {
    let effective = |key: &str| {
        status
            .lines()
            .find_map(|line| line.strip_prefix(key))
            .and_then(|ids| ids.split_whitespace().nth(1))
            .and_then(|id| id.parse().ok())
    };
    Some((effective("Uid:")?, effective("Gid:")?))
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases.

#[cfg(test)]
use crate::testing::{IMAGE, Scratch, docker, grants, labelled, needs_docker};
#[cfg(test)]
use std::time::{Duration, Instant};

/// A sandbox for a run whose record file is in `scratch`, granted each of
/// `folders` by name, writable or not, with `limits`.
#[cfg(test)]
fn sandbox(
    scratch: &Scratch,
    folders: &[(&str, bool)],
    network: bool,
    limits: CommandLimits,
) -> Sandbox {
    needs_docker();
    Sandbox::new(
        &grants(scratch, folders, network, limits),
        &scratch.path("run.toml"),
    )
}

/// The step `done` came to, or a panic naming the engine's failure.
#[cfg(test)]
fn done(done: Result<Done, EngineFailure>) -> Done {
    done.unwrap_or_else(|failure| panic!("expected the step performed, got {failure}"))
}

#[cfg(test)]
#[test]
fn paths_and_text_not_shell() {
    let scratch = Scratch::new("sandbox_paths_and_text_not_shell");
    let mut sandbox = sandbox(&scratch, &[("w", true)], false, CommandLimits::default());

    let name = "a'; touch pwned; '";
    let written = done(sandbox.write(&format!("w/{name}"), "quoted"));
    assert_eq!(written.status, 0, "{}", written.output);
    assert_eq!(
        std::fs::read_to_string(scratch.path("w").join(name)).expect("the file"),
        "quoted"
    );
    assert!(!scratch.path("w/pwned").exists());
    assert!(!scratch.path("pwned").exists());

    // Past the 131072 bytes one argument may hold on Linux.
    let long: String = (0..300_000u32)
        .map(|i| char::from(b'a' + (i % 26) as u8))
        .collect();
    let written = done(sandbox.write("w/long.txt", &long));
    assert_eq!(written.status, 0, "{}", written.output);
    assert_eq!(
        std::fs::read_to_string(scratch.path("w/long.txt")).expect("the file"),
        long
    );
}

#[cfg(test)]
#[test]
fn confined_to_grants() {
    let scratch = Scratch::new("sandbox_confined_to_grants");
    scratch.write("kept/k.txt", "kept");
    scratch.write("open/o.txt", "open");
    scratch.write("beside/b.txt", "beside");
    let limits = CommandLimits::default();
    let mut sandbox = sandbox(&scratch, &[("kept", false), ("open", true)], false, limits);

    let removed = done(sandbox.run("rm -rf /work/kept/*"));
    assert_ne!(removed.status, 0, "{}", removed.output);
    assert_eq!(
        std::fs::read_to_string(scratch.path("kept/k.txt")).expect("kept"),
        "kept"
    );

    done(sandbox.run("rm -rf /"));
    assert_eq!(
        std::fs::read_to_string(scratch.path("kept/k.txt")).expect("kept"),
        "kept"
    );
    assert_eq!(
        std::fs::read_to_string(scratch.path("beside/b.txt")).expect("beside"),
        "beside"
    );
    assert!(!scratch.path("open/o.txt").exists());

    let capabilities = done(sandbox.run("grep CapEff /proc/self/status"));
    assert_eq!(
        capabilities.output.split_whitespace().nth(1),
        Some("0000000000000000"),
        "{}",
        capabilities.output
    );

    let offline = done(sandbox.run("ls /sys/class/net"));
    assert_eq!(
        offline.output.split_whitespace().collect::<Vec<_>>(),
        ["lo"]
    );
    drop(sandbox);
    let scratch = Scratch::new("sandbox_confined_to_grants_online");
    let mut online = self::sandbox(&scratch, &[], true, limits);
    let networked = done(online.run("ls /sys/class/net"));
    let interfaces: Vec<_> = networked.output.split_whitespace().collect();
    assert!(
        interfaces.contains(&"lo") && interfaces.len() > 1,
        "{interfaces:?}"
    );
}

#[cfg(test)]
#[test]
fn own_process_survives() {
    let scratch = Scratch::new("sandbox_own_process_survives");
    let mut sandbox = sandbox(&scratch, &[], false, CommandLimits::default());

    done(sandbox.run("kill -KILL -1; kill -KILL 1"));
    let alive = done(sandbox.run("echo alive"));

    assert_eq!((alive.status, alive.output.as_str()), (0, "alive\n"));
}

#[cfg(test)]
#[test]
fn step_user() {
    let scratch = Scratch::new("sandbox_step_user");
    let mut sandbox = sandbox(&scratch, &[("w", true)], false, CommandLimits::default());

    let ids = done(sandbox.run("id -u; id -g; touch w/mine"));
    let ids: Vec<u32> = ids
        .output
        .split_whitespace()
        .map(|id| id.parse().expect("an id"))
        .collect();

    let expected = if cfg!(target_os = "linux") {
        let status = std::fs::read_to_string("/proc/self/status").expect("the status file");
        let (user, group) = ids_from_status(&status).expect("the ids");
        [user, group]
    } else {
        [1000, 1000]
    };
    assert_eq!(ids, expected);
    assert!(!ids.contains(&0));
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let owner = std::fs::metadata(scratch.path("w/mine")).expect("the file");
        assert_eq!([owner.uid(), owner.gid()], expected);
    }
}

#[cfg(test)]
#[test]
fn user_from_status() {
    let status = "Name:\tsh\nUmask:\t0022\nState:\tR (running)\nTgid:\t7\nNgid:\t0\nPid:\t7\nUid:\t1001\t1002\t1003\t1004\nGid:\t2001\t2002\t2003\t2004\nNSpgid:\t7\n";
    assert_eq!(ids_from_status(status), Some((1002, 2002)));

    let without_gid = status.replace("Gid:", "Xid:");
    assert_eq!(ids_from_status(&without_gid), None);
    let without_uid = status.replace("Uid:", "Xid:");
    assert_eq!(ids_from_status(&without_uid), None);
}

#[cfg(test)]
#[test]
fn leftovers_removed() {
    let scratch = Scratch::new("sandbox_leftovers_removed");
    let mut sandbox = sandbox(&scratch, &[], false, CommandLimits::default());
    let ours = format!("{LABEL}={}", label(&scratch.path("run.toml")));
    let theirs = format!("{LABEL}={}", label(&scratch.path("other.toml")));
    let running = |label: &str| {
        let started = docker(&[
            "run", "-d", "--pull", "never", "--label", label, IMAGE, "sleep", "60",
        ]);
        String::from_utf8_lossy(&started.stdout).trim().to_owned()
    };
    let our_running = running(&ours);
    let created = docker(&[
        "create", "--pull", "never", "--label", &ours, IMAGE, "sleep", "60",
    ]);
    let our_stopped = String::from_utf8_lossy(&created.stdout).trim().to_owned();
    let their_running = running(&theirs);
    assert_eq!(labelled(&scratch.path("run.toml")).len(), 2);

    done(sandbox.run("true"));

    let ours_left = labelled(&scratch.path("run.toml"));
    let theirs_left = labelled(&scratch.path("other.toml"));
    docker(&["rm", "-f", &their_running]);
    assert_eq!(
        ours_left,
        [sandbox.container().expect("the run's container").to_owned()]
    );
    assert!(!ours_left.contains(&our_running) && !ours_left.contains(&our_stopped));
    assert_eq!(theirs_left, [their_running]);
}

#[cfg(test)]
#[test]
fn time_limit() {
    let scratch = Scratch::new("sandbox_time_limit");
    let limits = CommandLimits {
        seconds: 2,
        ..CommandLimits::default()
    };
    let mut sandbox = sandbox(&scratch, &[], false, limits);
    done(sandbox.run("true"));

    let started = Instant::now();
    let ended = done(sandbox.run("echo started; sleep 30; echo never"));

    assert!(started.elapsed() < Duration::from_secs(10));
    assert_eq!(ended.status, 137);
    assert!(ended.output.contains("started"), "{}", ended.output);
    assert!(!ended.output.contains("never"), "{}", ended.output);
    let listed = done(sandbox.run("ps -o args"));
    assert!(!listed.output.contains("sleep 30"), "{}", listed.output);
}

#[cfg(test)]
#[test]
fn nothing_left_running() {
    let scratch = Scratch::new("sandbox_nothing_left_running");
    let mut sandbox = sandbox(&scratch, &[], false, CommandLimits::default());

    let left = done(sandbox.run("sleep 40 & (trap '' TERM HUP; sh -c 'sleep 50 &'); echo ok"));
    assert_eq!(left.output, "ok\n");

    // Every process but the container's own is the listing step's: its
    // wrapper, its timeout and ps.
    let listed = done(sandbox.run("ps -o user,stat,args"));
    let processes: Vec<&str> = listed.output.lines().skip(1).collect();
    let root = processes
        .iter()
        .filter(|line| line.split_whitespace().next() == Some("root"))
        .count();
    assert_eq!(root, 2, "{}", listed.output);
    assert!(
        !processes
            .iter()
            .any(|line| line.split_whitespace().next() != Some("root") && line.contains("sleep")),
        "{}",
        listed.output
    );
    assert!(
        !processes.iter().any(|line| line
            .split_whitespace()
            .nth(1)
            .is_some_and(|s| s.contains('Z'))),
        "{}",
        listed.output
    );

    let after = done(sandbox.run("echo still"));
    assert_eq!((after.status, after.output.as_str()), (0, "still\n"));
}

#[cfg(test)]
#[test]
fn output_keeps_both_ends() {
    let scratch = Scratch::new("sandbox_output_keeps_both_ends");
    let limits = CommandLimits {
        output: 100,
        ..CommandLimits::default()
    };
    let mut sandbox = sandbox(&scratch, &[("w", true)], false, limits);
    let digits =
        |n: usize| -> String { (0..n).map(|i| char::from(b'0' + (i % 10) as u8)).collect() };

    for length in [99, 100] {
        let text = digits(length);
        done(sandbox.write("w/out.txt", &text));
        assert_eq!(done(sandbox.run("cat w/out.txt")).output, text);
    }
    for (length, cut) in [(101, 1), (1000, 900)] {
        let text = digits(length);
        done(sandbox.write("w/out.txt", &text));
        assert_eq!(
            done(sandbox.run("cat w/out.txt")).output,
            format!(
                "{}\n[{cut} bytes cut]\n{}",
                &text[..50],
                &text[length - 50..]
            )
        );
    }

    // A two-byte character across each cut: bytes 49 and 50, and 949 and 950.
    let text = format!(
        "{}\u{e9}{}\u{e9}{}",
        "a".repeat(49),
        "b".repeat(898),
        "c".repeat(49)
    );
    assert_eq!(text.len(), 1000);
    done(sandbox.write("w/out.txt", &text));
    assert_eq!(
        done(sandbox.run("cat w/out.txt")).output,
        format!(
            "{}\u{fffd}\n[900 bytes cut]\n\u{fffd}{}",
            "a".repeat(49),
            "c".repeat(49)
        )
    );
}

#[cfg(test)]
#[test]
fn engine_failure_apart() {
    let scratch = Scratch::new("sandbox_engine_failure_apart");
    let unreachable = sandbox(&scratch, &[], false, CommandLimits::default())
        .with_env("DOCKER_HOST", "tcp://127.0.0.1:1");
    let mut unreachable = unreachable;
    match unreachable.run("true") {
        Err(EngineFailure { message }) => assert!(!message.is_empty()),
        Ok(done) => panic!("expected the engine's failure, got {done:?}"),
    }
    assert!(unreachable.container().is_none());

    let mut sandbox = sandbox(&scratch, &[], false, CommandLimits::default());
    let one = done(sandbox.run("echo one; exit 1"));
    assert_eq!((one.status, one.output.as_str()), (1, "one\n"));
    let own = done(sandbox.run("echo own; exit 125"));
    assert_eq!((own.status, own.output.as_str()), (125, "own\n"));

    let container = sandbox.container().expect("the container").to_owned();
    docker(&["rm", "-f", &container]);
    match sandbox.run("true") {
        Err(EngineFailure { message }) => assert!(!message.is_empty()),
        Ok(done) => panic!("expected the engine's failure, got {done:?}"),
    }

    // Removed while a step runs, which docker exec reports as it does a
    // step that killed its own wrapper.
    let scratch = Scratch::new("sandbox_engine_failure_apart_during");
    let mut sandbox = self::sandbox(&scratch, &[], false, CommandLimits::default());
    done(sandbox.run("true"));
    let container = sandbox.container().expect("the container").to_owned();
    let remover = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1500));
        docker(&["rm", "-f", &container]);
    });
    let during = sandbox.run("sleep 8; echo never");
    remover.join().expect("the container removed");
    match during {
        Err(EngineFailure { .. }) => {}
        Ok(done) => panic!("expected the engine's failure, got {done:?}"),
    }
}

#[cfg(test)]
#[test]
fn removed_at_the_end() {
    let scratch = Scratch::new("sandbox_removed_at_the_end");
    let record = scratch.path("run.toml");

    {
        let mut sandbox = sandbox(&scratch, &[], false, CommandLimits::default());
        done(sandbox.run("true"));
        assert_eq!(labelled(&record).len(), 1);
    }
    assert_eq!(labelled(&record), Vec::<String>::new());

    let returned_early = || -> Result<(), EngineFailure> {
        let mut sandbox = sandbox(&scratch, &[], false, CommandLimits::default());
        sandbox.run("true")?;
        assert_eq!(labelled(&record).len(), 1);
        Err(EngineFailure {
            message: "early".to_owned(),
        })?;
        sandbox.run("true").map(drop)
    };
    assert!(returned_early().is_err());
    assert_eq!(labelled(&record), Vec::<String>::new());

    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut sandbox = sandbox(&scratch, &[], false, CommandLimits::default());
        done(sandbox.run("true"));
        assert_eq!(labelled(&record).len(), 1);
        panic!("the owner panics");
    }));
    assert!(panicked.is_err());
    assert_eq!(labelled(&record), Vec::<String>::new());
}

#[cfg(test)]
#[test]
fn image_not_ready() {
    let scratch = Scratch::new("sandbox_image_not_ready");
    needs_docker();
    let with_image = |image: &str| {
        let file = scratch.write("grants.toml", format!("image = \"{image}\"\n"));
        Sandbox::new(
            &crate::grants::read_grants(&file).expect("the grants"),
            &scratch.path("run.toml"),
        )
    };

    let absent = format!("alpine@sha256:{}", "0".repeat(64));
    assert_eq!(
        with_image(&absent).ready(),
        Err(NotReady::Absent {
            image: absent.clone()
        })
    );
    let listed = docker(&[
        "image",
        "ls",
        "-q",
        "--no-trunc",
        "--filter",
        "reference=alpine",
    ]);
    assert!(!String::from_utf8_lossy(&listed.stdout).contains(&"0".repeat(64)));

    // An image of nothing, imported from an empty archive.
    let mut import = Command::new("docker")
        .args(["import", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("docker import");
    import
        .stdin
        .take()
        .expect("its input")
        .write_all(&[0u8; 1024])
        .expect("an empty archive");
    let empty = import.wait_with_output().expect("the import");
    let empty = String::from_utf8_lossy(&empty.stdout).trim().to_owned();

    // The pinned image with timeout removed.
    let made = docker(&[
        "create",
        "--pull",
        "never",
        "--network",
        "none",
        "--entrypoint",
        "rm",
        IMAGE,
        "/usr/bin/timeout",
    ]);
    let container = String::from_utf8_lossy(&made.stdout).trim().to_owned();
    docker(&["start", "-a", &container]);
    let committed = docker(&["commit", &container]);
    docker(&["rm", "-f", &container]);
    let untimed = String::from_utf8_lossy(&committed.stdout).trim().to_owned();

    let no_shell = with_image(&empty).ready();
    let no_timeout = with_image(&untimed).ready();
    docker(&["image", "rm", "-f", &empty, &untimed]);

    assert_eq!(no_shell, Err(NotReady::NoShell { image: empty }));
    assert_eq!(no_timeout, Err(NotReady::NoTimeout { image: untimed }));
    assert_eq!(with_image(IMAGE).ready(), Ok(()));
}
