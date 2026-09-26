//! What the tests of every module share: a directory of files of their own,
//! and a stub provider on the loopback interface.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

/// The image the tests' containers are made from, by its digest.
pub(crate) const IMAGE: &str =
    "alpine@sha256:294b683cb724975bec92580e1e685676bd4b50bda910ddb8c51d4cabeaec77e6";

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
    let image = docker(&["image", "inspect", "--format", "{{.Id}}", IMAGE]);
    assert!(
        image.status.success(),
        "this test needs the image {IMAGE}; pull it with: docker pull {IMAGE}"
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
        "image = \"{IMAGE}\"\nnetwork = {network}\nactions = [\"read\", \"write\", \"run\"]\n\n[limits]\nseconds = {}\noutput = {}\n",
        limits.seconds, limits.output
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
/// chat-completions format and Anthropic's messages format by the path each is
/// sent to, and recording every request.
pub(crate) struct Stub {
    pub(crate) base: String,
    seen: Arc<Mutex<Vec<Request>>>,
}

impl Stub {
    /// A stub answering every request with `answer`.
    pub(crate) fn answering(answer: &str) -> Self {
        Self::serving(Some(answer.to_owned()))
    }

    /// A stub holding every request open without a word: a provider a run can
    /// be interrupted while waiting on.
    pub(crate) fn holding() -> Self {
        Self::serving(None)
    }

    fn serving(answer: Option<String>) -> Self {
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
                recorded.lock().expect("the log").push(request);
                let Some(answer) = &answer else {
                    held.push(connection);
                    continue;
                };
                let reply = if anthropic {
                    serde_json::json!({
                        "id": "m", "type": "message", "role": "assistant", "model": "stub",
                        "content": [{"type": "text", "text": answer}], "stop_reason": "end_turn",
                        "usage": {"input_tokens": 1, "output_tokens": 1}
                    })
                } else {
                    serde_json::json!({
                        "id": "c", "object": "chat.completion", "created": 0, "model": "stub",
                        "choices": [{"index": 0, "finish_reason": "stop",
                                     "message": {"role": "assistant", "content": answer}}],
                        "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
                    })
                }
                .to_string();
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
        self.seen.lock().expect("the log").clone()
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
