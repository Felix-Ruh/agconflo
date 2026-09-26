//! The model roster: the caller's mapping from role to model, and the client
//! that reaches them. Nothing here knows about scripts or activations.

use std::collections::HashMap;
use std::fmt;

use agconflo_core::{Call, Context};
use genai::Client;
use genai::chat::{
    ChatMessage, ChatOptions, ChatRequest, ContentPart, MessageContent, Tool, ToolCall,
    ToolResponse,
};

/// Which model plays each role, and the client that reaches them.
///
/// The client is the caller's own: where each model's requests go and what
/// credentials they carry are decided when it is built, and the roster reads no
/// credentials itself. Models are named as `genai` names them - `openai::gpt-...`,
/// `claude-...`.
// @A mapping from role to model,IMPL_MODELS_ROSTER,impl,[CREQ_ROSTER_ROLE_TO_MODEL],[DEC_MODELS_BY_ROLE, DEC_DECISIONS_THROUGH_THEIR_ENDPOINT]
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
    /// replaces its model.
    pub fn map(mut self, role: &str, model: &str) -> Self {
        self.roles.insert(role.to_owned(), model.to_owned());
        self
    }

    /// Ask the model `role` is mapped to about `prompt`, offering it nothing, and
    /// return its answer's text: a window of one part, the user's.
    #[cfg(test)]
    pub(crate) async fn call(&self, role: &str, prompt: &Context) -> Result<String, ModelFailure> {
        self.send(role, &[Part::User(prompt.clone())], &[])
            .await
            .map(|answered| answered.text)
    }

    /// Send the model `role` is mapped to `window`, offering it `offer`, and
    /// return what it answered: its text, and every call it made.
    ///
    /// Each part goes as the message it is, and no text goes that is not the
    /// rendering of one of the window's or the offer's contexts, whole and byte
    /// for byte: no system prompt, no instructions, nothing trimmed. A window of
    /// one user part is one user message holding its rendering. With nothing
    /// offered, no tools are sent.
    ///
    /// The answer's text is the text the provider sent, which excludes any
    /// reasoning the model reports separately, and is empty when the response
    /// holds none - as it does beside a call in OpenAI's format. Its calls come
    /// back in the order it made them, unjudged.
    ///
    /// A role mapped to nothing fails before anything is sent.
    // @One role one model and every part as it is,IMPL_MODELS_CALL,impl,[CREQ_ROSTER_ROLE_TO_MODEL, CREQ_ROSTER_CONTEXTS_WHOLE, CREQ_ROSTER_UNMAPPED_ROLE, CREQ_ROSTER_PROVIDER_FAILURE, CREQ_ROSTER_ANSWER_AS_SENT],[DEC_WINDOW_IS_A_CONTEXT, DEC_ANSWER_AS_SENT, DEC_MODELS_BY_ROLE]
    pub(crate) async fn send(
        &self,
        role: &str,
        window: &[Part],
        offer: &[Offered],
    ) -> Result<Answered, ModelFailure> {
        let Some(model) = self.roles.get(role) else {
            return Err(ModelFailure::Unmapped {
                role: role.to_owned(),
            });
        };
        let mut request = ChatRequest::default();
        for message in messages(window) {
            request = request.append_message(message);
        }
        if !offer.is_empty() {
            request = request.with_tools(offer.iter().map(tool).collect::<Vec<_>>());
        }
        let options = ChatOptions::default().with_capture_raw_body(true);
        match self.client.exec_chat(model, request, Some(&options)).await {
            Ok(response) => {
                let calls = returned_calls(&response.tool_calls());
                let text = response
                    .captured_raw_body
                    .as_ref()
                    .and_then(answer_as_sent)
                    .unwrap_or_else(|| response.into_first_text().unwrap_or_default());
                Ok(Answered { text, calls })
            }
            Err(error) => Err(ModelFailure::Provider {
                role: role.to_owned(),
                status: error.status().map(|status| status.as_u16()),
                message: error.to_string(),
            }),
        }
    }
}

/// One part of a model call's window, as the script host knows it from what the
/// run holds.
// @A window's part by its provenance,TRACE_MODELS_PART,trace,[],[DEC_WINDOW_IS_A_CONTEXT]
#[derive(Clone, Debug)]
pub(crate) enum Part {
    /// A part no answer or call produced - a script's prompt among them - sent
    /// as the user's.
    User(Context),
    /// A model's answer, sent as the model's turn with the calls it made: each
    /// named by its node type and filled with its contexts' renderings.
    Answer { answer: Context, calls: Vec<Call> },
    /// A call's output, sent as that call's result, paired with it by the
    /// provider's identifier.
    Result { call: String, output: Context },
}

