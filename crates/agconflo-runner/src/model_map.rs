//! The model map: a model mapping file read into the roster a scripted run
//! calls models through.
//!
//! A model mapping is a TOML document with one table, `roles`, giving each
//! role a model named with its provider, or a table naming the model, the
//! endpoint its calls go to, and the environment variable its key is in - or a
//! decisions model, with an endpoint ending in a slash and a key variable,
//! both required:
//!
//! ```toml
//! [roles]
//! drafting = "anthropic::claude-sonnet-5"
//! asking = { model = "openai::qwen3", endpoint = "http://localhost:1234/v1/", key_env = "LM_API_TOKEN" }
//! routing = { decisions = "~typesafe/jev-latest", endpoint = "https://openrouter.ai/api/alpha/", key_env = "OPEN_ROUTER_API_KEY" }
//! ```

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use crate::text::{FileFault, KeyFault, Place, Toml, article, path, read_text};
use agconflo_lua::Roster;
use genai::adapter::AdapterKind;
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};

/// A model mapping read: the roster its roles are played through.
#[derive(Clone, Debug)]
pub struct ModelMap {
    roster: Roster,
}

impl ModelMap {
    /// A mapping of no role at all, for a run whose scripts call no model.
    pub fn none() -> Result<Self, ModelsFault> {
        Ok(Self {
            roster: Roster::new(client(HashMap::new())?),
        })
    }

    /// The roster the mapping's roles are played through.
    pub fn roster(&self) -> &Roster {
        &self.roster
    }
}

/// Why a model mapping cannot be read. Each fault names the mapping's file as
/// it was given.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModelsFault {
    /// The file cannot be read as text.
    File(FileFault),
    /// The file is not a model mapping, and where.
    Mapping {
        /// Where in the file.
        place: Place,
        /// What is wrong there.
        fault: KeyFault,
    },
    /// A role's model is not named with the name of one of genai's adapters
    /// before a double colon, and a name after it.
    UnknownProvider {
        /// Where the model is named.
        place: Place,
        /// The role.
        role: String,
        /// The model as the mapping names it.
        model: String,
    },
    /// A role's entry names a decisions model it cannot be read with.
    Decisions {
        /// Where in the entry.
        place: Place,
        /// The role.
        role: String,
        /// What is wrong with it.
        fault: DecisionsFault,
    },
    /// A role's key is in a variable that is not set.
    UnsetVariable {
        /// Where the variable is named.
        place: Place,
        /// The role.
        role: String,
        /// The variable.
        variable: String,
    },
    /// No client to reach models through could be made.
    Client {
        /// genai's account of why.
        message: String,
    },
}

/// Why a role's decisions entry is refused.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecisionsFault {
    /// The entry also names a chat model.
    BothKinds,
    /// The entry names no endpoint.
    NoEndpoint,
    /// The entry's endpoint does not end in a slash.
    EndpointWithoutSlash {
        /// The endpoint as the mapping names it.
        endpoint: String,
    },
    /// The entry names no variable to take the key from.
    NoKeyVariable,
}

impl fmt::Display for DecisionsFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BothKinds => f.write_str("it names both a model and a decisions model"),
            Self::NoEndpoint => f.write_str("a decisions model needs an endpoint"),
            Self::EndpointWithoutSlash { endpoint } => write!(
                f,
                "its endpoint {endpoint} does not end in a slash, and decisions is joined to it"
            ),
            Self::NoKeyVariable => {
                f.write_str("a decisions model needs key_env, the variable its key is in")
            }
        }
    }
}

impl fmt::Display for ModelsFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(fault) => fault.fmt(f),
            Self::Mapping { place, fault } => write!(f, "{place}: {fault}"),
            Self::UnknownProvider { place, role, model } => write!(
                f,
                "{place}: the role {role} names {model}, which does not name a provider as provider::model"
            ),
            Self::Decisions { place, role, fault } => {
                write!(f, "{place}: the role {role}'s entry is refused: {fault}")
            }
            Self::UnsetVariable {
                place,
                role,
                variable,
            } => write!(
                f,
                "{place}: the key for the role {role} is in {variable}, which is not set"
            ),
            Self::Client { message } => {
                write!(f, "no client for the models could be made: {message}")
            }
        }
    }
}

