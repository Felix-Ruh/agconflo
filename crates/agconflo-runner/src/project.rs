//! The project reader: a manifest and every file it names, read into what a
//! scripted run is started with.
//!
//! A manifest is a TOML document:
//!
//! ```toml
//! workflow = "flow.toml"            # the workflow document
//! types = ["types.toml"]            # the node type documents
//! budget = 20                       # the run's step budget
//! persons = ["review"]              # node types a person performs
//!
//! [scripts]                         # the script for each other node type
//! draft = "draft.lua"
//!
//! [tools]                           # node types a tool performs, and how
//! fetch = "read"                    # read, write or run
//! test = { action = "run", image = "python@sha256:...", container = "py" }
//!
//! [limits]                          # each optional
//! instructions = 10000000
//! memory = 67108864
//! model_calls = 4
//! ```
//!
//! Every path is relative to the manifest's own directory.

use std::fmt;
use std::path::Path;

use agconflo_core::{
    ReadFault, RepeatedTypes, TypeCatalogue, WorkflowDefinition, read_node_types, read_workflow,
};
use agconflo_lua::{Behaviours, Limits};

use crate::grants::{Action, pinned};
use crate::text::{FileFault, KeyFault, Place, Toml, path, read_text};

/// Everything a manifest names, read: the workflow with every node type its
/// documents declare, the scripts, the node types a person performs, the
/// tools, the budget and the limits.
#[derive(Clone, Debug)]
pub struct Project {
    definition: WorkflowDefinition,
    scripts: Vec<Script>,
    persons: Vec<String>,
    tools: Vec<Tool>,
    budget: usize,
    limits: Limits,
}

impl Project {
    /// The workflow, carrying every node type the node type documents declare.
    pub fn definition(&self) -> &WorkflowDefinition {
        &self.definition
    }

    /// The scripts, in the order the manifest lists them.
    pub fn scripts(&self) -> &[Script] {
        &self.scripts
    }

    /// The node types a person performs, in the order the manifest lists them.
    pub fn persons(&self) -> &[String] {
        &self.persons
    }

    /// The node types a tool performs, each with its action, in the order the
    /// manifest lists them.
    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    /// The action of the tool performing `node_type`, or `None` when no tool
    /// performs it.
    pub fn tool(&self, node_type: &str) -> Option<Action> {
        self.tools
            .iter()
            .find(|tool| tool.node_type == node_type)
            .map(|tool| tool.action)
    }

    /// The run's step budget.
    pub fn budget(&self) -> usize {
        self.budget
    }

    /// The limits each activation's script runs under.
    pub fn limits(&self) -> Limits {
        self.limits
    }

    /// What performs each node type: its script, named by the file it came
    /// from, or a person. A tool's node type is given as a person's: the
    /// scripted run hands each of its steps back.
    // @A tool's steps handed back as a person's,IMPL_PROJECT_TOOLS_AS_PERSONS,impl,[CREQ_PROJECT_READS_TOOLS],[DEC_TOOL_PERFORMED_BY_THE_RUNNER]
    pub fn behaviours(&self) -> Behaviours {
        let scripted = self
            .scripts
            .iter()
            .fold(Behaviours::new(), |behaviours, script| {
                behaviours.define(&script.node_type, &script.file, &script.text)
            });
        self.persons
            .iter()
            .chain(self.tools.iter().map(|tool| &tool.node_type))
            .fold(scripted, |behaviours, node_type| {
                behaviours.person(node_type)
            })
    }
}

/// One tool: the node type it performs, its action, the image it names and
/// the container it runs in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tool {
    /// The node type it performs.
    pub node_type: String,
    /// What it does.
    pub action: Action,
    /// The image it names, or `None` for the grants' image.
    pub image: Option<String>,
    /// The container it runs in, shared with every tool naming the same.
    pub container: String,
}

/// The container a tool naming none runs in.
pub const SHARED_CONTAINER: &str = "tools";

/// One script: the node type it performs, its file as the manifest writes it,
/// and its text as the file holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Script {
    /// The node type it performs.
    pub node_type: String,
    /// Its file, as the manifest writes the path.
    pub file: String,
    /// Its text, byte for byte.
    pub text: String,
}

/// Why a project cannot be read. Each fault names its file as the manifest
/// writes it, and the manifest as it was given.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProjectFault {
    /// The manifest or a file it names cannot be read as text.
    File(FileFault),
    /// The manifest is not a manifest, and where.
    Manifest {
        /// Where in the manifest.
        place: Place,
        /// What is wrong there.
        fault: KeyFault,
    },
    /// A workflow or node type document cannot be read, as the topology reader
    /// refused it.
    Document(ReadFault),
    /// Node type documents declare one name more than once.
    RepeatedTypes(RepeatedTypes),
    /// The manifest names a tool with an action there is none of.
    Action {
        /// Where the tool's node type is named.
        place: Place,
        /// The node type.
        node_type: String,
        /// The action, as the manifest names it.
        action: String,
    },
    /// The manifest gives a tool an image named by neither a digest nor an
    /// image id.
    Image {
        /// Where the image is named.
        place: Place,
        /// The tool's node type.
        node_type: String,
        /// The image, as the manifest names it.
        image: String,
    },
    /// The manifest gives a tool a container name that is not lower-case
    /// letters, digits, `_`, `.` and `-`, beginning with a letter or a digit.
    Container {
        /// Where the container is named.
        place: Place,
        /// The tool's node type.
        node_type: String,
        /// The container, as the manifest names it.
        container: String,
    },
    /// The manifest gives two tools one container and different images.
    TwoImages {
        /// Where the second tool's node type is named.
        place: Place,
        /// The second tool's node type.
        node_type: String,
        /// The first tool's node type.
        first: String,
        /// The container they share.
        container: String,
    },
    /// The manifest names one node type as performed in two ways.
    PerformedTwice {
        /// Where the node type is named the second way.
        place: Place,
        /// The node type.
        node_type: String,
        /// What it is named as performed by first: a script, a person or a
        /// tool.
        first: &'static str,
        /// What it is named as performed by second.
        second: &'static str,
    },
}