/// One node type offered to a model, as the contexts its offer is made of: its
/// name, its description, and each parameter's name with whether it is required.
// @An offer made of contexts,TRACE_MODELS_OFFERED,trace,[],[DEC_TOOLS_OFFERED_AS_CONTEXTS]
#[derive(Clone, Debug)]
pub(crate) struct Offered {
    pub(crate) name: Context,
    pub(crate) description: Context,
    pub(crate) parameters: Vec<(Context, bool)>,
}

/// What a model answered: its text, and each call it made.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Answered {
    pub(crate) text: String,
    pub(crate) calls: Vec<Asked>,
}

/// One call a model's answer made, as the provider sent it: its identifier, the
/// name it called, and its arguments as `genai` parsed them. Nothing about it
/// has been checked.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Asked {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) arguments: serde_json::Value,
}

/// The window's parts as the messages they are.
///
/// A model's turn holds its text beside its calls when it has any, and no text
/// part when its answer is empty, which sends the whole of an empty rendering.
/// A call's arguments are an object of its contexts' renderings under their
/// parameters' names, in the order the call gave them; `genai` writes the JSON
/// around them.
// @Each part of a window sent as the message it is,IMPL_MODELS_MESSAGES,impl,[CREQ_ROSTER_PARTS_AS_MESSAGES, CREQ_ROSTER_CONTEXTS_WHOLE],[DEC_WINDOW_IS_A_CONTEXT, DEC_CALL_CARRIES_STRING_VALUES]
fn messages(window: &[Part]) -> Vec<ChatMessage> {
    window
        .iter()
        .map(|part| match part {
            Part::User(context) => ChatMessage::user(context.render().into_owned()),
            Part::Answer { answer, calls } => {
                let mut content = Vec::new();
                let text = answer.render();
                if !text.is_empty() {
                    content.push(ContentPart::Text(text.into_owned()));
                }
                for call in calls {
                    let arguments: serde_json::Map<String, serde_json::Value> = call
                        .inputs()
                        .iter()
                        .map(|(parameter, given)| {
                            (parameter.clone(), given.render().into_owned().into())
                        })
                        .collect();
                    content.push(ContentPart::ToolCall(ToolCall {
                        call_id: call.id().to_owned(),
                        fn_name: call.node_type().to_owned(),
                        fn_arguments: serde_json::Value::Object(arguments),
                        thought_signatures: None,
                    }));
                }
                ChatMessage::assistant(MessageContent::from_parts(content))
            }
            Part::Result { call, output } => ChatMessage::from(ToolResponse::new(
                call.clone(),
                output.render().into_owned(),
            )),
        })
        .collect()
}

/// One offered node type as a tool: named and described by its contexts'
/// renderings, taking one string for each parameter, the required ones marked
/// required.
// @An offered node type sent as a tool,IMPL_MODELS_TOOLS,impl,[CREQ_ROSTER_OFFER_AS_TOOLS],[DEC_TOOLS_OFFERED_AS_CONTEXTS]
fn tool(offered: &Offered) -> Tool {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    for (parameter, is_required) in &offered.parameters {
        let name = parameter.render().into_owned();
        properties.insert(name.clone(), serde_json::json!({"type": "string"}));
        if *is_required {
            required.push(serde_json::Value::String(name));
        }
    }
    Tool::new(offered.name.render().into_owned())
        .with_description(offered.description.render().into_owned())
        .with_schema(serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required,
        }))
}

/// Every call a response made, in its order, as the provider sent it.
// @Every call an answer makes handed back unjudged,IMPL_MODELS_CALLS_RETURNED,impl,[CREQ_ROSTER_CALLS_RETURNED]
fn returned_calls(calls: &[&ToolCall]) -> Vec<Asked> {
    calls
        .iter()
        .map(|call| Asked {
            id: call.call_id.clone(),
            name: call.fn_name.clone(),
            arguments: call.fn_arguments.clone(),
        })
        .collect()
}