impl std::error::Error for ModelsFault {}

/// The model mapping at `path`, with each key variable looked up in `env`, or
/// the first fault met reading it.
///
/// Each role's calls go to its model at the endpoint it names, or its
/// provider's own, with the key from the variable it names, or an empty key
/// when it names none.
// @A mapping read into a roster with every key from a variable it names,IMPL_MODEL_MAP_READ,impl,[CREQ_MODEL_MAP_ROLE_TO_MODEL, CREQ_MODEL_MAP_REFUSES_UNREADABLE],[DEC_MODELS_IN_A_FILE_OF_THEIR_OWN, DEC_KEYS_ONLY_BY_NAMED_VARIABLE, DEC_UNKNOWN_KEYS_REFUSED]
pub fn read_models(
    path: &Path,
    env: impl Fn(&str) -> Option<String>,
) -> Result<ModelMap, ModelsFault> {
    let file = path.display().to_string();
    let text = read_text(path, &file).map_err(ModelsFault::File)?;
    let roles = roles(&file, &text, &env)?;
    let targets = roles
        .iter()
        .filter_map(|role| match &role.plays {
            Plays::Chat(target) => Some((role.called_as(target), target.clone())),
            Plays::Decisions { .. } => None,
        })
        .collect();
    let roster = roles
        .iter()
        .fold(Roster::new(client(targets)?), |roster, role| {
            match &role.plays {
                Plays::Chat(target) => roster.map(&role.name, &role.called_as(target)),
                Plays::Decisions {
                    model,
                    endpoint,
                    key,
                } => roster.map_decisions(&role.name, model, endpoint, key),
            }
        });
    Ok(ModelMap { roster })
}

/// One role as the mapping gives it.
struct Role {
    name: String,
    plays: Plays,
}

/// What plays a role: a chat model at its target, or a decisions model at its
/// endpoint with its key.
enum Plays {
    Chat(Target),
    Decisions {
        model: String,
        endpoint: String,
        key: String,
    },
}

impl Role {
    /// The name the roster calls the role's chat model by: the provider's
    /// namespace and the role, which the client's resolver looks the role's
    /// target up by.
    fn called_as(&self, target: &Target) -> String {
        format!("{}::{}", target.adapter.as_lower_str(), self.name)
    }
}

/// Where one role's calls go: the adapter, the model's name without its
/// namespace, the endpoint when the mapping names one, and the key.
#[derive(Clone)]
struct Target {
    adapter: AdapterKind,
    model: String,
    endpoint: Option<String>,
    key: String,
}

