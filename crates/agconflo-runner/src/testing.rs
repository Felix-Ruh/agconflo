//! What the tests of every module share: a directory of files of their own,
//! and a stub provider on the loopback interface.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

/// The image the tests' containers are made from, by its digest.
pub(crate) const IMAGE: &str =
    "alpine@sha256:294b683cb724975bec92580e1e685676bd4b50bda910ddb8c51d4cabeaec77e6";

/// A second image, whose sh is dash and whose timeout is GNU's, by its digest.
pub(crate) const PYTHON: &str =
    "python@sha256:cea0e6040540fb2b965b6e7fb5ffa00871e632eef63719f0ea54bca189ce14a6";

/// A directory for one test's files, emptied when it is made and removed when
/// it is dropped.
pub(crate) struct Scratch {
    root: PathBuf,
}

impl Scratch {
    /// A directory named after `test` and this process, in the system's
    /// temporary directory.
    pub(crate) fn new(test: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("agconflo-runner-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");
        Self { root }
    }

    /// The path of `file` in the directory.
    pub(crate) fn path(&self, file: &str) -> PathBuf {
        self.root.join(file)
    }

    /// The directory itself.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// `text` written to `file` in the directory, its directories made first.
    pub(crate) fn write(&self, file: &str, text: impl AsRef<[u8]>) -> PathBuf {
        let path = self.path(file);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("the file's directory");
        }
        std::fs::write(&path, text).expect("the file written");
        path
    }
}

/// Docker's engine reached and the test image present, or a panic naming
/// which is missing and how to mend it.
pub(crate) fn needs_docker() {
    let version = Command::new("docker")
        .args(["version", "--format", "{{.Server.Version}}"])
        .output();
    match version {
        Ok(version) if version.status.success() => {}
        Ok(version) => panic!(
            "this test needs Docker, and its engine was not reached: {}",
            String::from_utf8_lossy(&version.stderr).trim()
        ),
        Err(error) => panic!("this test needs Docker, and docker could not be run: {error}"),
    }
    needs_image(IMAGE);
}

/// `image` present, or a panic naming it and how to pull it.
pub(crate) fn needs_image(image: &str) {
    let inspected = docker(&["image", "inspect", "--format", "{{.Id}}", image]);
    assert!(
        inspected.status.success(),
        "this test needs the image {image}; pull it with: docker pull {image}"
    );
}

/// `docker` run with `args`, and what it gave back.
pub(crate) fn docker(args: &[&str]) -> std::process::Output {
    Command::new("docker")
        .args(args)
        .output()
        .expect("docker could be run")
}

/// Every container labelled with the record file `record`, by its full id.
pub(crate) fn labelled(record: &Path) -> Vec<String> {
    let filter = format!(
        "label={}={}",
        crate::sandbox::LABEL,
        crate::sandbox::label(record)
    );
    let listed = docker(&["ps", "-aq", "--no-trunc", "--filter", &filter]);
    String::from_utf8_lossy(&listed.stdout)
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

/// Grants of the test image written to `scratch`: each of `folders` made
/// there and granted by its name, writable or not, the network if `network`,
/// every action, and `limits`.
pub(crate) fn grants(
    scratch: &Scratch,
    folders: &[(&str, bool)],
    network: bool,
    limits: crate::grants::CommandLimits,
) -> crate::grants::Grants {
    let mut text = format!(
        "image = \"{IMAGE}\"\nimages = [\"{PYTHON}\"]\nnetwork = {network}\nactions = [\"read\", \"write\", \"run\"]\n\n[limits]\nseconds = {}\noutput = {}\ntmp = {}\n",
        limits.seconds, limits.output, limits.tmp
    );
    for (name, writable) in folders {
        std::fs::create_dir_all(scratch.path(name)).expect("the folder");
        text.push_str(&format!(
            "\n[folders.{name}]\npath = \"{name}\"\nwritable = {writable}\n"
        ));
    }
    crate::grants::read_grants(&scratch.write("grants.toml", text)).expect("the grants")
}

/// `command` with every variable named `*_API_KEY` removed from the
/// environment it will run in.
pub(crate) fn without_keys(command: &mut Command) -> &mut Command {
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().ends_with("_API_KEY") {
            command.env_remove(name);
        }
    }
    command
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// One request a stub was sent: its path, the model it names, and how long
/// the key in each key header was, or `None` for a header not sent. No key's
/// value is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Request {
    pub(crate) path: String,
    pub(crate) model: String,
    pub(crate) authorization: Option<usize>,
    pub(crate) api_key: Option<usize>,
}

/// A stub provider on the loopback interface, answering OpenAI's
/// chat-completions format, Anthropic's messages format and the decisions
/// endpoint's by the path each is sent to, and recording every request. To a
/// decision, a reply's text is the answers, as JSON text.
pub(crate) struct Stub {
    pub(crate) base: String,
    seen: Arc<Mutex<Vec<(Request, String)>>>,
}

