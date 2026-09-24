//! The model roster: the caller's mapping from role to model, and the client
//! that reaches them.
//!
//! A script names a role - what the call is for - and the caller starting the
//! run says which model plays it (`DEC_MODELS_BY_ROLE`), so the same workflow
//! reaches another provider by changing one mapping (`STKH_PROVIDER_CHOICE`).
//! Nothing here knows about scripts or activations.

use std::collections::HashMap;
use std::fmt;

use agconflo_core::Context;
use genai::Client;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest};

/// Which model plays each role, and the client that reaches them.
///
/// The client is the caller's own: where each model's requests go and what
/// credentials they carry are decided when it is built, from the caller's
/// environment and never from this repository. Models are named as `genai`
/// names them - `openai::gpt-...`, `claude-...` - and reached through it
/// (`DEC_MODELS_THROUGH_GENAI`).
#[derive(Clone, Debug)]
pub struct Roster {
    client: Client,
    roles: HashMap<String, String>,
}

impl Roster {
    /// A roster reaching models through `client`, with no role mapped yet.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            roles: HashMap::new(),
        }
    }

    /// The same roster, with `role` played by `model`. Mapping a role again
    /// replaces its model: the roster is the caller's choice for this run, and
    /// the last word is the choice.
    pub fn map(mut self, role: &str, model: &str) -> Self {
        self.roles.insert(role.to_owned(), model.to_owned());
        self
    }

    /// Ask the model `role` is mapped to about `prompt`, and return its answer.
    ///
    /// The prompt goes as one user message holding its rendering and nothing
    /// else - no system prompt, no instructions, nothing trimmed
    /// (`DEC_PROMPT_IS_A_CONTEXT`); measured, `genai` adds parameters and no text
    /// (`EVD_GENAI_TWO_FORMATS_EXACT`). The answer is the text the provider sent
    /// (`DEC_ANSWER_AS_SENT`), which excludes any reasoning the model reports
    /// separately, and is empty when the response holds none.
    ///
    /// A role mapped to nothing fails before anything is sent: sending the role
    /// as a model name would reach whatever provider the name resembles.
    // @One role one model one message,IMPL_MODELS_CALL,impl,[CREQ_ROSTER_ROLE_TO_MODEL, CREQ_ROSTER_ONE_MESSAGE, CREQ_ROSTER_UNMAPPED_ROLE, CREQ_ROSTER_PROVIDER_FAILURE, CREQ_ROSTER_ANSWER_AS_SENT]
    pub(crate) async fn call(&self, role: &str, prompt: &Context) -> Result<String, ModelFailure> {
        let Some(model) = self.roles.get(role) else {
            return Err(ModelFailure::Unmapped {
                role: role.to_owned(),
            });
        };
        let request =
            ChatRequest::default().append_message(ChatMessage::user(prompt.render().into_owned()));
        let options = ChatOptions::default().with_capture_raw_body(true);
        match self.client.exec_chat(model, request, Some(&options)).await {
            Ok(response) => Ok(response
                .captured_raw_body
                .as_ref()
                .and_then(answer_as_sent)
                .unwrap_or_else(|| response.into_first_text().unwrap_or_default())),
            Err(error) => Err(ModelFailure::Provider {
                role: role.to_owned(),
                status: error.status().map(|status| status.as_u16()),
                message: error.to_string(),
            }),
        }
    }
}

/// The answer's text as the provider sent it, read from the response body, for
/// the two formats measured; `None` for any other.
///
/// Read here rather than taken from `genai`, because `genai`'s OpenAI adapter
/// trims the answer and its Anthropic adapter does not
/// (`EVD_GENAI_OPENAI_TRIMS`): the same workflow would get different bytes from
/// two providers saying the same thing. OpenAI's format holds the answer as one
/// string; Anthropic's as blocks, whose text ones are joined in order. A
/// response in any other format falls back to `genai`'s reading of it, which is
/// the limit of what was measured.
// @The answer as the provider sent it,IMPL_MODELS_ANSWER_AS_SENT,impl,[CREQ_ROSTER_ANSWER_AS_SENT]
fn answer_as_sent(raw: &serde_json::Value) -> Option<String> {
    if let Some(content) = raw["choices"][0]["message"]["content"].as_str() {
        return Some(content.to_owned());
    }
    let blocks = raw["content"].as_array()?;
    let texts: Vec<&str> = blocks
        .iter()
        .filter(|block| block["type"] == "text")
        .filter_map(|block| block["text"].as_str())
        .collect();
    (!texts.is_empty()).then(|| texts.concat())
}