/// Every role the mapping `text` gives, named `file` in its faults.
// @Each model named with its provider and each key variable set,IMPL_MODEL_MAP_ROLES,impl,[CREQ_MODEL_MAP_REFUSES_UNNAMESPACED, CREQ_MODEL_MAP_REFUSES_UNSET_VARIABLE, CREQ_MODEL_MAP_REFUSES_UNREADABLE],[DEC_MODELS_NAMED_WITH_THEIR_PROVIDER, DEC_KEYS_ONLY_BY_NAMED_VARIABLE]
fn roles(
    file: &str,
    text: &str,
    env: &impl Fn(&str) -> Option<String>,
) -> Result<Vec<Role>, ModelsFault> {
    let mapping = |(place, fault)| ModelsFault::Mapping { place, fault };
    let toml = Toml::parse(file, text).map_err(mapping)?;
    let root = toml.root();
    let top: &[String] = &[];
    toml.only(root, top, &["roles"]).map_err(mapping)?;
    let under = path(top, "roles");
    let table = toml
        .table(toml.needed(root, top, "roles").map_err(mapping)?, &under)
        .map_err(mapping)?;

    let mut roles = Vec::new();
    for (name, item) in table.iter() {
        let key = path(&under, name);
        let (model, model_key, endpoint, variable) = match item.as_str() {
            Some(_) => (item, key.clone(), None, None),
            None => {
                let role = toml
                    .table(item, &key)
                    .map_err(|(place, _)| ModelsFault::Mapping {
                        place,
                        fault: KeyFault::WrongKind {
                            key: key.clone(),
                            expected: "a model's name or a table",
                            found: article(item.type_name()),
                        },
                    })?;
                toml.only(role, &key, &["model", "decisions", "endpoint", "key_env"])
                    .map_err(mapping)?;
                if let Some(decisions) = role.get("decisions") {
                    roles.push(decider(&toml, env, name, &key, role, decisions)?);
                    continue;
                }
                (
                    toml.needed(role, &key, "model").map_err(mapping)?,
                    path(&key, "model"),
                    role.get("endpoint"),
                    role.get("key_env"),
                )
            }
        };

        let written = toml.string(model, &model_key).map_err(mapping)?;
        let (adapter, model_name) =
            provider(written).ok_or_else(|| ModelsFault::UnknownProvider {
                place: toml.place(model.span()),
                role: name.to_owned(),
                model: written.to_owned(),
            })?;
        let endpoint = endpoint
            .map(|item| {
                toml.string(item, &path(&key, "endpoint"))
                    .map(str::to_owned)
            })
            .transpose()
            .map_err(mapping)?;
        let key_value = match variable {
            None => String::new(),
            Some(item) => {
                let variable = toml.string(item, &path(&key, "key_env")).map_err(mapping)?;
                env(variable).ok_or_else(|| ModelsFault::UnsetVariable {
                    place: toml.place(item.span()),
                    role: name.to_owned(),
                    variable: variable.to_owned(),
                })?
            }
        };

        roles.push(Role {
            name: name.to_owned(),
            plays: Plays::Chat(Target {
                adapter,
                model: model_name.to_owned(),
                endpoint,
                key: key_value,
            }),
        });
    }
    Ok(roles)
}

/// The role `name`, whose entry `role` under `key` names the decisions model
/// `decisions`: read with its endpoint and the key from the variable it names,
/// or refused at the entry.
// @A decisions entry read whole or refused at the entry,IMPL_MODEL_MAP_DECISIONS,impl,[CREQ_MODEL_MAP_READS_DECISIONS, CREQ_MODEL_MAP_REFUSES_BAD_DECISIONS_ENTRY, CREQ_MODEL_MAP_REFUSES_UNSET_VARIABLE],[DEC_DECISIONS_ROLE_IN_THE_MAP, DEC_KEYS_ONLY_BY_NAMED_VARIABLE]
fn decider(
    toml: &Toml<'_>,
    env: &impl Fn(&str) -> Option<String>,
    name: &str,
    key: &[String],
    role: &dyn toml_edit::TableLike,
    decisions: &toml_edit::Item,
) -> Result<Role, ModelsFault> {
    let mapping = |(place, fault)| ModelsFault::Mapping { place, fault };
    let refused = |item: &toml_edit::Item, fault| ModelsFault::Decisions {
        place: toml.place(item.span()),
        role: name.to_owned(),
        fault,
    };
    let model = toml
        .string(decisions, &path(key, "decisions"))
        .map_err(mapping)?;
    if role.get("model").is_some() {
        return Err(refused(decisions, DecisionsFault::BothKinds));
    }
    let Some(endpoint_item) = role.get("endpoint") else {
        return Err(refused(decisions, DecisionsFault::NoEndpoint));
    };
    let endpoint = toml
        .string(endpoint_item, &path(key, "endpoint"))
        .map_err(mapping)?;
    if !endpoint.ends_with('/') {
        return Err(refused(
            endpoint_item,
            DecisionsFault::EndpointWithoutSlash {
                endpoint: endpoint.to_owned(),
            },
        ));
    }
    let Some(variable_item) = role.get("key_env") else {
        return Err(refused(decisions, DecisionsFault::NoKeyVariable));
    };
    let variable = toml
        .string(variable_item, &path(key, "key_env"))
        .map_err(mapping)?;
    let key_value = env(variable).ok_or_else(|| ModelsFault::UnsetVariable {
        place: toml.place(variable_item.span()),
        role: name.to_owned(),
        variable: variable.to_owned(),
    })?;
    Ok(Role {
        name: name.to_owned(),
        plays: Plays::Decisions {
            model: model.to_owned(),
            endpoint: endpoint.to_owned(),
            key: key_value,
        },
    })
}