/// A model's call in a stub's reply: its identifier, the node type it calls,
/// and its arguments as JSON text.
pub(crate) type Call = (String, String, String);

impl Stub {
    /// A stub answering every request with `answer`.
    pub(crate) fn answering(answer: &str) -> Self {
        Self::serving(Some(vec![(answer.to_owned(), Vec::new())]))
    }

    /// A stub holding every request open without a word: a provider a run can
    /// be interrupted while waiting on.
    pub(crate) fn holding() -> Self {
        Self::serving(None)
    }

    /// A stub answering its n-th request with the n-th of `replies` - a text
    /// and the calls it makes - and every request past the last with the last,
    /// in OpenAI's format.
    pub(crate) fn replying(replies: Vec<(String, Vec<Call>)>) -> Self {
        assert!(!replies.is_empty(), "a stub replies with something");
        Self::serving(Some(replies))
    }

    fn serving(replies: Option<Vec<(String, Vec<Call>)>>) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let base = format!("http://{}/v1/", listener.local_addr().expect("an address"));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let recorded = seen.clone();
        std::thread::spawn(move || {
            let mut held = Vec::new();
            for mut connection in listener.incoming().flatten() {
                let Some((head, body)) = read_request(&mut connection) else {
                    continue;
                };
                let request = request(&head, &body);
                let anthropic = request.path.ends_with("/messages");
                let decides = request.path.ends_with("/decisions");
                let index = {
                    let mut log = recorded.lock().expect("the log");
                    log.push((request, body));
                    log.len() - 1
                };
                let Some(replies) = &replies else {
                    held.push(connection);
                    continue;
                };
                let (answer, calls) = &replies[index.min(replies.len() - 1)];
                let reply = if decides {
                    format!("{{\"model\":\"stub\",\"answers\":{answer}}}")
                } else if anthropic {
                    serde_json::json!({
                        "id": "m", "type": "message", "role": "assistant", "model": "stub",
                        "content": [{"type": "text", "text": answer}], "stop_reason": "end_turn",
                        "usage": {"input_tokens": 1, "output_tokens": 1}
                    })
                    .to_string()
                } else {
                    let mut message = serde_json::json!({"role": "assistant", "content": answer});
                    if !calls.is_empty() {
                        message["tool_calls"] = calls
                            .iter()
                            .map(|(id, name, arguments)| {
                                serde_json::json!({"id": id, "type": "function",
                                                   "function": {"name": name, "arguments": arguments}})
                            })
                            .collect();
                    }
                    let finish = if calls.is_empty() {
                        "stop"
                    } else {
                        "tool_calls"
                    };
                    serde_json::json!({
                        "id": "c", "object": "chat.completion", "created": 0, "model": "stub",
                        "choices": [{"index": 0, "finish_reason": finish, "message": message}],
                        "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
                    })
                    .to_string()
                };
                let _ = write!(
                    connection,
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{reply}",
                    reply.len()
                );
            }
        });
        Self { base, seen }
    }

    /// Every request received so far.
    pub(crate) fn requests(&self) -> Vec<Request> {
        let log = self.seen.lock().expect("the log");
        log.iter().map(|(request, _)| request.clone()).collect()
    }

    /// The body of every request received so far.
    pub(crate) fn bodies(&self) -> Vec<String> {
        let log = self.seen.lock().expect("the log");
        log.iter().map(|(_, body)| body.clone()).collect()
    }
}

/// The request whose head is `head` and body `body`, with each key header's
/// value measured and dropped.
fn request(head: &str, body: &str) -> Request {
    let path = head
        .lines()
        .next()
        .and_then(|line| line.split(' ').nth(1))
        .unwrap_or("")
        .to_owned();
    let header = |name: &str| {
        head.lines().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case(name).then(|| value.trim())
        })
    };
    let authorization = header("authorization")
        .map(|value| value.strip_prefix("Bearer").unwrap_or(value).trim().len());
    let api_key = header("x-api-key").map(str::len);
    let model = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|body| body["model"].as_str().map(str::to_owned))
        .unwrap_or_default();
    Request {
        path,
        model,
        authorization,
        api_key,
    }
}

/// One HTTP request's head and body, or `None` if the connection closed first.
fn read_request(connection: &mut std::net::TcpStream) -> Option<(String, String)> {
    let mut received = Vec::new();
    let mut chunk = [0u8; 8192];
    let head_end = loop {
        let read = connection.read(&mut chunk).ok()?;
        if read == 0 {
            return None;
        }
        received.extend_from_slice(&chunk[..read]);
        if let Some(end) = received.windows(4).position(|w| w == b"\r\n\r\n") {
            break end;
        }
    };
    let head = String::from_utf8_lossy(&received[..head_end]).to_string();
    let length: usize = head
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().unwrap_or(0))
        })
        .unwrap_or(0);
    while received.len() < head_end + 4 + length {
        let read = connection.read(&mut chunk).ok()?;
        if read == 0 {
            return None;
        }
        received.extend_from_slice(&chunk[..read]);
    }
    let body = String::from_utf8_lossy(&received[head_end + 4..head_end + 4 + length]).to_string();
    Some((head, body))
}