impl fmt::Display for ProjectFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(fault) => fault.fmt(f),
            Self::Manifest { place, fault } => write!(f, "{place}: {fault}"),
            Self::Document(fault) => fault.fmt(f),
            Self::RepeatedTypes(repeated) => repeated.fmt(f),
            Self::Action {
                place,
                node_type,
                action,
            } => write!(
                f,
                "{place}: the tool {node_type} names {action}, which is not an action; the actions are read, write and run"
            ),
            Self::Image {
                place,
                node_type,
                image,
            } => write!(
                f,
                "{place}: the tool {node_type} names the image {image}, which is named by neither a digest nor an image id"
            ),
            Self::Container {
                place,
                node_type,
                container,
            } => write!(
                f,
                "{place}: the tool {node_type} names the container {container:?}; a container name is lower-case letters, digits, _, . and -, beginning with a letter or a digit"
            ),
            Self::TwoImages {
                place,
                node_type,
                first,
                container,
            } => write!(
                f,
                "{place}: the tool {node_type} runs in the container {container} with another image than the tool {first}; a container has one image, so name a container of its own for it"
            ),
            Self::PerformedTwice {
                place,
                node_type,
                first,
                second,
            } => write!(
                f,
                "{place}: {node_type} is named as performed by {first} and by {second}; a node type is performed one way"
            ),
        }
    }
}

impl std::error::Error for ProjectFault {}

/// The project the manifest at `manifest` describes, or the first fault met
/// reading it: the manifest, then the node type documents, the workflow
/// document and the scripts, each in the order the manifest names them.
// @A manifest and every file it names read beside it,IMPL_PROJECT_READ,impl,[CREQ_PROJECT_READS_THE_MANIFEST, CREQ_PROJECT_REFUSES_UNREADABLE],[DEC_RUN_FROM_A_MANIFEST]
pub fn read_project(manifest: &Path) -> Result<Project, ProjectFault> {
    let named = manifest.display().to_string();
    let text = read_text(manifest, &named).map_err(ProjectFault::File)?;
    let written = Manifest::read(&named, &text)
        .map_err(|(place, fault)| ProjectFault::Manifest { place, fault })?;
    let tools = tools(&written)?;
    let directory = manifest.parent().unwrap_or(Path::new(""));
    let beside = |file: &str| read_text(&directory.join(file), file).map_err(ProjectFault::File);

    let mut documents = Vec::new();
    for file in &written.types {
        let text = beside(file)?;
        documents.push(read_node_types(file, &text).map_err(ProjectFault::Document)?);
    }
    let catalogue = TypeCatalogue::gather(documents).map_err(ProjectFault::RepeatedTypes)?;
    let text = beside(&written.workflow)?;
    let (definition, _) =
        read_workflow(&written.workflow, &text, &catalogue).map_err(ProjectFault::Document)?;
    let scripts = written
        .scripts
        .into_iter()
        .map(|(node_type, file)| {
            let text = beside(&file)?;
            Ok(Script {
                node_type,
                file,
                text,
            })
        })
        .collect::<Result<_, ProjectFault>>()?;

    Ok(Project {
        definition,
        scripts,
        persons: written.persons,
        tools,
        budget: written.budget,
        limits: written.limits,
    })
}