/// The adapter a model named `provider::model` goes through, and the model's
/// name without the namespace, or `None` when the namespace names none of
/// genai's adapters or no name follows it.
fn provider(written: &str) -> Option<(AdapterKind, &str)> {
    let (namespace, model) = written.split_once("::")?;
    let adapter = AdapterKind::from_lower_str(namespace)?;
    (!model.is_empty()).then_some((adapter, model))
}

/// A client that sends each model called as one of `targets` to its target,
/// with its key, and every other as genai would.
// @Each role's calls resolved to its target and nothing else,TRACE_MODEL_MAP_RESOLVER,trace,[],[DEC_KEYS_ONLY_BY_NAMED_VARIABLE]
fn client(targets: HashMap<String, Target>) -> Result<Client, ModelsFault> {
    let resolver = ServiceTargetResolver::from_resolver_fn(
        move |service: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
            let Some(target) = targets.get(&service.model.model_name.to_string()) else {
                return Ok(service);
            };
            Ok(ServiceTarget {
                endpoint: target
                    .endpoint
                    .clone()
                    .map_or(service.endpoint, Endpoint::from_owned),
                auth: AuthData::from_single(target.key.clone()),
                model: ModelIden::new(target.adapter, target.model.clone()),
            })
        },
    );
    Client::builder()
        .with_service_target_resolver(resolver)
        .build()
        .map_err(|error| ModelsFault::Client {
            message: error.to_string(),
        })
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases.

#[cfg(test)]
use crate::testing::{Request, Scratch, Stub, without_keys};
#[cfg(test)]
use agconflo_core::{
    Arguments, IdSource, RunEnding, TypeCatalogue, WorkflowDefinition, read_node_types,
    read_workflow,
};
#[cfg(test)]
use agconflo_lua::{Behaviours, Limits, Outcome, run_scripted};

/// A workflow of one node whose script asks each of `roles` in turn and
/// composes their answers, joined by `|`.
#[cfg(test)]
fn asking(roles: &[&str]) -> (WorkflowDefinition, Behaviours) {
    let types = read_node_types("types.toml", "[types.ask]\noutput = \"note\"\n").expect("types");
    let catalogue = TypeCatalogue::gather([types]).expect("declared once");
    let (definition, _) = read_workflow(
        "flow.toml",
        "name = \"asking\"\noutput = \"asked\"\n\n[instances.asked]\nnode_type = \"ask\"\n",
        &catalogue,
    )
    .expect("the workflow");
    let calls: String = roles
        .iter()
        .map(|role| format!("answers[#answers + 1] = host.complete('{role}', prompt)\n"))
        .collect();
    let script = format!(
        "local given, host = ...\nlocal prompt = host.text('note', 'hi')\nlocal answers = {{}}\n{calls}return host.compose(host.output, answers, '|')\n"
    );
    (
        definition,
        Behaviours::new().define("ask", "ask.lua", &script),
    )
}

/// What the asking workflow completes with through `map`, or a panic naming
/// how it ended instead.
#[cfg(test)]
fn asked(map: &ModelMap, roles: &[&str]) -> String {
    let (definition, behaviours) = asking(roles);
    let limits = Limits {
        model_calls: roles.len() as u32,
        ..Limits::default()
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a runtime");
    let outcome = runtime.block_on(run_scripted(
        &definition,
        &behaviours,
        map.roster(),
        Arguments::new(),
        &mut IdSource::new(),
        3,
        limits,
        |_| {},
    ));
    match outcome {
        Ok(Outcome::Ended(RunEnding::Completed(result))) => result.render().into_owned(),
        other => panic!("expected the run to complete, got {other:?}"),
    }
}

/// An environment holding exactly `pairs`.
#[cfg(test)]
fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let pairs: Vec<(String, String)> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    move |name| {
        pairs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
    }
}