/// Why a model call failed.
///
/// Values rather than a message, because a caller acts differently on each: an
/// unmapped role is its own mapping to fix, a 401 is a key, a 503 is a provider
/// to wait for (`CREQ_ROSTER_PROVIDER_FAILURE`). `genai` holds the status as a
/// value already (`EVD_GENAI_ERROR_STATUS`).
///
/// `#[non_exhaustive]`: the ways a call can fail grow with what a call can be.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModelFailure {
    /// The role is mapped to no model, and nothing was sent
    /// (`CREQ_ROSTER_UNMAPPED_ROLE`).
    Unmapped {
        /// The role the script called.
        role: String,
    },
    /// The provider failed the call, or could not be reached.
    Provider {
        /// The role the script called.
        role: String,
        /// The HTTP status the provider answered with; none when there was no
        /// answer at all.
        status: Option<u16>,
        /// The failure as `genai` reports it.
        message: String,
    },
}

impl fmt::Display for ModelFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unmapped { role } => write!(f, "the role {role} is mapped to no model"),
            Self::Provider {
                role,
                status: Some(status),
                message,
            } => write!(f, "the model for {role} answered {status}: {message}"),
            Self::Provider {
                role,
                status: None,
                message,
            } => write!(f, "the model for {role} could not be reached: {message}"),
        }
    }
}

impl std::error::Error for ModelFailure {}

// --- test builders -----------------------------------------------------------
// A stub provider, and a client pointed at it. Used by every module's model
// tests, so they live beside the roster they exercise.

#[cfg(test)]
use std::io::{Read, Write};
#[cfg(test)]
use std::sync::{Arc, Mutex};

/// A stub provider on the loopback interface, answering OpenAI's
/// chat-completions format at `/v1/chat/completions` and Anthropic's messages
/// format at `/v1/messages`, and recording every request it was sent.
#[cfg(test)]
pub(crate) struct Stub {
    pub(crate) base: String,
    seen: Arc<Mutex<Vec<(String, serde_json::Value)>>>,
}

#[cfg(test)]
impl Stub {
    /// A stub answering every request with `status`, and with `answer` as the
    /// model's text when that is 200.
    ///
    /// Blocking sockets on a thread of its own, so that it serves whichever
    /// runtime a test happens to make.
    pub(crate) fn answering(status: u16, answer: &str) -> Self {
        Self::serving(status, answer, usize::MAX)
    }

    /// A stub answering the first `answered` requests with `answer`, and
    /// holding every later one open without a word - a provider a run can be
    /// interrupted while waiting on.
    pub(crate) fn holding_after(answered: usize, answer: &str) -> Self {
        Self::serving(200, answer, answered)
    }

    fn serving(status: u16, answer: &str, answered: usize) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let base = format!("http://{}/v1/", listener.local_addr().expect("an address"));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let recorded = seen.clone();
        let answer = answer.to_owned();
        std::thread::spawn(move || {
            let mut held = Vec::new();
            for mut connection in listener.incoming().flatten() {
                let Some((path, body)) = read_request(&mut connection) else {
                    continue;
                };
                let request = serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
                let mut log = recorded.lock().expect("the log");
                log.push((path.clone(), request));
                if log.len() > answered {
                    held.push(connection);
                    continue;
                }
                drop(log);
                let reply = if status != 200 {
                    serde_json::json!({"error": {"message": "the stub refused", "type": "stub"}})
                } else if path.ends_with("/messages") {
                    serde_json::json!({
                        "id": "m", "type": "message", "role": "assistant", "model": "stub",
                        "content": [{"type": "text", "text": answer}],
                        "stop_reason": "end_turn",
                        "usage": {"input_tokens": 1, "output_tokens": 1}
                    })
                } else {
                    serde_json::json!({
                        "id": "c", "object": "chat.completion", "created": 0, "model": "stub",
                        "choices": [{"index": 0, "finish_reason": "stop",
                                     "message": {"role": "assistant", "content": answer}}],
                        "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
                    })
                };
                let reply = reply.to_string();
                let _ = write!(
                    connection,
                    "HTTP/1.1 {status} Stub\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{reply}",
                    reply.len()
                );
            }
        });
        Self { base, seen }
    }

    /// Every request received so far: its path and its body.
    pub(crate) fn requests(&self) -> Vec<(String, serde_json::Value)> {
        self.seen.lock().expect("the log").clone()
    }
}