/// Each tool the manifest names, or the first refused: a node type named as
/// performed by more than one of a script, a person and a tool, then a tool
/// naming an action there is none of.
// @Each node type performed one way and each tool by an action,IMPL_PROJECT_TOOLS,impl,[CREQ_PROJECT_READS_TOOLS, CREQ_PROJECT_REFUSES_TOOL_FAULTS],[DEC_TOOLS_NAMED_IN_THE_MANIFEST, DEC_THREE_TOOL_ACTIONS]
fn tools(manifest: &Manifest) -> Result<Vec<Tool>, ProjectFault> {
    let mut performed: Vec<(&str, &'static str)> = Vec::new();
    let named = manifest
        .named_scripts
        .iter()
        .map(|(node_type, place)| (node_type, place, "a script"))
        .chain(
            manifest
                .named_persons
                .iter()
                .map(|(node_type, place)| (node_type, place, "a person")),
        )
        .chain(
            manifest
                .tools
                .iter()
                .map(|tool| (&tool.node_type, &tool.named, "a tool")),
        );
    for (node_type, place, way) in named {
        match performed
            .iter()
            .find(|(named, _)| *named == node_type.as_str())
        {
            Some((_, first)) if *first != way => {
                return Err(ProjectFault::PerformedTwice {
                    place: place.clone(),
                    node_type: node_type.clone(),
                    first,
                    second: way,
                });
            }
            Some(_) => {}
            None => performed.push((node_type, way)),
        }
    }

    let tools = manifest
        .tools
        .iter()
        .map(|written| {
            let action = Action::named(&written.action).ok_or_else(|| ProjectFault::Action {
                place: written.named.clone(),
                node_type: written.node_type.clone(),
                action: written.action.clone(),
            })?;
            Ok(Tool {
                node_type: written.node_type.clone(),
                action,
                image: written.image.as_ref().map(|(image, _)| image.clone()),
                container: written
                    .container
                    .as_ref()
                    .map_or(SHARED_CONTAINER, |(container, _)| container)
                    .to_owned(),
            })
        })
        .collect::<Result<Vec<_>, ProjectFault>>()?;
    environments(manifest, &tools)?;
    Ok(tools)
}

/// Nothing, or the first tool whose environment the manifest names wrongly:
/// an image not pinned, a container name malformed, or a container another
/// tool gave another image, compared as written.
// @Each tool's image pinned and container well named and one image to a container,IMPL_PROJECT_TOOL_ENVIRONMENT,impl,[CREQ_PROJECT_READS_TOOL_ENVIRONMENT, CREQ_PROJECT_REFUSES_ENVIRONMENT_FAULTS],[DEC_TOOL_NAMES_ITS_ENVIRONMENT, DEC_ONE_CONTAINER_PER_NAME, DEC_IMAGE_BY_DIGEST_NEVER_PULLED]
fn environments(manifest: &Manifest, tools: &[Tool]) -> Result<(), ProjectFault> {
    let mut containers: Vec<(&str, Option<&str>, &str)> = Vec::new();
    for (written, tool) in manifest.tools.iter().zip(tools) {
        if let Some((image, place)) = &written.image
            && !pinned(image)
        {
            return Err(ProjectFault::Image {
                place: place.clone(),
                node_type: tool.node_type.clone(),
                image: image.clone(),
            });
        }
        if let Some((container, place)) = &written.container
            && !well_named(container)
        {
            return Err(ProjectFault::Container {
                place: place.clone(),
                node_type: tool.node_type.clone(),
                container: container.clone(),
            });
        }
        let image = tool.image.as_deref();
        match containers
            .iter()
            .find(|(container, _, _)| *container == tool.container)
        {
            Some((_, first_image, first)) if *first_image != image => {
                return Err(ProjectFault::TwoImages {
                    place: written.named.clone(),
                    node_type: tool.node_type.clone(),
                    first: (*first).to_owned(),
                    container: tool.container.clone(),
                });
            }
            Some(_) => {}
            None => containers.push((&tool.container, image, &tool.node_type)),
        }
    }
    Ok(())
}

/// Whether `container` is lower-case letters, digits, `_`, `.` and `-`,
/// beginning with a letter or a digit.
fn well_named(container: &str) -> bool {
    let mut characters = container.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_lowercase() || first.is_ascii_digit())
        && characters
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | '-'))
}

/// A tool as a manifest writes it: its node type and where it is named, its
/// action, and the image and container it names with where each is named.
struct WrittenTool {
    node_type: String,
    named: Place,
    action: String,
    image: Option<(String, Place)>,
    container: Option<(String, Place)>,
}

/// A manifest's values, with its paths as it writes them, and where it names
/// each node type a script, a person or a tool performs.
struct Manifest {
    workflow: String,
    types: Vec<String>,
    budget: usize,
    scripts: Vec<(String, String)>,
    persons: Vec<String>,
    named_scripts: Vec<(String, Place)>,
    named_persons: Vec<(String, Place)>,
    tools: Vec<WrittenTool>,
    limits: Limits,
}

impl Manifest {
    /// The manifest `text`, named `file` in its faults.
    // @A manifest's keys read and every other refused,IMPL_PROJECT_MANIFEST,impl,[CREQ_PROJECT_REFUSES_UNKNOWN_KEYS, CREQ_PROJECT_REFUSES_MISSING_KEYS],[DEC_UNKNOWN_KEYS_REFUSED]
    fn read(file: &str, text: &str) -> Result<Self, (Place, KeyFault)> {
        let toml = Toml::parse(file, text)?;
        let root = toml.root();
        let top: &[String] = &[];
        toml.only(
            root,
            top,
            &[
                "workflow", "types", "budget", "persons", "scripts", "tools", "limits",
            ],
        )?;

        let key = |name: &str| path(top, name);
        let workflow = toml
            .string(toml.needed(root, top, "workflow")?, &key("workflow"))?
            .to_owned();
        let budget = toml.whole(
            toml.needed(root, top, "budget")?,
            &key("budget"),
            usize::MAX as u64,
        )?;
        let types = match root.get("types") {
            Some(item) => toml.strings(item, &key("types"))?,
            None => Vec::new(),
        };
        let (persons, named_persons) = match root.get("persons") {
            Some(item) => {
                let persons = toml.strings(item, &key("persons"))?;
                let places = item
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|value| toml.place(value.span()));
                let named = persons.iter().cloned().zip(places).collect();
                (persons, named)
            }
            None => (Vec::new(), Vec::new()),
        };

        let mut scripts = Vec::new();
        let mut named_scripts = Vec::new();
        if let Some(item) = root.get("scripts") {
            let table = toml.table(item, &key("scripts"))?;
            for (node_type, item) in table.iter() {
                let file = toml.string(item, &path(&key("scripts"), node_type))?;
                scripts.push((node_type.to_owned(), file.to_owned()));
                named_scripts.push((node_type.to_owned(), key_place(&toml, table, node_type)));
            }
        }