/// A request as the stub records it.
#[cfg(test)]
fn sent(path: &str, model: &str, authorization: Option<usize>, api_key: Option<usize>) -> Request {
    Request {
        path: path.to_owned(),
        model: model.to_owned(),
        authorization,
        api_key,
    }
}

/// The mapping `text` written and read with nothing in its environment.
#[cfg(test)]
fn mapped(scratch: &Scratch, text: &str) -> Result<ModelMap, ModelsFault> {
    read_models(&scratch.write("models.toml", text), env(&[]))
}

#[cfg(test)]
#[test]
fn roles_reach_their_models() {
    let first = Stub::answering("one");
    let second = Stub::answering("two");
    let scratch = Scratch::new("roles_reach_their_models");
    let mapping = scratch.write(
        "models.toml",
        format!(
            "[roles]\nfirst = {{ model = \"openai::alpha\", endpoint = \"{}\", key_env = \"FIRST_KEY\" }}\nsecond = {{ model = \"anthropic::beta\", endpoint = \"{}\" }}\nthird = {{ model = \"openai::gamma\", endpoint = \"{}\", key_env = \"THIRD_KEY\" }}\n",
            first.base, second.base, second.base
        ),
    );

    let map = read_models(
        &mapping,
        env(&[("FIRST_KEY", "k1"), ("THIRD_KEY", "three")]),
    )
    .expect("the mapping reads");

    assert_eq!(asked(&map, &["first", "second", "third"]), "one|two|two");
    assert_eq!(
        first.requests(),
        [sent("/v1/chat/completions", "alpha", Some(2), None)]
    );
    assert_eq!(
        second.requests(),
        [
            sent("/v1/messages", "beta", None, Some(0)),
            sent("/v1/chat/completions", "gamma", Some(5), None),
        ]
    );
}

#[cfg(test)]
#[test]
fn default_key_never_sent() {
    if std::env::var_os("AGCONFLO_RUNNER_TEST_CHILD").is_none() {
        let child = without_keys(&mut std::process::Command::new(
            std::env::current_exe().expect("the test binary"),
        ))
        .args([
            "--exact",
            "model_map::default_key_never_sent",
            "--nocapture",
        ])
        .env("AGCONFLO_RUNNER_TEST_CHILD", "1")
        .env("ANTHROPIC_API_KEY", "planted-anthropic-key")
        .env("OPENAI_API_KEY", "planted-openai-key-of-another-length")
        .output()
        .expect("the child ran");
        let out = String::from_utf8_lossy(&child.stdout);
        assert!(child.status.success(), "{out}");
        assert!(out.contains("1 passed"), "the child ran no test: {out}");
        return;
    }

    let stub = Stub::answering("a");
    let scratch = Scratch::new("default_key_never_sent");
    let map = mapped(
        &scratch,
        &format!(
            "[roles]\nwriting = {{ model = \"anthropic::claude-probe\", endpoint = \"{0}\" }}\nasking = {{ model = \"openai::gpt-probe\", endpoint = \"{0}\" }}\n",
            stub.base
        ),
    )
    .expect("the mapping reads");

    asked(&map, &["writing", "asking"]);

    assert_eq!(
        stub.requests(),
        [
            sent("/v1/messages", "claude-probe", None, Some(0)),
            sent("/v1/chat/completions", "gpt-probe", Some(0), None),
        ]
    );
}