/// One HTTP request's path and body, or `None` if the connection closed first.
#[cfg(test)]
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
            let line = line.to_ascii_lowercase();
            line.strip_prefix("content-length:")
                .map(|value| value.trim().parse().unwrap_or(0))
        })
        .unwrap_or(0);
    while received.len() < head_end + 4 + length {
        let read = connection.read(&mut chunk).ok()?;
        if read == 0 {
            break;
        }
        received.extend_from_slice(&chunk[..read]);
    }
    let path = head.split_whitespace().nth(1)?.to_owned();
    let body = String::from_utf8_lossy(&received[head_end + 4..]).to_string();
    Some((path, body))
}

/// A client sending every model to `base`: names beginning `anthropic::` in
/// Anthropic's format, all others in OpenAI's, the prefix taken off.
#[cfg(test)]
pub(crate) fn client_for(base: &str) -> Client {
    use genai::adapter::AdapterKind;
    use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
    use genai::{ModelIden, ServiceTarget};

    let base = base.to_owned();
    let resolver = ServiceTargetResolver::from_resolver_fn(
        move |target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
            let name = target.model.model_name.to_string();
            let (kind, model) = match name.split_once("::") {
                Some(("anthropic", model)) => (AdapterKind::Anthropic, model.to_owned()),
                Some((_, model)) => (AdapterKind::OpenAI, model.to_owned()),
                None => (AdapterKind::OpenAI, name),
            };
            Ok(ServiceTarget {
                endpoint: Endpoint::from_owned(base.clone()),
                auth: AuthData::from_single("no-key-needed"),
                model: ModelIden::new(kind, model),
            })
        },
    );
    Client::builder()
        .with_service_target_resolver(resolver)
        .build()
        .expect("a client")
}

/// A base address nothing listens on: a port taken and given back.
#[cfg(test)]
pub(crate) fn nowhere() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let address = listener.local_addr().expect("an address");
    drop(listener);
    format!("http://{address}/v1/")
}

/// The text of the one user message a request carried, whichever format it was
/// in, and how many messages it carried.
#[cfg(test)]
pub(crate) fn sent(request: &serde_json::Value) -> (usize, Option<String>, Option<String>) {
    let messages = request["messages"].as_array().cloned().unwrap_or_default();
    let first = messages.first().cloned().unwrap_or(serde_json::Value::Null);
    let text = first["content"]
        .as_str()
        .map(str::to_owned)
        .or_else(|| first["content"][0]["text"].as_str().map(str::to_owned));
    (
        messages.len(),
        first["role"].as_str().map(str::to_owned),
        text,
    )
}

#[cfg(test)]
use crate::scripted::{block, note};
#[cfg(test)]
use agconflo_core::IdSource;

#[cfg(test)]
#[test]
fn roles_reach_their_models() {
    let stub = Stub::answering(200, "answered");
    // `gpt-4o` is a role here that looks like a model name, mapped elsewhere.
    let roster = Roster::new(client_for(&stub.base))
        .map("drafting", "openai::gpt-draft")
        .map("review", "anthropic::claude-review")
        .map("gpt-4o", "openai::chosen");
    let prompt = note(&mut IdSource::new(), "note", "question");

    for role in ["drafting", "review", "gpt-4o"] {
        assert_eq!(block(roster.call(role, &prompt)), Ok("answered".to_owned()));
    }
    let seen: Vec<(String, String)> = stub
        .requests()
        .into_iter()
        .map(|(path, body)| (path, body["model"].as_str().unwrap_or("").to_owned()))
        .collect();
    assert_eq!(
        seen,
        [
            ("/v1/chat/completions".to_owned(), "gpt-draft".to_owned()),
            ("/v1/messages".to_owned(), "claude-review".to_owned()),
            ("/v1/chat/completions".to_owned(), "chosen".to_owned()),
        ]
    );
}