        let mut tools = Vec::new();
        if let Some(item) = root.get("tools") {
            let table = toml.table(item, &key("tools"))?;
            for (node_type, item) in table.iter() {
                tools.push(written_tool(&toml, table, node_type, item)?);
            }
        }

        let mut limits = Limits::default();
        if let Some(item) = root.get("limits") {
            let under = key("limits");
            let table = toml.table(item, &under)?;
            toml.only(table, &under, &["instructions", "memory", "model_calls"])?;
            if let Some(item) = table.get("instructions") {
                limits.instructions = toml.whole(item, &path(&under, "instructions"), u64::MAX)?;
            }
            if let Some(item) = table.get("memory") {
                limits.memory =
                    toml.whole(item, &path(&under, "memory"), usize::MAX as u64)? as usize;
            }
            if let Some(item) = table.get("model_calls") {
                limits.model_calls =
                    toml.whole(item, &path(&under, "model_calls"), u32::MAX.into())? as u32;
            }
        }

        Ok(Self {
            workflow,
            types,
            budget: budget as usize,
            scripts,
            persons,
            named_scripts,
            named_persons,
            tools,
            limits,
        })
    }
}

/// The tool `node_type`, whose entry of `table` is `item`: its action as a
/// string, or a table of its action, image and container.
fn written_tool(
    toml: &Toml<'_>,
    table: &dyn toml_edit::TableLike,
    node_type: &str,
    item: &toml_edit::Item,
) -> Result<WrittenTool, (Place, KeyFault)> {
    let key = path(&["tools".to_owned()], node_type);
    let named = key_place(toml, table, node_type);
    if let Some(action) = item.as_str() {
        return Ok(WrittenTool {
            node_type: node_type.to_owned(),
            named,
            action: action.to_owned(),
            image: None,
            container: None,
        });
    }
    let entry = toml.table(item, &key).map_err(|(place, _)| {
        (
            place,
            KeyFault::WrongKind {
                key: key.clone(),
                expected: "an action or a table",
                found: crate::text::article(item.type_name()),
            },
        )
    })?;
    toml.only(entry, &key, &["action", "image", "container"])?;
    let action = toml
        .string(toml.needed(entry, &key, "action")?, &path(&key, "action"))?
        .to_owned();
    let optional = |name: &str| -> Result<Option<(String, Place)>, (Place, KeyFault)> {
        entry
            .get(name)
            .map(|item| {
                let text = toml.string(item, &path(&key, name))?;
                Ok((text.to_owned(), toml.place(item.span())))
            })
            .transpose()
    };
    Ok(WrittenTool {
        node_type: node_type.to_owned(),
        named,
        action,
        image: optional("image")?,
        container: optional("container")?,
    })
}