/// One thing a stand-in for the sandbox was asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Asked {
    /// Whether the image is ready.
    Ready,
    /// A read of this path.
    Read(String),
    /// A write of this text to this path.
    Write(String, String),
    /// A run of this command.
    Run(String),
}

/// What a stand-in answers a step it is asked for with.
type Answer = Box<dyn FnMut(&Asked) -> Result<crate::Done, crate::EngineFailure>>;

/// A stand-in for the sandbox: it answers each step as it was told to, finds
/// the image ready unless told otherwise, and records everything it was
/// asked. Its clones share what they answer with and what they were asked.
#[derive(Clone)]
pub(crate) struct StandIn {
    inner: std::rc::Rc<std::cell::RefCell<Inner>>,
}

struct Inner {
    answer: Answer,
    ready: Result<(), crate::NotReady>,
    not_ready: Vec<(String, crate::NotReady)>,
    asked: Vec<Asked>,
    environments: Vec<(String, String)>,
    images: Vec<String>,
}

impl StandIn {
    /// A stand-in answering every step with `answer`.
    pub(crate) fn with(
        answer: impl FnMut(&Asked) -> Result<crate::Done, crate::EngineFailure> + 'static,
    ) -> Self {
        Self {
            inner: std::rc::Rc::new(std::cell::RefCell::new(Inner {
                answer: Box::new(answer),
                ready: Ok(()),
                not_ready: Vec::new(),
                asked: Vec::new(),
                environments: Vec::new(),
                images: Vec::new(),
            })),
        }
    }

    /// A stand-in answering every step with status 0 and `output`.
    pub(crate) fn answering(output: &str) -> Self {
        let output = output.to_owned();
        Self::with(move |_| {
            Ok(crate::Done {
                status: 0,
                output: output.clone(),
            })
        })
    }

    /// A stand-in answering every step with the engine's failure, `message`.
    pub(crate) fn failing(message: &str) -> Self {
        let message = message.to_owned();
        Self::with(move |_| {
            Err(crate::EngineFailure {
                message: message.clone(),
            })
        })
    }

    /// The same stand-in, finding every image not ready for `why`.
    pub(crate) fn not_ready(self, why: crate::NotReady) -> Self {
        self.inner.borrow_mut().ready = Err(why);
        self
    }

    /// The same stand-in, finding `image` not ready for `why`.
    pub(crate) fn not_ready_for(self, image: &str, why: crate::NotReady) -> Self {
        self.inner
            .borrow_mut()
            .not_ready
            .push((image.to_owned(), why));
        self
    }

    /// The container and the image of every step asked for, in order.
    pub(crate) fn environments(&self) -> Vec<(String, String)> {
        self.inner.borrow().environments.clone()
    }

    /// Every image asked about, in order.
    pub(crate) fn images(&self) -> Vec<String> {
        self.inner.borrow().images.clone()
    }

    /// Everything asked so far, in order.
    pub(crate) fn asked(&self) -> Vec<Asked> {
        self.inner.borrow().asked.clone()
    }

    fn step(
        &mut self,
        environment: crate::Environment<'_>,
        asked: Asked,
    ) -> Result<crate::Done, crate::EngineFailure> {
        let mut inner = self.inner.borrow_mut();
        inner.asked.push(asked.clone());
        inner.environments.push((
            environment.container.to_owned(),
            environment.image.to_owned(),
        ));
        (inner.answer)(&asked)
    }
}

impl crate::performer::Container for StandIn {
    fn ready(&self, image: &str) -> Result<(), crate::NotReady> {
        let mut inner = self.inner.borrow_mut();
        inner.asked.push(Asked::Ready);
        inner.images.push(image.to_owned());
        match inner.not_ready.iter().find(|(named, _)| named == image) {
            Some((_, why)) => Err(why.clone()),
            None => inner.ready.clone(),
        }
    }

    fn read(
        &mut self,
        environment: crate::Environment<'_>,
        path: &str,
    ) -> Result<crate::Done, crate::EngineFailure> {
        self.step(environment, Asked::Read(path.to_owned()))
    }

    fn write(
        &mut self,
        environment: crate::Environment<'_>,
        path: &str,
        text: &str,
    ) -> Result<crate::Done, crate::EngineFailure> {
        self.step(environment, Asked::Write(path.to_owned(), text.to_owned()))
    }

    fn run(
        &mut self,
        environment: crate::Environment<'_>,
        command: &str,
    ) -> Result<crate::Done, crate::EngineFailure> {
        self.step(environment, Asked::Run(command.to_owned()))
    }
}