#[cfg(test)]
proptest::proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(24))]

    /// For any prompt text, and for a prompt composed of parts, in either
    /// format: one message, from the user, holding the rendering byte for byte,
    /// and no system prompt.
    #[test]
    fn prompt_sent_exactly(
        first in "(?s)[ \t\r\na-z\u{e9}\u{2014}\u{1F600}]{0,24}",
        second in "(?s)[ \t\r\na-z\u{e9}]{0,12}",
        anthropic in proptest::prelude::any::<bool>(),
    ) {
        let stub = Stub::answering(200, "ok");
        let model = if anthropic { "anthropic::m" } else { "openai::m" };
        let roster = Roster::new(client_for(&stub.base)).map("asking", model);
        let mut source = IdSource::new();
        let plain = note(&mut source, "note", &first);
        let tail = note(&mut source, "note", &second);
        let composed = Context::compose(
            &mut source,
            agconflo_core::ContextType::new("note").expect("a name"),
            [&plain, &tail],
            "\n--\n",
        )
        .expect("a fresh source issues");

        for prompt in [&plain, &composed] {
            proptest::prop_assert!(block(roster.call("asking", prompt)).is_ok());
        }
        let requests = stub.requests();
        proptest::prop_assert_eq!(requests.len(), 2);
        for ((_, body), prompt) in requests.iter().zip([&plain, &composed]) {
            let (count, role, text) = sent(body);
            proptest::prop_assert_eq!(count, 1);
            proptest::prop_assert_eq!(role.as_deref(), Some("user"));
            proptest::prop_assert_eq!(text, Some(prompt.render().into_owned()));
            proptest::prop_assert!(body.get("system").is_none(), "{}", body);
        }
    }
}

#[cfg(test)]
#[test]
fn unmapped_role_fails() {
    let stub = Stub::answering(200, "answered");
    let roster = Roster::new(client_for(&stub.base)).map("drafting", "openai::m");
    let prompt = note(&mut IdSource::new(), "note", "question");

    // `gpt-4o` is a name the client would resolve on its own.
    for role in ["gpt-4o", "draftng"] {
        assert_eq!(
            block(roster.call(role, &prompt)),
            Err(ModelFailure::Unmapped {
                role: role.to_owned()
            })
        );
    }
    assert!(stub.requests().is_empty(), "nothing was sent");
}

#[cfg(test)]
#[test]
fn provider_failure_carried() {
    let prompt = note(&mut IdSource::new(), "note", "question");
    for status in [401, 503] {
        let stub = Stub::answering(status, "unused");
        let roster = Roster::new(client_for(&stub.base)).map("drafting", "openai::m");
        match block(roster.call("drafting", &prompt)) {
            Err(ModelFailure::Provider {
                role,
                status: got,
                message,
            }) => {
                assert_eq!(role, "drafting");
                assert_eq!(got, Some(status));
                assert!(!message.is_empty());
            }
            other => panic!("expected the provider's failure, got {other:?}"),
        }
    }
}

#[cfg(test)]
#[test]
fn unreachable_provider_has_no_status() {
    let roster = Roster::new(client_for(&nowhere())).map("drafting", "openai::m");
    let prompt = note(&mut IdSource::new(), "note", "question");
    match block(roster.call("drafting", &prompt)) {
        Err(ModelFailure::Provider {
            role,
            status,
            message,
        }) => {
            assert_eq!(role, "drafting");
            assert_eq!(status, None, "there was no answer to have a status");
            assert!(!message.is_empty());
        }
        other => panic!("expected the call to fail, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn answer_kept_as_sent() {
    // Whitespace at both ends: `genai`'s OpenAI adapter trims it and its
    // Anthropic adapter keeps it (EVD_GENAI_OPENAI_TRIMS).
    let answer = "\n  an answer, spaced \n\n";
    let stub = Stub::answering(200, answer);
    let roster = Roster::new(client_for(&stub.base))
        .map("openai", "openai::m")
        .map("anthropic", "anthropic::m");
    let prompt = note(&mut IdSource::new(), "note", "question");

    for role in ["openai", "anthropic"] {
        assert_eq!(
            block(roster.call(role, &prompt)),
            Ok(answer.to_owned()),
            "{role}"
        );
    }
}