/// Where the key `name` of `table` is written.
fn key_place(toml: &Toml<'_>, table: &dyn toml_edit::TableLike, name: &str) -> Place {
    toml.place(table.get_key_value(name).and_then(|(key, _)| key.span()))
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases.

#[cfg(test)]
use crate::testing::Scratch;
#[cfg(test)]
use proptest::prelude::*;

/// Two node types a script performs, and one a person performs between them.
#[cfg(test)]
const TYPES: &str = "[types.draft]\nrequired = { brief = \"note\" }\noutput = \"note\"\n\n[types.review]\nrequired = { draft = \"note\" }\noutput = \"verdict\"\n";

/// The third node type, and one no instance names.
#[cfg(test)]
const MORE_TYPES: &str = "[types.polish]\nrequired = { text = \"verdict\" }\noutput = \"note\"\n\n[types.spare]\noutput = \"note\"\n";

/// A chain of the three, named `name`.
#[cfg(test)]
fn flow(name: &str) -> String {
    format!(
        "name = \"{name}\"\noutput = \"polished\"\n\n[instances.drafted]\nnode_type = \"draft\"\nentry = true\n\n[instances.reviewed]\nnode_type = \"review\"\nbindings = {{ draft = \"drafted\" }}\n\n[instances.polished]\nnode_type = \"polish\"\nbindings = {{ text = \"reviewed\" }}\n"
    )
}

/// A manifest naming `flow.toml`, both node type documents and the scripts for
/// `draft` and `polish`, with `extra` before its scripts; the files it names
/// written beside it.
#[cfg(test)]
fn written(scratch: &Scratch, extra: &str) -> std::path::PathBuf {
    scratch.write("types.toml", TYPES);
    scratch.write("more/types.toml", MORE_TYPES);
    scratch.write("flow.toml", flow("chain"));
    scratch.write(
        "draft.lua",
        "local given, host = ...\nreturn host.text(host.output, 'd')\n",
    );
    scratch.write(
        "polish.lua",
        "local given, host = ...\nreturn host.text(host.output, 'p')\n",
    );
    scratch.write(
        "manifest.toml",
        format!(
            "workflow = \"flow.toml\"\ntypes = [\"types.toml\", \"more/types.toml\"]\nbudget = 7\npersons = [\"review\"]\n{extra}\n[scripts]\ndraft = \"draft.lua\"\npolish = \"polish.lua\"\n"
        ),
    )
}

/// The manifest `text`, with every other file `written` writes beside it.
#[cfg(test)]
fn read_manifest(scratch: &Scratch, text: &str) -> Result<Project, ProjectFault> {
    written(scratch, "");
    read_project(&scratch.write("manifest.toml", text))
}

/// The fault a manifest was refused for, or a panic naming what came instead.
#[cfg(test)]
fn manifest_fault(read: Result<Project, ProjectFault>) -> (Place, KeyFault) {
    match read {
        Err(ProjectFault::Manifest { place, fault }) => (place, fault),
        other => panic!("expected a manifest fault, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn reads_what_it_names() {
    let scratch = Scratch::new("reads_what_it_names");
    written(&scratch, "");
    scratch.write("spare.lua", "error('never run')");
    let manifest = scratch.write(
        "manifest.toml",
        "workflow = \"flow.toml\"\ntypes = [\"types.toml\", \"more/types.toml\"]\nbudget = 7\npersons = [\"review\"]\n\n[scripts]\ndraft = \"draft.lua\"\npolish = \"polish.lua\"\nspare = \"spare.lua\"\n\n[limits]\ninstructions = 1000\nmemory = 2048\nmodel_calls = 3\n",
    );

    let project = read_project(&manifest).expect("the project reads");

    let definition = project.definition();
    assert_eq!(definition.name, "chain");
    let instances: Vec<_> = definition
        .instances
        .iter()
        .map(|i| i.name.as_str())
        .collect();
    assert_eq!(instances, ["drafted", "reviewed", "polished"]);
    let node_types: Vec<_> = definition
        .node_types
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(node_types, ["draft", "review", "polish", "spare"]);
    let script = |node_type: &str, file: &str, text: &str| Script {
        node_type: node_type.to_owned(),
        file: file.to_owned(),
        text: text.to_owned(),
    };
    assert_eq!(
        project.scripts(),
        [
            script(
                "draft",
                "draft.lua",
                "local given, host = ...\nreturn host.text(host.output, 'd')\n"
            ),
            script(
                "polish",
                "polish.lua",
                "local given, host = ...\nreturn host.text(host.output, 'p')\n"
            ),
            script("spare", "spare.lua", "error('never run')"),
        ]
    );
    assert_eq!(project.persons(), ["review"]);
    assert_eq!(project.budget(), 7);
    assert_eq!(
        project.limits(),
        Limits {
            instructions: 1000,
            memory: 2048,
            model_calls: 3,
        }
    );
    // The person's type has no script and needs none; the others have one each.
    assert_eq!(project.behaviours().faults(definition), []);
}

#[cfg(test)]
#[test]
fn paths_from_the_manifest() {
    let scratch = Scratch::new("paths_from_the_manifest");
    scratch.write("shared/types.toml", format!("{TYPES}\n{MORE_TYPES}"));
    for (directory, name) in [("a/b", "first"), ("c", "second")] {
        scratch.write(&format!("{directory}/flow.toml"), flow(name));
        scratch.write(&format!("{directory}/draft.lua"), name);
    }
    scratch.write(
        "a/b/manifest.toml",
        "workflow = \"flow.toml\"\ntypes = [\"../../shared/types.toml\"]\nbudget = 1\n[scripts]\ndraft = \"draft.lua\"\n",
    );
    scratch.write(
        "c/manifest.toml",
        "workflow = \"flow.toml\"\ntypes = [\"../shared/types.toml\"]\nbudget = 1\n[scripts]\ndraft = \"draft.lua\"\n",
    );

    for (manifest, name) in [
        ("a/b/manifest.toml", "first"),
        ("c/manifest.toml", "second"),
    ] {
        let project = read_project(&scratch.path(manifest)).expect("the project reads");
        assert_eq!(project.definition().name, name);
        assert_eq!(project.scripts()[0].text, name);
        assert_eq!(project.definition().node_types.len(), 4);
    }
}

/// Text of ASCII and wider characters with every kind of line ending, and a
/// byte-order mark or none.
#[cfg(test)]
fn any_script() -> impl Strategy<Value = String> {
    let piece = prop_oneof![
        Just("\n".to_owned()),
        Just("\r\n".to_owned()),
        Just("\r".to_owned()),
        Just("é".to_owned()),
        Just("😀".to_owned()),
        "[ -~]{0,6}",
    ];
    (any::<bool>(), prop::collection::vec(piece, 0..24)).prop_map(|(mark, pieces)| {
        let text = pieces.concat();
        if mark {
            format!("\u{feff}{text}")
        } else {
            text
        }
    })
}

#[cfg(test)]
proptest! {
    #[test]
    fn scripts_kept_exactly(text in any_script()) {
        let scratch = Scratch::new("scripts_kept_exactly");
        written(&scratch, "");
        scratch.write("draft.lua", &text);

        let project = read_project(&scratch.path("manifest.toml")).expect("the project reads");

        prop_assert_eq!(&project.scripts()[0].text, &text);
    }
}

#[cfg(test)]
#[test]
fn limits_default_when_absent() {
    let scratch = Scratch::new("limits_default_when_absent");
    let project = read_project(&written(&scratch, "")).expect("the project reads");
    assert_eq!(project.limits(), Limits::default());

    let project =
        read_project(&written(&scratch, "[limits]\nmodel_calls = 5\n")).expect("the project reads");
    assert_eq!(
        project.limits(),
        Limits {
            model_calls: 5,
            ..Limits::default()
        }
    );
}

#[cfg(test)]
#[test]
fn missing_file_named() {
    let scratch = Scratch::new("missing_file_named");
    let absent = scratch.path("absent.toml");
    match read_project(&absent) {
        Err(ProjectFault::File(FileFault::Unreadable { file, message })) => {
            assert_eq!(file, absent.display().to_string());
            assert!(!message.is_empty());
        }
        other => panic!("expected the manifest unreadable, got {other:?}"),
    }

    let manifest = written(&scratch, "");
    let text = std::fs::read_to_string(&manifest).expect("the manifest");
    for (from, to) in [
        (
            "workflow = \"flow.toml\"",
            "workflow = \"nowhere/flow.toml\"",
        ),
        ("\"more/types.toml\"", "\"missing-types.toml\""),
        ("polish = \"polish.lua\"", "polish = \"missing.lua\""),
    ] {
        let named = to.split('"').nth(1).expect("the path");
        match read_manifest(&scratch, &text.replace(from, to)) {
            Err(ProjectFault::File(FileFault::Unreadable { file, .. })) => assert_eq!(file, named),
            other => panic!("expected {named} unreadable, got {other:?}"),
        }
    }
}

#[cfg(test)]
#[test]
fn faults_carry_their_place() {
    let scratch = Scratch::new("faults_carry_their_place");
    let (place, fault) = manifest_fault(read_manifest(
        &scratch,
        "workflow = \"flow.toml\"\nbudget = 2\n    = 1\n",
    ));
    assert_eq!((place.line, place.column), (3, 5), "{fault}");
    assert!(matches!(fault, KeyFault::Syntax { .. }), "{fault:?}");
    assert_eq!(
        place.file,
        scratch.path("manifest.toml").display().to_string()
    );

    written(&scratch, "");
    scratch.write("flow.toml", "name = \"chain\"\noutput = \n");
    match read_project(&scratch.path("manifest.toml")) {
        Err(ProjectFault::Document(fault)) => {
            assert_eq!(fault.document(), "flow.toml");
            assert_eq!(fault.line(), 2);
        }
        other => panic!("expected the workflow document refused, got {other:?}"),
    }

    written(&scratch, "");
    scratch.write("more/types.toml", "[types.polish]\noutput = \"\"\n");
    match read_project(&scratch.path("manifest.toml")) {
        Err(ProjectFault::Document(fault)) => {
            assert_eq!(fault.document(), "more/types.toml");
            assert_eq!((fault.line(), fault.column()), (2, 10));
            assert!(
                matches!(
                    fault.kind(),
                    agconflo_core::FaultKind::InvalidContextType { .. }
                ),
                "{fault:?}"
            );
        }
        other => panic!("expected the node type document refused, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn undecodable_file_refused() {
    let scratch = Scratch::new("undecodable_file_refused");
    let manifest = scratch.write("manifest.toml", b"workflow = \"\xff\"\n");
    assert_eq!(
        read_project(&manifest).map(|_| ()),
        Err(ProjectFault::File(FileFault::NotText {
            file: manifest.display().to_string()
        }))
    );

    for file in ["flow.toml", "polish.lua"] {
        let manifest = written(&scratch, "");
        scratch.write(file, b"name = \"\xff\"\n");
        assert_eq!(
            read_project(&manifest).map(|_| ()),
            Err(ProjectFault::File(FileFault::NotText {
                file: file.to_owned()
            }))
        );
    }
}

#[cfg(test)]
#[test]
fn unknown_key_refused() {
    let scratch = Scratch::new("unknown_key_refused");
    let (place, fault) = manifest_fault(read_manifest(
        &scratch,
        "workflow = \"flow.toml\"\nbudget = 2\npersns = [\"review\"]\n",
    ));
    assert_eq!((place.line, place.column), (3, 1));
    assert_eq!(
        fault,
        KeyFault::Unexpected {
            key: vec!["persns".to_owned()]
        }
    );

    let (place, fault) = manifest_fault(read_manifest(
        &scratch,
        "workflow = \"flow.toml\"\nbudget = 2\n[limits]\nmemory = 1\ninstruction = 5\n",
    ));
    assert_eq!((place.line, place.column), (5, 1));
    assert_eq!(
        fault,
        KeyFault::Unexpected {
            key: vec!["limits".to_owned(), "instruction".to_owned()]
        }
    );

    let project = read_manifest(
        &scratch,
        "workflow = \"flow.toml\"\nbudget = 2\n[scripts]\n\"a/b\" = \"draft.lua\"\n\"c:d\" = \"draft.lua\"\n\"e f\" = \"draft.lua\"\n",
    )
    .expect("the project reads");
    let node_types: Vec<_> = project
        .scripts()
        .iter()
        .map(|s| s.node_type.as_str())
        .collect();
    assert_eq!(node_types, ["a/b", "c:d", "e f"]);
}

#[cfg(test)]
#[test]
fn missing_or_mistyped_key_refused() {
    let scratch = Scratch::new("missing_or_mistyped_key_refused");
    let key = |names: &[&str]| names.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>();

    for (text, missing) in [
        ("budget = 2\n", "workflow"),
        ("workflow = \"flow.toml\"\n", "budget"),
    ] {
        let (place, fault) = manifest_fault(read_manifest(&scratch, text));
        assert_eq!((place.line, place.column), (1, 1));
        assert_eq!(
            fault,
            KeyFault::Missing {
                key: key(&[missing])
            }
        );
    }

    for (extra, at, found, line, column) in [
        ("budget = \"5\"\n", &["budget"][..], "a string", 2, 10),
        ("budget = -1\n", &["budget"][..], "-1", 2, 10),
        (
            "budget = 3\ntypes = \"types.toml\"\n",
            &["types"][..],
            "a string",
            3,
            9,
        ),
        (
            "budget = 3\n[scripts]\ndraft = 3\n",
            &["scripts", "draft"][..],
            "an integer",
            4,
            9,
        ),
    ] {
        let (place, fault) = manifest_fault(read_manifest(
            &scratch,
            &format!("workflow = \"flow.toml\"\n{extra}"),
        ));
        assert_eq!((place.line, place.column), (line, column), "{fault}");
        match fault {
            KeyFault::WrongKind {
                key: k, found: f, ..
            } => {
                assert_eq!(k, key(at));
                assert_eq!(f, found);
            }
            other => panic!("expected a value of the wrong kind, got {other:?}"),
        }
    }
}

/// Three node types a tool performs: one taking a path, one a path and a text,
/// one a command.
#[cfg(test)]
const TOOL_TYPES: &str = "[types.fetch]\nrequired = { path = \"note\" }\noutput = \"note\"\n\n[types.save]\nrequired = { path = \"note\", text = \"note\" }\noutput = \"note\"\n\n[types.exec]\nrequired = { command = \"note\" }\noutput = \"note\"\n";

/// A manifest naming a workflow in which each tool's node type has an
/// instance, between the script's first step and the person's review, with
/// `tools` as its tools table and `extra` after its scripts. Every file it
/// names written beside it.
#[cfg(test)]
fn with_tools(scratch: &Scratch, tools: &str, extra: &str) -> std::path::PathBuf {
    written(scratch, "");
    scratch.write("tools.toml", TOOL_TYPES);
    scratch.write(
        "tool-flow.toml",
        "name = \"tooled\"\noutput = \"polished\"\n\n[instances.drafted]\nnode_type = \"draft\"\nentry = true\n\n[instances.fetched]\nnode_type = \"fetch\"\nbindings = { path = \"drafted\" }\n\n[instances.saved]\nnode_type = \"save\"\nbindings = { path = \"drafted\", text = \"fetched\" }\n\n[instances.executed]\nnode_type = \"exec\"\nbindings = { command = \"saved\" }\n\n[instances.reviewed]\nnode_type = \"review\"\nbindings = { draft = \"executed\" }\n\n[instances.polished]\nnode_type = \"polish\"\nbindings = { text = \"reviewed\" }\n",
    );
    scratch.write(
        "manifest.toml",
        format!(
            "workflow = \"tool-flow.toml\"\ntypes = [\"types.toml\", \"more/types.toml\", \"tools.toml\"]\nbudget = 9\npersons = [\"review\"]\n\n[scripts]\ndraft = \"draft.lua\"\npolish = \"polish.lua\"\n{extra}\n[tools]\n{tools}"
        ),
    )
}

#[cfg(test)]
#[test]
fn reads_tools() {
    let scratch = Scratch::new("reads_tools");
    let manifest = with_tools(
        &scratch,
        "fetch = \"read\"\nsave = \"write\"\nexec = \"run\"\n",
        "",
    );

    let project = read_project(&manifest).expect("the project reads");

    let tool = |node_type: &str, action| Tool {
        node_type: node_type.to_owned(),
        action,
        image: None,
        container: SHARED_CONTAINER.to_owned(),
    };
    assert_eq!(
        project.tools(),
        [
            tool("fetch", Action::Read),
            tool("save", Action::Write),
            tool("exec", Action::Run),
        ]
    );
    assert_eq!(project.tool("save"), Some(Action::Write));
    assert_eq!(project.tool("draft"), None);
    assert_eq!(project.persons(), ["review"]);
    // Every instance's type has a person or one script and not both: each
    // tool's type is a person's, and none of them has a script.
    assert_eq!(project.behaviours().faults(project.definition()), []);
}

#[cfg(test)]
#[test]
fn tool_faults_refused() {
    let scratch = Scratch::new("tool_faults_refused");
    let tools = "fetch = \"read\"\nsave = \"write\"\nexec = \"run\"\n";
    let manifest_line = |manifest: &std::path::Path, text: &str| {
        let written = std::fs::read_to_string(manifest).expect("the manifest");
        let line = written
            .lines()
            .position(|line| line.starts_with(text))
            .expect("the line");
        line + 1
    };

    let manifest = with_tools(
        &scratch,
        "fetch = \"read\"\nsave = \"write\"\nexec = \"rum\"\n",
        "",
    );
    match read_project(&manifest) {
        Err(ProjectFault::Action {
            place,
            node_type,
            action,
        }) => {
            assert_eq!((node_type.as_str(), action.as_str()), ("exec", "rum"));
            assert_eq!(
                (place.line, place.column),
                (manifest_line(&manifest, "exec ="), 1)
            );
        }
        other => panic!("expected the action refused, got {other:?}"),
    }

    for (tools, extra, node_type, first, second, at) in [
        (
            format!("{tools}draft = \"read\"\n"),
            "",
            "draft",
            "a script",
            "a tool",
            "draft = \"read\"",
        ),
        (
            format!("{tools}review = \"run\"\n"),
            "",
            "review",
            "a person",
            "a tool",
            "review =",
        ),
        (
            tools.to_owned(),
            "review = \"polish.lua\"\n",
            "review",
            "a script",
            "a person",
            "persons =",
        ),
    ] {
        let manifest = with_tools(&scratch, &tools, extra);
        match read_project(&manifest) {
            Err(ProjectFault::PerformedTwice {
                place,
                node_type: named,
                first: was,
                second: also,
            }) => {
                assert_eq!((named.as_str(), was, also), (node_type, first, second));
                assert_eq!(place.line, manifest_line(&manifest, at));
            }
            other => panic!("expected {node_type} refused as {first} and {second}, got {other:?}"),
        }
    }

    // Each fault removed, the same manifest reads.
    read_project(&with_tools(&scratch, tools, "")).expect("the project reads");
}

#[cfg(test)]
#[test]
fn reads_tool_environment() {
    let scratch = Scratch::new("reads_tool_environment");
    let python = format!("python@sha256:{}", "a".repeat(64));
    let manifest = with_tools(
        &scratch,
        &format!(
            "fetch = \"read\"\nsave = {{ action = \"write\" }}\nexec = {{ action = \"run\", image = \"{python}\", container = \"py\" }}\nbuild = {{ action = \"run\", container = \"build\" }}\ntest = {{ action = \"read\", container = \"build\" }}\n"
        ),
        "",
    );
    scratch.write(
        "tools.toml",
        format!("{TOOL_TYPES}\n[types.build]\nrequired = {{ command = \"note\" }}\noutput = \"note\"\n\n[types.test]\nrequired = {{ path = \"note\" }}\noutput = \"note\"\n"),
    );

    let project = read_project(&manifest).expect("the project reads");

    let environment: Vec<_> = project
        .tools()
        .iter()
        .map(|tool| {
            (
                tool.node_type.as_str(),
                tool.image.as_deref(),
                tool.container.as_str(),
            )
        })
        .collect();
    assert_eq!(
        environment,
        [
            ("fetch", None, "tools"),
            ("save", None, "tools"),
            ("exec", Some(python.as_str()), "py"),
            ("build", None, "build"),
            ("test", None, "build"),
        ]
    );
    assert_eq!(project.tool("save"), Some(Action::Write));
}

#[cfg(test)]
#[test]
fn environment_faults_refused() {
    let scratch = Scratch::new("environment_faults_refused");
    let python = format!("python@sha256:{}", "a".repeat(64));
    let other = format!("python@sha256:{}", "b".repeat(64));
    let base = "fetch = \"read\"\nsave = \"write\"\n";
    let refused = |exec: &str| {
        let manifest = with_tools(&scratch, &format!("{base}exec = {exec}\n"), "");
        read_project(&manifest)
    };

    match refused("{ action = \"run\", image = \"python:3\" }") {
        Err(ProjectFault::Image {
            node_type, image, ..
        }) => {
            assert_eq!((node_type.as_str(), image.as_str()), ("exec", "python:3"));
        }
        other => panic!("expected the tagged image refused, got {other:?}"),
    }
    for name in ["Build", "a b", "-x", ".x", "a/b", ""] {
        match refused(&format!("{{ action = \"run\", container = \"{name}\" }}")) {
            Err(ProjectFault::Container {
                node_type,
                container,
                ..
            }) => {
                assert_eq!((node_type.as_str(), container.as_str()), ("exec", name));
            }
            other => panic!("expected the container {name:?} refused, got {other:?}"),
        }
    }

    let shared = |fetch: &str, exec: &str| {
        let manifest = with_tools(
            &scratch,
            &format!("fetch = {fetch}\nsave = \"write\"\nexec = {exec}\n"),
            "",
        );
        read_project(&manifest)
    };
    for (fetch, exec, shared_by) in [
        // An image and no container, beside a tool naming neither: both in
        // the shared container.
        (
            "\"read\"".to_owned(),
            format!("{{ action = \"run\", image = \"{python}\" }}"),
            SHARED_CONTAINER,
        ),
        (
            format!("{{ action = \"read\", container = \"build\", image = \"{python}\" }}"),
            format!("{{ action = \"run\", container = \"build\", image = \"{other}\" }}"),
            "build",
        ),
        (
            format!("{{ action = \"read\", container = \"build\", image = \"{python}\" }}"),
            "{ action = \"run\", container = \"build\" }".to_owned(),
            "build",
        ),
    ] {
        match shared(&fetch, &exec) {
            Err(ProjectFault::TwoImages {
                node_type,
                first,
                container,
                place,
            }) => {
                assert_eq!(
                    (node_type.as_str(), first.as_str(), container.as_str()),
                    ("exec", "fetch", shared_by)
                );
                let line = std::fs::read_to_string(scratch.path("manifest.toml"))
                    .expect("the manifest")
                    .lines()
                    .position(|line| line.starts_with("exec ="))
                    .expect("the line");
                assert_eq!(place.line, line + 1);
            }
            other => panic!("expected two images in one container refused, got {other:?}"),
        }
    }

    // The controls: a well-formed name, and two tools of one image in one
    // container.
    refused("{ action = \"run\", container = \"b.u_i-l0\" }").expect("the name is read");
    shared(
        &format!("{{ action = \"read\", container = \"build\", image = \"{python}\" }}"),
        &format!("{{ action = \"run\", container = \"build\", image = \"{python}\" }}"),
    )
    .expect("one image in one container is read");
}
