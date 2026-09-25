//! The `agconflo` binary, run as a person runs it: in a directory of its own,
//! with a stub provider on the loopback interface for any model, and with no
//! provider's key variable in its environment.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const BINARY: &str = env!("CARGO_BIN_EXE_agconflo");

/// Every node type the tests' workflows use.
const TYPES: &str = "[types.begin]\nrequired = { brief = \"note\" }\noptional = { extra = \"note\" }\noutput = \"note\"\n\n[types.ask]\nrequired = { before = \"note\" }\noutput = \"note\"\n\n[types.count]\nrequired = { before = \"note\" }\noutput = \"note\"\n\n[types.review]\nrequired = { before = \"note\" }\noutput = \"note\"\n\n[types.join]\nrequired = { before = \"note\", start = \"note\" }\noutput = \"note\"\n\n[types.add]\nrequired = { before = \"note\" }\noutput = \"note\"\n\n[types.broken]\nrequired = { before = \"note\" }\noutput = \"note\"\n";

/// The scripts, by file: `begin` joins its brief and extra with `|`, `ask` and
/// `count` put what came before to the roles `helping` and `counting`, `join`
/// joins the start and what came before with `/`, `add` appends `+c`.
const SCRIPTS: [(&str, &str); 6] = [
    (
        "begin.lua",
        "local given, host = ...\nreturn host.compose(host.output, {given.brief, given.extra}, '|')\n",
    ),
    (
        "ask.lua",
        "local given, host = ...\nreturn host.text(host.output, host.complete('helping', given.before):render())\n",
    ),
    (
        "count.lua",
        "local given, host = ...\nreturn host.text(host.output, host.complete('counting', given.before):render())\n",
    ),
    (
        "join.lua",
        "local given, host = ...\nreturn host.compose(host.output, {given.start, given.before}, '/')\n",
    ),
    (
        "add.lua",
        "local given, host = ...\nreturn host.text(host.output, given.before:render() .. '+c')\n",
    ),
    ("broken.lua", "error('broke here')\n"),
];