#[cfg(test)]
#[test]
fn unnamespaced_refused() {
    let scratch = Scratch::new("unnamespaced_refused");
    for model in [
        "claude-sonnet",
        "gpt-probe",
        "cluade-probe",
        "opnai::probe",
        "::probe",
        "openai::",
    ] {
        match mapped(&scratch, &format!("[roles]\nwriting = \"{model}\"\n")) {
            Err(ModelsFault::UnknownProvider {
                place,
                role,
                model: named,
            }) => {
                assert_eq!((place.line, place.column), (2, 11));
                assert_eq!(role, "writing");
                assert_eq!(named, model);
            }
            other => panic!("expected {model} refused, got {other:?}"),
        }
    }

    mapped(
        &scratch,
        "[roles]\nwriting = \"anthropic::claude-sonnet\"\n",
    )
    .expect("a model with its provider reads");

    match mapped(
        &scratch,
        "[roles]\nfirst = \"openai::alpha\"\nlast = \"cluade-probe\"\n",
    ) {
        Err(ModelsFault::UnknownProvider { role, .. }) => assert_eq!(role, "last"),
        other => panic!("expected the last role refused, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn unset_variable_refused() {
    let stub = Stub::answering("a");
    let scratch = Scratch::new("unset_variable_refused");
    let text = format!(
        "[roles]\nasking = {{ model = \"openai::alpha\", endpoint = \"{}\", key_env = \"NOT_SET_KEY\" }}\n",
        stub.base
    );
    let mapping = scratch.write("models.toml", &text);
    let column = text
        .lines()
        .nth(1)
        .expect("the role")
        .find("\"NOT_SET_KEY\"")
        .expect("the variable")
        + 1;

    match read_models(&mapping, env(&[])) {
        Err(ModelsFault::UnsetVariable {
            place,
            role,
            variable,
        }) => {
            assert_eq!((place.line, place.column), (2, column));
            assert_eq!(role, "asking");
            assert_eq!(variable, "NOT_SET_KEY");
        }
        other => panic!("expected the variable refused, got {other:?}"),
    }
    assert_eq!(stub.requests(), []);

    let map = read_models(&mapping, env(&[("NOT_SET_KEY", "")])).expect("an empty variable is set");
    asked(&map, &["asking"]);
    assert_eq!(
        stub.requests(),
        [sent("/v1/chat/completions", "alpha", Some(0), None)]
    );
}

#[cfg(test)]
#[test]
fn unreadable_refused() {
    let scratch = Scratch::new("unreadable_refused");
    let key = |names: &[&str]| names.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>();
    let refused = |text: &str| match mapped(&scratch, text) {
        Err(ModelsFault::Mapping { place, fault }) => {
            assert_eq!(
                place.file,
                scratch.path("models.toml").display().to_string()
            );
            ((place.line, place.column), fault)
        }
        other => panic!("expected {text:?} refused, got {other:?}"),
    };

    let (at, fault) = refused("[roles\n");
    assert_eq!(at.0, 1);
    assert!(matches!(fault, KeyFault::Syntax { .. }), "{fault:?}");

    assert_eq!(
        refused(""),
        (
            (1, 1),
            KeyFault::Missing {
                key: key(&["roles"])
            }
        )
    );
    assert_eq!(
        refused("timeout = 5\n[roles]\n"),
        (
            (1, 1),
            KeyFault::Unexpected {
                key: key(&["timeout"])
            }
        )
    );
    let role = "asking = { model = \"openai::a\", key_evn = \"X\" }";
    let column = role.find("key_evn").expect("the key") + 1;
    assert_eq!(
        refused(&format!("[roles]\n{role}\n")),
        (
            (2, column),
            KeyFault::Unexpected {
                key: key(&["roles", "asking", "key_evn"])
            }
        )
    );
    for (value, found) in [("5", "an integer"), ("[\"openai::a\"]", "an array")] {
        let (at, fault) = refused(&format!("[roles]\nasking = {value}\n"));
        assert_eq!(at, (2, 10));
        assert_eq!(
            fault,
            KeyFault::WrongKind {
                key: key(&["roles", "asking"]),
                expected: "a model's name or a table",
                found: found.to_owned(),
            }
        );
    }

    let absent = scratch.path("absent.toml");
    assert!(matches!(
        read_models(&absent, env(&[])),
        Err(ModelsFault::File(FileFault::Unreadable { file, .. })) if file == absent.display().to_string()
    ));
}

/// What a one-node workflow completes with through `map`, its script `script`,
/// under a limit of `calls` model calls - or a panic naming how it ended.
#[cfg(test)]
fn scripted(map: &ModelMap, script: &str, calls: u32) -> String {
    let types = read_node_types("types.toml", "[types.ask]\noutput = \"note\"\n").expect("types");
    let catalogue = TypeCatalogue::gather([types]).expect("declared once");
    let (definition, _) = read_workflow(
        "flow.toml",
        "name = \"asking\"\noutput = \"asked\"\n\n[instances.asked]\nnode_type = \"ask\"\n",
        &catalogue,
    )
    .expect("the workflow");
    let behaviours = Behaviours::new().define("ask", "ask.lua", script);
    let limits = Limits {
        model_calls: calls,
        ..Limits::default()
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a runtime");
    match runtime.block_on(run_scripted(
        &definition,
        &behaviours,
        map.roster(),
        Arguments::new(),
        &mut IdSource::new(),
        3,
        limits,
        |_| {},
    )) {
        Ok(Outcome::Ended(RunEnding::Completed(result))) => result.render().into_owned(),
        other => panic!("expected the run to complete, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn decisions_reach_their_model() {
    let chat = Stub::answering("chatted");
    let decider = Stub::answering(r#"{"verdict":{"choice":"accept","confidence":1}}"#);
    let scratch = Scratch::new("decisions_reach_their_model");
    let mapping = scratch.write(
        "models.toml",
        format!(
            "[roles]\nasking = {{ model = \"openai::alpha\", endpoint = \"{}\", key_env = \"CHAT_KEY\" }}\nrouting = {{ decisions = \"~typesafe/jev-latest\", endpoint = \"{}\", key_env = \"ROUTE_KEY\" }}\n",
            chat.base, decider.base
        ),
    );
    let map = read_models(
        &mapping,
        env(&[("CHAT_KEY", "k1"), ("ROUTE_KEY", "route-key")]),
    )
    .expect("the mapping reads");

    let script = "local given, host = ...\nlocal said = host.complete('asking', host.text('note', 'q'))\nlocal answer, chosen = host.decide('routing', host.text('note', 's'), { verdict = { instructions = 'i', options = { accept = 'a', revise = 'r' } } }, 'decision')\nreturn host.text(host.output, said:render() .. ' ' .. chosen.verdict.choice)";
    assert_eq!(scripted(&map, script, 2), "chatted accept");
    assert_eq!(
        chat.requests(),
        [sent("/v1/chat/completions", "alpha", Some(2), None)]
    );
    assert_eq!(
        decider.requests(),
        [sent("/v1/decisions", "~typesafe/jev-latest", Some(9), None)]
    );
}

#[cfg(test)]
#[test]
fn bad_decisions_entry_refused() {
    let scratch = Scratch::new("bad_decisions_entry_refused");
    let refused = |entry: &str| {
        let text = format!("[roles]\nrouting = {{ {entry} }}\n");
        match read_models(&scratch.write("models.toml", &text), env(&[("K", "k")])) {
            Err(ModelsFault::Decisions { place, role, fault }) => {
                assert_eq!(place.line, 2, "{entry}");
                assert_eq!(role, "routing");
                fault
            }
            other => panic!("expected {entry} refused, got {other:?}"),
        }
    };

    assert_eq!(
        refused(
            "model = \"openai::a\", decisions = \"~d\", endpoint = \"http://x/\", key_env = \"K\""
        ),
        DecisionsFault::BothKinds
    );
    assert_eq!(
        refused("decisions = \"~d\", endpoint = \"http://x/\""),
        DecisionsFault::NoKeyVariable
    );
    assert_eq!(
        refused("decisions = \"~d\", key_env = \"K\""),
        DecisionsFault::NoEndpoint
    );
    assert_eq!(
        refused("decisions = \"~d\", endpoint = \"http://x/api/alpha\", key_env = \"K\""),
        DecisionsFault::EndpointWithoutSlash {
            endpoint: "http://x/api/alpha".to_owned()
        }
    );

    read_models(
        &scratch.write(
            "models.toml",
            "[roles]\nrouting = { decisions = \"~d\", endpoint = \"http://x/\", key_env = \"K\" }\n",
        ),
        env(&[("K", "k")]),
    )
    .expect("a whole entry reads");
}