/// The answer's text as the provider sent it, read from the response body: in
/// OpenAI's format one string, in Anthropic's the text blocks joined in order.
/// `None` for any other format.
// @The answer as the provider sent it,IMPL_MODELS_ANSWER_AS_SENT,impl,[CREQ_ROSTER_ANSWER_AS_SENT],[DEC_ANSWER_AS_SENT]
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
// @A model call's failure as values,IMPL_MODELS_FAILURE,impl,[CREQ_ROSTER_PROVIDER_FAILURE, CREQ_ROSTER_UNMAPPED_ROLE],[DEC_FAILURES_NON_EXHAUSTIVE]
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModelFailure {
    /// The role is mapped to no model, and nothing was sent.
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
// A stub provider, and a client pointed at it, for every module's model tests.

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
    /// model's text when that is 200. It serves from a thread of its own, under
    /// whatever runtime a test makes.
    pub(crate) fn answering(status: u16, answer: &str) -> Self {
        Self::serving(status, vec![Reply::text(answer)], true, usize::MAX)
    }

    /// A stub answering the first `answered` requests with `answer`, and
    /// holding every later one open without a word - a provider a run can be
    /// interrupted while waiting on.
    pub(crate) fn holding_after(answered: usize, answer: &str) -> Self {
        Self::serving(200, vec![Reply::text(answer)], true, answered)
    }

    /// A stub answering its n-th request with the n-th of `replies`, and
    /// holding every request past the last open without a word.
    pub(crate) fn replying(replies: Vec<Reply>) -> Self {
        let answered = replies.len();
        Self::serving(200, replies, false, answered)
    }

    fn serving(status: u16, replies: Vec<Reply>, repeat: bool, answered: usize) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let base = format!("http://{}/v1/", listener.local_addr().expect("an address"));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let recorded = seen.clone();
        std::thread::spawn(move || {
            let mut held = Vec::new();
            for mut connection in listener.incoming().flatten() {
                let Some((path, body)) = read_request(&mut connection) else {
                    continue;
                };
                let request = serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
                let mut log = recorded.lock().expect("the log");
                log.push((path.clone(), request));
                let index = log.len() - 1;
                if log.len() > answered {
                    held.push(connection);
                    continue;
                }
                drop(log);
                let reply = if repeat { &replies[0] } else { &replies[index] };
                let reply = if status != 200 {
                    serde_json::json!({"error": {"message": "the stub refused", "type": "stub"}})
                } else if path.ends_with("/messages") {
                    reply.anthropic()
                } else {
                    reply.openai()
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

/// One reply the stub gives: the model's text, and the calls it makes - each an
/// identifier, a name and the arguments as JSON text, which is how OpenAI's
/// format carries them and what Anthropic's is parsed from.
#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct Reply {
    text: String,
    calls: Vec<(String, String, String)>,
}

#[cfg(test)]
impl Reply {
    /// A reply of `text` and no call.
    pub(crate) fn text(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            calls: Vec::new(),
        }
    }

    /// The same reply, also calling `name` under `id` with `arguments`, JSON
    /// text written as the model wrote it.
    pub(crate) fn call(mut self, id: &str, name: &str, arguments: &str) -> Self {
        self.calls
            .push((id.to_owned(), name.to_owned(), arguments.to_owned()));
        self
    }

    /// The reply in OpenAI's chat-completions format: the text as `content`, or
    /// null beside calls when it is empty, and each call's arguments as the
    /// string it was written as.
    fn openai(&self) -> serde_json::Value {
        let mut message = serde_json::json!({"role": "assistant", "content": self.text});
        if !self.calls.is_empty() {
            if self.text.is_empty() {
                message["content"] = serde_json::Value::Null;
            }
            message["tool_calls"] = self
                .calls
                .iter()
                .map(|(id, name, arguments)| {
                    serde_json::json!({"id": id, "type": "function",
                                       "function": {"name": name, "arguments": arguments}})
                })
                .collect();
        }
        let finish = if self.calls.is_empty() {
            "stop"
        } else {
            "tool_calls"
        };
        serde_json::json!({
            "id": "c", "object": "chat.completion", "created": 0, "model": "stub",
            "choices": [{"index": 0, "finish_reason": finish, "message": message}],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
        })
    }

    /// The reply in Anthropic's messages format: the text as a text block when
    /// there is any or no call, and each call as a `tool_use` block whose input
    /// is its arguments parsed - or, when they are not JSON, the text itself.
    fn anthropic(&self) -> serde_json::Value {
        let mut content = Vec::new();
        if !self.text.is_empty() || self.calls.is_empty() {
            content.push(serde_json::json!({"type": "text", "text": self.text}));
        }
        for (id, name, arguments) in &self.calls {
            let input = serde_json::from_str::<serde_json::Value>(arguments)
                .unwrap_or_else(|_| serde_json::Value::String(arguments.clone()));
            content.push(
                serde_json::json!({"type": "tool_use", "id": id, "name": name, "input": input}),
            );
        }
        let stop = if self.calls.is_empty() {
            "end_turn"
        } else {
            "tool_use"
        };
        serde_json::json!({
            "id": "m", "type": "message", "role": "assistant", "model": "stub",
            "content": content, "stop_reason": stop,
            "usage": {"input_tokens": 1, "output_tokens": 1}
        })
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
    // Whitespace at both ends, which `genai`'s OpenAI adapter trims.
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

// --- windows, offers and calls ---------------------------------------------------

/// One message a request carried, whichever format: its role, its texts, the
/// calls it made - identifier, name, arguments - and the results it carried -
/// the call's identifier and the text.
#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SentMessage {
    pub(crate) role: String,
    pub(crate) texts: Vec<String>,
    pub(crate) calls: Vec<(String, String, serde_json::Value)>,
    pub(crate) results: Vec<(String, String)>,
}

/// Every message `request` carried, read from either format.
#[cfg(test)]
pub(crate) fn sent_messages(request: &serde_json::Value) -> Vec<SentMessage> {
    let text_of = |content: &serde_json::Value| -> String {
        match content {
            serde_json::Value::String(text) => text.clone(),
            serde_json::Value::Array(blocks) => blocks
                .iter()
                .filter_map(|block| block["text"].as_str())
                .collect(),
            _ => String::new(),
        }
    };
    let mut sent = Vec::new();
    for message in request["messages"].as_array().cloned().unwrap_or_default() {
        let mut this = SentMessage {
            role: message["role"].as_str().unwrap_or("").to_owned(),
            texts: Vec::new(),
            calls: Vec::new(),
            results: Vec::new(),
        };
        if this.role == "tool" {
            this.results.push((
                message["tool_call_id"].as_str().unwrap_or("").to_owned(),
                text_of(&message["content"]),
            ));
            sent.push(this);
            continue;
        }
        match &message["content"] {
            serde_json::Value::String(text) => this.texts.push(text.clone()),
            serde_json::Value::Array(blocks) => {
                for block in blocks {
                    match block["type"].as_str() {
                        Some("text") => this
                            .texts
                            .push(block["text"].as_str().unwrap_or("").to_owned()),
                        Some("tool_use") => this.calls.push((
                            block["id"].as_str().unwrap_or("").to_owned(),
                            block["name"].as_str().unwrap_or("").to_owned(),
                            block["input"].clone(),
                        )),
                        Some("tool_result") => this.results.push((
                            block["tool_use_id"].as_str().unwrap_or("").to_owned(),
                            text_of(&block["content"]),
                        )),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        for call in message["tool_calls"]
            .as_array()
            .cloned()
            .unwrap_or_default()
        {
            let arguments = call["function"]["arguments"].as_str().unwrap_or("null");
            this.calls.push((
                call["id"].as_str().unwrap_or("").to_owned(),
                call["function"]["name"].as_str().unwrap_or("").to_owned(),
                serde_json::from_str(arguments).unwrap_or(serde_json::Value::Null),
            ));
        }
        sent.push(this);
    }
    sent
}

/// Every tool `request` offered, read from either format: its name, its
/// description, its parameters' names with the type each is declared as, and
/// the names listed as required - or `None` when it offered no tools at all.
#[cfg(test)]
#[allow(clippy::type_complexity)]
pub(crate) fn sent_tools(
    request: &serde_json::Value,
) -> Option<Vec<(String, Option<String>, Vec<(String, String)>, Vec<String>)>> {
    let tools = request.get("tools")?.as_array()?;
    Some(
        tools
            .iter()
            .map(|tool| {
                let described = if tool["function"].is_object() {
                    &tool["function"]
                } else {
                    tool
                };
                let schema = if described["parameters"].is_object() {
                    &described["parameters"]
                } else {
                    &described["input_schema"]
                };
                let properties = schema["properties"]
                    .as_object()
                    .map(|properties| {
                        properties
                            .iter()
                            .map(|(name, declared)| {
                                (
                                    name.clone(),
                                    declared["type"].as_str().unwrap_or("").to_owned(),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let required = schema["required"]
                    .as_array()
                    .map(|required| {
                        required
                            .iter()
                            .filter_map(|name| name.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default();
                (
                    described["name"].as_str().unwrap_or("").to_owned(),
                    described["description"].as_str().map(str::to_owned),
                    properties,
                    required,
                )
            })
            .collect(),
    )
}

/// The node type `lookup` offered: one required parameter `query` and one
/// optional `hint`, described as `description`.
#[cfg(test)]
fn lookup_offered(source: &mut IdSource, description: &str) -> Offered {
    Offered {
        name: note(source, "note", "lookup"),
        description: note(source, "note", description),
        parameters: vec![
            (note(source, "note", "query"), true),
            (note(source, "note", "hint"), false),
        ],
    }
}

#[cfg(test)]
#[test]
fn window_parts_as_messages() {
    let stub = Stub::answering(200, "done");
    let roster = Roster::new(client_for(&stub.base))
        .map("openai", "openai::m")
        .map("anthropic", "anthropic::m");
    let mut source = IdSource::new();
    let prompt = note(&mut source, "note", "What is amber-7?");
    let answer = note(&mut source, "note", "Let me look.");
    let query = note(&mut source, "note", "amber-7");
    let output = note(&mut source, "note", "the harbour is closed");
    let window = [
        Part::User(prompt),
        Part::Answer {
            answer,
            calls: vec![Call::new("call_7", "lookup").input("query", query)],
        },
        Part::Result {
            call: "call_7".to_owned(),
            output,
        },
    ];
    let offer = [lookup_offered(&mut source, "Looks a codeword up.")];

    for role in ["openai", "anthropic"] {
        let answered = block(roster.send(role, &window, &offer)).expect("answered");
        assert_eq!(answered.text, "done");
    }
    let requests = stub.requests();
    for (_, body) in &requests {
        let sent = sent_messages(body);
        // The prompt as the user's; the answer as the model's turn, its text
        // beside its call under the provider's identifier; the output as that
        // call's result - a tool message in one format, a user turn holding a
        // result in the other.
        assert_eq!(sent.len(), 3, "{body}");
        assert_eq!(
            (sent[0].role.as_str(), sent[0].texts.as_slice()),
            ("user", &["What is amber-7?".to_owned()][..])
        );
        assert_eq!(sent[1].role, "assistant");
        assert_eq!(sent[1].texts, ["Let me look."], "{body}");
        assert_eq!(
            sent[1].calls,
            [(
                "call_7".to_owned(),
                "lookup".to_owned(),
                serde_json::json!({"query": "amber-7"})
            )]
        );
        assert_eq!(
            sent[2].results,
            [("call_7".to_owned(), "the harbour is closed".to_owned())],
            "{body}"
        );
    }
}

#[cfg(test)]
#[test]
fn offer_sent_as_tools() {
    let stub = Stub::answering(200, "done");
    let roster = Roster::new(client_for(&stub.base))
        .map("openai", "openai::m")
        .map("anthropic", "anthropic::m");
    let mut source = IdSource::new();
    let prompt = note(&mut source, "note", "q");
    // An empty description, which is sent as the empty text it is.
    let offer = [lookup_offered(&mut source, "")];

    for role in ["openai", "anthropic"] {
        block(roster.send(role, &[Part::User(prompt.clone())], &offer)).expect("answered");
        block(roster.send(role, &[Part::User(prompt.clone())], &[])).expect("answered");
    }
    let requests = stub.requests();
    assert_eq!(requests.len(), 4);
    for (index, (_, body)) in requests.iter().enumerate() {
        let tools = sent_tools(body);
        if index % 2 == 1 {
            assert_eq!(tools, None, "nothing offered, no tools field: {body}");
            continue;
        }
        assert_eq!(
            tools,
            Some(vec![(
                "lookup".to_owned(),
                Some(String::new()),
                vec![
                    ("query".to_owned(), "string".to_owned()),
                    ("hint".to_owned(), "string".to_owned())
                ],
                vec!["query".to_owned()],
            )]),
            "{body}"
        );
    }
}

#[cfg(test)]
#[test]
fn calls_returned_as_sent() {
    // Two calls: one offered, filled; one to a name never offered, whose
    // arguments are a bare string. Text beside them in Anthropic's format;
    // OpenAI's carries none beside a call.
    let replies = |text: &str| {
        Reply::text(text)
            .call("call_7", "lookup", "{\"query\": \"amber-7\"}")
            .call("toolu_9", "delete_everything", "\"amber-7\"")
    };
    let roster_for =
        |stub: &Stub, model: &str| Roster::new(client_for(&stub.base)).map("asking", model);
    let mut source = IdSource::new();
    let prompt = note(&mut source, "note", "q");
    let offer = [lookup_offered(&mut source, "Looks a codeword up.")];

    for (model, text) in [("openai::m", ""), ("anthropic::m", "Let me look.")] {
        let stub = Stub::replying(vec![replies(text)]);
        let answered =
            block(roster_for(&stub, model).send("asking", &[Part::User(prompt.clone())], &offer))
                .expect("answered");
        assert_eq!(answered.text, text, "{model}");
        assert_eq!(
            answered.calls,
            [
                Asked {
                    id: "call_7".to_owned(),
                    name: "lookup".to_owned(),
                    arguments: serde_json::json!({"query": "amber-7"}),
                },
                Asked {
                    id: "toolu_9".to_owned(),
                    name: "delete_everything".to_owned(),
                    arguments: serde_json::json!("amber-7"),
                },
            ],
            "{model}: in order, as sent, the unoffered one and its bare string included"
        );
    }
}

#[cfg(test)]
proptest::proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(24))]

    /// For any prompt, answer, description, argument and output - whitespace at
    /// either end, both line endings, characters outside ASCII, quotes and
    /// backslashes - every text a continuation's request carries in either format
    /// is the rendering of one of the window's or the offer's contexts, byte for
    /// byte, and every rendering that is not empty is carried.
    #[test]
    fn continuation_sends_contexts_whole(
        prompt in "(?s)[ \t\r\na-z\u{e9}\"\\\\\u{1F600}]{0,12}",
        answer in "(?s)[ \t\r\na-z\u{e9}\"\\\\]{0,12}",
        description in "(?s)[ \t\r\na-z\u{e9}\"\\\\]{0,12}",
        argument in "(?s)[ \t\r\na-z\u{e9}\"\\\\\u{1F600}]{0,12}",
        output in "(?s)[ \t\r\na-z\u{e9}\"\\\\]{0,12}",
        anthropic in proptest::prelude::any::<bool>(),
    ) {
        let stub = Stub::answering(200, "done");
        let model = if anthropic { "anthropic::m" } else { "openai::m" };
        let roster = Roster::new(client_for(&stub.base)).map("asking", model);
        let mut source = IdSource::new();
        let window = [
            Part::User(note(&mut source, "note", &prompt)),
            Part::Answer {
                answer: note(&mut source, "note", &answer),
                calls: vec![Call::new("call_1", "lookup").input("query", note(&mut source, "note", &argument))],
            },
            Part::Result { call: "call_1".to_owned(), output: note(&mut source, "note", &output) },
        ];
        let offer = [lookup_offered(&mut source, &description)];
        proptest::prop_assert!(block(roster.send("asking", &window, &offer)).is_ok());

        let renderings: Vec<&str> = vec![
            &prompt, &answer, &argument, &output, "lookup", &description, "query", "hint",
        ];
        let (_, body) = stub.requests().pop().expect("one request");
        let mut carried: Vec<String> = Vec::new();
        for message in sent_messages(&body) {
            carried.extend(message.texts);
            for (_, name, arguments) in message.calls {
                carried.push(name);
                if let Some(arguments) = arguments.as_object() {
                    for (parameter, value) in arguments {
                        carried.push(parameter.clone());
                        carried.push(value.as_str().unwrap_or("<not a string>").to_owned());
                    }
                }
            }
            carried.extend(message.results.into_iter().map(|(_, text)| text));
        }
        for (name, description, parameters, required) in sent_tools(&body).unwrap_or_default() {
            carried.push(name);
            carried.extend(description);
            carried.extend(parameters.into_iter().map(|(parameter, _)| parameter));
            carried.extend(required);
        }
        for text in &carried {
            proptest::prop_assert!(renderings.contains(&text.as_str()), "{:?} is no context's rendering: {}", text, body);
        }
        for rendering in renderings.iter().filter(|rendering| !rendering.is_empty()) {
            proptest::prop_assert!(carried.iter().any(|text| text == rendering), "{:?} was not carried: {}", rendering, body);
        }
    }
}