/// A directory for one test's files, removed when it is dropped.
struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(test: &str) -> Self {
        let root = std::env::temp_dir().join(format!("agconflo-cli-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");
        Self { root }
    }

    fn path(&self, file: &str) -> PathBuf {
        self.root.join(file)
    }

    fn write(&self, file: &str, text: impl AsRef<[u8]>) -> PathBuf {
        let path = self.path(file);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("the file's directory");
        }
        std::fs::write(&path, text).expect("the file written");
        path
    }

    /// A project whose workflow document is `flow`, under a budget of `budget`,
    /// with a person performing `review`.
    fn project(&self, flow: &str, budget: usize) {
        self.write("types.toml", TYPES);
        for (file, text) in SCRIPTS {
            self.write(file, text);
        }
        self.write("flow.toml", flow);
        self.write(
            "manifest.toml",
            format!(
                "workflow = \"flow.toml\"\ntypes = [\"types.toml\"]\nbudget = {budget}\npersons = [\"review\"]\n\n[scripts]\nbegin = \"begin.lua\"\nask = \"ask.lua\"\ncount = \"count.lua\"\njoin = \"join.lua\"\nadd = \"add.lua\"\nbroken = \"broken.lua\"\n"
            ),
        );
    }

    /// A model mapping sending `helping` to `helping` and `counting` to
    /// `counting`, written as `file`.
    fn models(&self, file: &str, helping: &Stub, counting: &Stub) {
        self.write(
            file,
            format!(
                "[roles]\nhelping = {{ model = \"openai::helper\", endpoint = \"{}\" }}\ncounting = {{ model = \"openai::counter\", endpoint = \"{}\" }}\n",
                helping.base, counting.base
            ),
        );
    }

    /// The binary, run in this directory with `args`.
    fn agconflo(&self, args: &[&str]) -> Command {
        let mut command = Command::new(BINARY);
        command.current_dir(&self.root).args(args);
        for (name, _) in std::env::vars_os() {
            if name.to_string_lossy().ends_with("_API_KEY") {
                command.env_remove(name);
            }
        }
        command
    }

    /// The binary run to its end with `args`: its status, standard output and
    /// standard error.
    fn ran(&self, args: &[&str]) -> (i32, String, String) {
        let output = self.agconflo(args).output().expect("the binary ran");
        (
            output.status.code().expect("an exit status"),
            String::from_utf8(output.stdout).expect("text on standard output"),
            String::from_utf8(output.stderr).expect("text on standard error"),
        )
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// A chain: `first` begins, `second` is of `middle`, `third` adds.
fn chain(middle: &str) -> String {
    format!(
        "name = \"chain\"\noutput = \"third\"\n\n[instances.first]\nnode_type = \"begin\"\nentry = true\n\n[instances.second]\nnode_type = \"{middle}\"\nbindings = {{ before = \"first\" }}\n\n[instances.third]\nnode_type = \"add\"\nbindings = {{ before = \"second\" }}\n"
    )
}

/// `first` begins, a person reviews it as `second`, and `third` joins the two.
const PERSON: &str = "name = \"reviewed\"\noutput = \"third\"\n\n[instances.first]\nnode_type = \"begin\"\nentry = true\n\n[instances.second]\nnode_type = \"review\"\nbindings = { before = \"first\" }\n\n[instances.third]\nnode_type = \"join\"\nbindings = { before = \"second\", start = \"first\" }\n";

/// `first` begins, `second` asks the counting role, `third` the helping one.
const TWO_MODELS: &str = "name = \"two\"\noutput = \"third\"\n\n[instances.first]\nnode_type = \"begin\"\nentry = true\n\n[instances.second]\nnode_type = \"count\"\nbindings = { before = \"first\" }\n\n[instances.third]\nnode_type = \"ask\"\nbindings = { before = \"second\" }\n";

/// Two instances each waiting on the other.
const CYCLE: &str = "name = \"cycle\"\noutput = \"c1\"\n\n[instances.c1]\nnode_type = \"add\"\nbindings = { before = \"c2\" }\n\n[instances.c2]\nnode_type = \"add\"\nbindings = { before = \"c1\" }\n";

/// A stub provider in OpenAI's chat-completions format, answering every
/// request with one text or holding every one open, and counting them.
struct Stub {
    base: String,
    requests: Arc<Mutex<usize>>,
}

impl Stub {
    fn answering(answer: &str) -> Self {
        Self::serving(Some(answer.to_owned()))
    }

    fn holding() -> Self {
        Self::serving(None)
    }

    fn serving(answer: Option<String>) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let base = format!("http://{}/v1/", listener.local_addr().expect("an address"));
        let requests = Arc::new(Mutex::new(0));
        let counted = requests.clone();
        std::thread::spawn(move || {
            let mut held = Vec::new();
            for mut connection in listener.incoming().flatten() {
                if !read_request(&mut connection) {
                    continue;
                }
                *counted.lock().expect("the count") += 1;
                let Some(answer) = &answer else {
                    held.push(connection);
                    continue;
                };
                let reply = serde_json::json!({
                    "id": "c", "object": "chat.completion", "created": 0, "model": "stub",
                    "choices": [{"index": 0, "finish_reason": "stop",
                                 "message": {"role": "assistant", "content": answer}}],
                    "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
                })
                .to_string();
                let _ = write!(
                    connection,
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{reply}",
                    reply.len()
                );
            }
        });
        Self { base, requests }
    }

    fn requests(&self) -> usize {
        *self.requests.lock().expect("the count")
    }

    /// Waits until the stub has been sent `count` requests, for up to ten
    /// seconds.
    fn until(&self, count: usize) {
        let started = Instant::now();
        while self.requests() < count {
            assert!(
                started.elapsed() < Duration::from_secs(10),
                "the stub was never asked"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

/// Whether one whole HTTP request was read from `connection`.
fn read_request(connection: &mut std::net::TcpStream) -> bool {
    let mut received = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let Ok(read) = connection.read(&mut chunk) else {
            return false;
        };
        if read == 0 {
            return false;
        }
        received.extend_from_slice(&chunk[..read]);
        let Some(end) = received.windows(4).position(|w| w == b"\r\n\r\n") else {
            continue;
        };
        let head = String::from_utf8_lossy(&received[..end]).to_ascii_lowercase();
        let length: usize = head
            .lines()
            .find_map(|line| line.strip_prefix("content-length:"))
            .and_then(|value| value.trim().parse().ok())
            .unwrap_or(0);
        if received.len() >= end + 4 + length {
            return true;
        }
    }
}

/// A child process killed when it is dropped.
struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn commands_reach_the_runner() {
    let scratch = Scratch::new("commands_reach_the_runner");
    scratch.project(PERSON, 5);
    scratch.write("extra.txt", "X Y\r\n");
    scratch.write("answer.txt", "ans wer\r\n");

    let (status, out, _) = scratch.ran(&["check", "manifest.toml"]);
    assert_eq!((status, out.as_str()), (0, ""));

    let (status, out, err) = scratch.ran(&[
        "run",
        "manifest.toml",
        "--record",
        "run.toml",
        "--arg",
        "first",
        "brief",
        "a b",
        "--arg-file",
        "first",
        "extra",
        "extra.txt",
    ]);
    assert_eq!(status, 3, "{err}");
    assert!(out.contains("instance: second"), "{out}");

    let answer = [
        "answer",
        "manifest.toml",
        "--record",
        "run.toml",
        "--instance",
        "second",
        "--text-file",
        "answer.txt",
    ];
    let (status, out, err) = scratch.ran(&answer);
    assert_eq!(
        (status, out.as_str()),
        (0, "a b|X Y\r\n/ans wer\r\n"),
        "{err}"
    );

    let (status, out, err) = scratch.ran(&["resume", "manifest.toml", "--record", "run.toml"]);
    assert_eq!(
        (status, out.as_str()),
        (0, "a b|X Y\r\n/ans wer\r\n"),
        "{err}"
    );
}

#[test]
fn resumed_after_a_kill() {
    let scratch = Scratch::new("resumed_after_a_kill");
    scratch.project(TWO_MODELS, 5);
    let counting = Stub::answering("counted");
    let holding = Stub::holding();
    scratch.models("holding.toml", &holding, &counting);

    let running = Running(
        scratch
            .agconflo(&[
                "run",
                "manifest.toml",
                "--record",
                "run.toml",
                "--models",
                "holding.toml",
                "--arg",
                "first",
                "brief",
                "x",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the run started"),
    );
    holding.until(1);
    drop(running);

    let answering = Stub::answering("helped");
    scratch.models("answering.toml", &answering, &counting);
    let (status, out, err) = scratch.ran(&[
        "resume",
        "manifest.toml",
        "--record",
        "run.toml",
        "--models",
        "answering.toml",
    ]);
    assert_eq!((status, out.as_str()), (0, "helped"), "{err}");
    assert_eq!(counting.requests(), 1);
}

#[test]
fn held_record_refused() {
    let scratch = Scratch::new("held_record_refused");
    scratch.project(&chain("ask"), 5);
    let holding = Stub::holding();
    scratch.models("holding.toml", &holding, &holding);

    let _running = Running(
        scratch
            .agconflo(&[
                "run",
                "manifest.toml",
                "--record",
                "run.toml",
                "--models",
                "holding.toml",
                "--arg",
                "first",
                "brief",
                "x",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the run started"),
    );
    holding.until(1);
    let before = std::fs::read(scratch.path("run.toml")).expect("the record");

    let (status, out, err) = scratch.ran(&[
        "resume",
        "manifest.toml",
        "--record",
        "run.toml",
        "--models",
        "holding.toml",
    ]);

    assert_eq!((status, out.as_str()), (4, ""));
    assert!(err.contains("run.toml.lock"), "{err}");
    assert_eq!(
        std::fs::read(scratch.path("run.toml")).expect("the record"),
        before
    );
}

#[test]
fn answered_in_a_new_process() {
    let scratch = Scratch::new("answered_in_a_new_process");
    scratch.project(PERSON, 5);

    let (status, out, err) = scratch.ran(&[
        "run",
        "manifest.toml",
        "--record",
        "run.toml",
        "--arg",
        "first",
        "brief",
        "x",
    ]);
    assert_eq!(status, 3, "{err}");
    assert!(out.contains("instance: second"), "{out}");

    let (status, out, err) = scratch.ran(&[
        "answer",
        "manifest.toml",
        "--record",
        "run.toml",
        "--instance",
        "second",
        "--text",
        "ok",
    ]);
    assert_eq!((status, out.as_str()), (0, "x/ok"), "{err}");
}

#[test]
fn exit_status_per_ending() {
    let scratch = Scratch::new("exit_status_per_ending");
    let run = |flow: &str, budget: usize, record: &str| {
        scratch.project(flow, budget);
        scratch
            .ran(&[
                "run",
                "manifest.toml",
                "--record",
                record,
                "--arg",
                "first",
                "brief",
                "x",
            ])
            .0
    };

    let completed = run(&chain("add"), 5, "completed.toml");
    let awaiting = run(PERSON, 5, "awaiting.toml");
    let failed = run(&chain("broken"), 5, "failed.toml");
    let budget = run(&chain("add"), 1, "budget.toml");
    scratch.project(CYCLE, 5);
    let stuck = scratch
        .ran(&["run", "manifest.toml", "--record", "stuck.toml"])
        .0;
    scratch.project(&chain("add"), 5);
    let refused = scratch
        .ran(&["run", "absent.toml", "--record", "refused.toml"])
        .0;
    std::fs::create_dir_all(scratch.path("unkept.toml.new/inside")).expect("the block");
    let unkept = run(&chain("add"), 5, "unkept.toml");
    let misuse = scratch.ran(&["run", "manifest.toml"]).0;

    let statuses = [
        completed, misuse, awaiting, refused, failed, budget, stuck, unkept,
    ];
    assert_eq!(statuses, [0, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn result_on_standard_output() {
    let scratch = Scratch::new("result_on_standard_output");
    scratch.project(&chain("add"), 5);
    let holding = Stub::holding();
    scratch.models("models.toml", &holding, &holding);

    let (status, out, _) = scratch.ran(&[
        "run",
        "manifest.toml",
        "--record",
        "completed.toml",
        "--models",
        "models.toml",
        "--arg",
        "first",
        "brief",
        "x",
    ]);
    assert_eq!((status, out.as_str()), (0, "x+c+c"));

    scratch.project(PERSON, 5);
    let (status, out, _) = scratch.ran(&[
        "run",
        "manifest.toml",
        "--record",
        "awaiting.toml",
        "--arg",
        "first",
        "brief",
        "x",
    ]);
    assert_eq!(
        (status, out.as_str()),
        (
            3,
            "instance: second\nproduces: note\ninput before (note):\nx\n"
        )
    );
}

#[test]
fn failure_on_standard_error() {
    let scratch = Scratch::new("failure_on_standard_error");
    scratch.project(&chain("broken"), 5);

    let (status, out, err) = scratch.ran(&[
        "run",
        "manifest.toml",
        "--record",
        "failed.toml",
        "--arg",
        "first",
        "brief",
        "x",
    ]);
    assert_eq!((status, out.as_str()), (5, ""));
    assert!(
        err.contains("second") && err.contains("broke here"),
        "{err}"
    );

    scratch.write(
        "manifest.toml",
        "workflow = \"flow.toml\"\nbudget = 2\n    = 1\n",
    );
    let (status, out, err) = scratch.ran(&["run", "manifest.toml", "--record", "refused.toml"]);
    assert_eq!((status, out.as_str()), (4, ""));
    assert!(err.contains("manifest.toml:3:5"), "{err}");
}

#[test]
fn unkept_record_told() {
    let scratch = Scratch::new("unkept_record_told");
    scratch.project(&chain("add"), 5);
    std::fs::create_dir_all(scratch.path("run.toml.new/inside")).expect("the block");

    let (status, out, err) = scratch.ran(&[
        "run",
        "manifest.toml",
        "--record",
        "run.toml",
        "--arg",
        "first",
        "brief",
        "x",
    ]);

    assert_eq!((status, out.as_str()), (8, "x+c+c"));
    assert!(err.contains("holds none"), "{err}");
}

#[test]
fn misuse_exits_2() {
    let scratch = Scratch::new("misuse_exits_2");
    for args in [
        &[][..],
        &["frobnicate"][..],
        &["run", "manifest.toml"][..],
        &[
            "answer",
            "manifest.toml",
            "--record",
            "run.toml",
            "--text",
            "t",
        ][..],
        &[
            "run",
            "manifest.toml",
            "--record",
            "run.toml",
            "--arg",
            "first",
        ][..],
        &["check", "manifest.toml", "--bogus"][..],
    ] {
        let (status, out, err) = scratch.ran(args);
        assert_eq!((status, out.as_str()), (2, ""), "{args:?}: {err}");
        assert!(err.contains("Usage"), "{args:?}: {err}");
    }
    let left = std::fs::read_dir(&scratch.root)
        .expect("the directory")
        .count();
    assert_eq!(left, 0);
}
