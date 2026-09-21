//! The topology writer: a definition put into the workflow document it was read
//! from, by editing that document in place.
//!
//! Editing rather than writing anew is the whole design. A document regenerated
//! from a definition reads back as that definition and has lost every comment
//! and every key the definition does not carry (`EVD_TOML_EDIT_KEEPS_COMMENTS`),
//! so everything here changes the least it can: a value only where it differs,
//! with what surrounds it kept, and a table only where one was added or removed.

use std::collections::HashSet;
use std::fmt;

use toml_edit::{InlineTable, Item, Table, TableLike, Value};

use crate::reader::WorkflowDocument;
use crate::workflow::{Binding, NodeInstance, WorkflowDefinition};

/// Puts `definition` into `document`, or refuses to write it at all, naming
/// every shape it holds that a workflow document cannot express.
///
/// What is written is the definition's name, instances, bindings and output.
/// Its node types are not: they live in documents of their own
/// (`DEC_TYPES_IN_OWN_DOCUMENTS`), and writing one is authoring a type.
///
/// Instances and bindings the document already holds keep their place in it,
/// and ones the definition adds are appended (`DEC_DOCUMENT_KEEPS_ITS_ORDER`).
/// An instance the definition no longer holds goes, with everything written
/// under it and the comments above and beside it; a renamed one is, as far as a
/// definition can say, one removed and one added.
///
/// A refusal leaves the document exactly as it was: every shape is looked for
/// before anything is edited. And nothing is refused for being a wiring defect -
/// an instance of an unknown type, a binding to an instance that is not there,
/// no output at all - since each of those can be written, and a writer refusing
/// them would be a validator it is not, and would make a half-finished workflow
/// impossible to save.
// @Every change written into the document and nothing refused but the unwritable,IMPL_WRITER_EDIT,impl,[CREQ_WRITER_WRITES]
pub fn write_workflow(
    document: &mut WorkflowDocument,
    definition: &WorkflowDefinition,
) -> Result<(), UnwritableDefinition> {
    let shapes = unwritable_shapes(definition);
    if !shapes.is_empty() {
        return Err(UnwritableDefinition { shapes });
    }

    let root = document.toml.as_table_mut();
    set_value(root, "name", Value::from(definition.name.as_str()));
    match definition.designated_outputs.first() {
        Some(output) => set_value(root, "output", Value::from(output.as_str())),
        None => {
            root.remove("output");
        }
    }
    write_instances(root, &definition.instances);
    Ok(())
}

/// Every shape `definition` holds that a workflow document cannot express,
/// each named once.
///
/// Three of them, and each is ruled out by the format rather than by a check of
/// ours: every name is a table key (`DEC_NAMES_AS_KEYS`), so two instances under
/// one name or a parameter bound twice would be a key written twice; and the
/// output is one key naming one instance (`DEC_ONE_OUTPUT_KEY`). Writing any of
/// them means dropping one of the things it holds, which is why they are refused
/// rather than written as nearly as possible.
// @Every unwritable shape named before anything is edited,IMPL_WRITER_REFUSE,impl,[CREQ_WRITER_UNWRITABLE_REFUSED]
fn unwritable_shapes(definition: &WorkflowDefinition) -> Vec<UnwritableShape> {
    let mut shapes = Vec::new();

    let mut named = HashSet::new();
    let mut repeated = HashSet::new();
    for instance in &definition.instances {
        if !named.insert(instance.name.as_str()) && repeated.insert(instance.name.as_str()) {
            shapes.push(UnwritableShape::RepeatedInstance {
                instance: instance.name.clone(),
            });
        }
    }

    for instance in &definition.instances {
        let mut bound = HashSet::new();
        let mut twice = HashSet::new();
        for binding in &instance.bindings {
            if !bound.insert(binding.parameter.as_str()) && twice.insert(binding.parameter.as_str())
            {
                shapes.push(UnwritableShape::RepeatedBinding {
                    instance: instance.name.clone(),
                    parameter: binding.parameter.clone(),
                });
            }
        }
    }

    if definition.designated_outputs.len() > 1 {
        shapes.push(UnwritableShape::SeveralOutputs {
            designated: definition.designated_outputs.clone(),
        });
    }
    shapes
}

/// The instances table made to hold exactly `instances`: stale ones removed,
/// kept ones edited, new ones appended.
fn write_instances(root: &mut Table, instances: &[NodeInstance]) {
    if instances.is_empty() && !root.contains_key("instances") {
        return;
    }
    if !root.contains_key("instances") {
        root.insert("instances", Item::Table(implicit_table()));
    }
    let Some(table) = root.get_mut("instances").and_then(Item::as_table_like_mut) else {
        // The reader refuses a document whose instances are not a table, and
        // nothing here writes one, so a document handed over cannot hold that.
        unreachable!("a workflow document read without refusal keeps its instances in a table");
    };

    let kept: HashSet<&str> = instances
        .iter()
        .map(|instance| instance.name.as_str())
        .collect();
    remove_all_but(table, &kept);

    for instance in instances {
        match table
            .get_mut(&instance.name)
            .and_then(Item::as_table_like_mut)
        {
            Some(written) => write_instance(written, instance),
            None => {
                table.insert(&instance.name, new_instance(instance));
            }
        }
    }
}

/// One instance the document already holds, edited to match `instance`.
fn write_instance(written: &mut dyn TableLike, instance: &NodeInstance) {
    set_value(
        written,
        "node_type",
        Value::from(instance.node_type.as_str()),
    );

    // An entry key the document has is kept and set; an absent one is added only
    // for an entry node, since absent already reads as not one.
    if instance.entry || written.contains_key("entry") {
        set_value(written, "entry", Value::from(instance.entry));
    }

    write_bindings(written, &instance.bindings);
}

/// The bindings of one instance made to be exactly `bindings`.
fn write_bindings(written: &mut dyn TableLike, bindings: &[Binding]) {
    if !written.contains_key("bindings") {
        if !bindings.is_empty() {
            written.insert(
                "bindings",
                Item::Value(Value::InlineTable(inline(bindings))),
            );
        }
        return;
    }
    let Some(table) = written
        .get_mut("bindings")
        .and_then(Item::as_table_like_mut)
    else {
        unreachable!("a workflow document read without refusal keeps its bindings in a table");
    };

    let kept: HashSet<&str> = bindings
        .iter()
        .map(|binding| binding.parameter.as_str())
        .collect();
    remove_all_but(table, &kept);
    for binding in bindings {
        set_value(
            table,
            &binding.parameter,
            Value::from(binding.source.as_str()),
        );
    }
}

/// Every key of `table` not in `kept` removed, with whatever it holds.
fn remove_all_but(table: &mut dyn TableLike, kept: &HashSet<&str>) {
    let stale: Vec<String> = table
        .iter()
        .map(|(key, _)| key.to_owned())
        .filter(|key| !kept.contains(key.as_str()))
        .collect();
    for key in stale {
        table.remove(&key);
    }
}

/// `new` written under `key`, changing the document only if the value differs.
///
/// A value equal to the one written is left alone rather than written again,
/// because a value written anew is quoted the default way and `'sink'` would
/// come back as `"sink"`. A value that does change takes over the old one's
/// decor - the space and comment around it - which assigning alone drops
/// (`EVD_TOML_EDIT_ASSIGNING_LOSES_FORMAT`).
// @A value written only where it changed with what surrounds it kept,IMPL_WRITER_KEEP,impl,[CREQ_WRITER_KEEPS_UNREAD]
fn set_value(table: &mut dyn TableLike, key: &str, new: Value) {
    match table.get_mut(key) {
        Some(Item::Value(old)) if same(old, &new) => {}
        Some(Item::Value(old)) => {
            let decor = old.decor().clone();
            *old = new;
            *old.decor_mut() = decor;
        }
        _ => {
            table.insert(key, Item::Value(new));
        }
    }
}

/// Whether two values hold the same string or the same boolean, however each is
/// written.
fn same(written: &Value, new: &Value) -> bool {
    match (written, new) {
        (Value::String(written), Value::String(new)) => written.value() == new.value(),
        (Value::Boolean(written), Value::Boolean(new)) => written.value() == new.value(),
        _ => false,
    }
}

/// A new instance, as a table of its own. Inserted into an inline table of
/// instances it becomes an inline table, and into any other it is written under
/// its own header, after the instances already there
/// (`EVD_TOML_EDIT_WHOLE_TABLES`).
fn new_instance(instance: &NodeInstance) -> Item {
    let mut table = Table::new();
    table.insert(
        "node_type",
        Item::Value(Value::from(instance.node_type.as_str())),
    );
    if instance.entry {
        table.insert("entry", Item::Value(Value::from(true)));
    }
    if !instance.bindings.is_empty() {
        table.insert(
            "bindings",
            Item::Value(Value::InlineTable(inline(&instance.bindings))),
        );
    }
    Item::Table(table)
}

fn inline(bindings: &[Binding]) -> InlineTable {
    let mut table = InlineTable::new();
    for binding in bindings {
        table.insert(&binding.parameter, Value::from(binding.source.as_str()));
    }
    table
}

/// A table written only through the tables under it, so that adding the first
/// instance to a document with none writes `[instances.<name>]` rather than an
/// empty `[instances]` above it.
fn implicit_table() -> Table {
    let mut table = Table::new();
    table.set_implicit(true);
    table
}

/// The document's text as it now stands, in the line endings it was read with.
///
/// toml_edit writes every line ending as LF (`EVD_TOML_EDIT_WRITES_LF`), so a
/// document saved on Windows would come back with every line changed. One read
/// with a CRLF anywhere in it is written with CRLF throughout - exactly its own
/// text for a document written one way, which is what an editor writes; one
/// that mixed the two comes back all CRLF.
// @Line endings written as they were read,IMPL_WRITER_LINE_ENDINGS,impl,[CREQ_WRITER_KEEPS_UNREAD]
impl fmt::Display for WorkflowDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.toml.to_string();
        if self.crlf {
            f.write_str(&text.replace("\r\n", "\n").replace('\n', "\r\n"))
        } else {
            f.write_str(&text)
        }
    }
}

/// A definition refused because a workflow document cannot express it, with
/// every reason why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnwritableDefinition {
    shapes: Vec<UnwritableShape>,
}

impl UnwritableDefinition {
    /// Every shape the definition holds that a document cannot express.
    pub fn shapes(&self) -> &[UnwritableShape] {
        &self.shapes
    }
}

/// One shape the in-memory model can hold and a workflow document cannot.
///
/// The model holds them deliberately: a definition built by other means can
/// carry any of them, and the validator has to see a shape to refuse it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnwritableShape {
    /// Two or more instances share one name.
    RepeatedInstance {
        /// The name they share.
        instance: String,
    },
    /// One instance binds one parameter more than once.
    RepeatedBinding {
        /// The instance binding it.
        instance: String,
        /// The parameter bound more than once.
        parameter: String,
    },
    /// The definition designates more than one output.
    SeveralOutputs {
        /// Every instance it designates, in its order.
        designated: Vec<String>,
    },
}

impl fmt::Display for UnwritableDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (position, shape) in self.shapes.iter().enumerate() {
            if position > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{shape}")?;
        }
        Ok(())
    }
}

impl fmt::Display for UnwritableShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepeatedInstance { instance } => write!(
                f,
                "more than one instance is named '{instance}', where a document holds one"
            ),
            Self::RepeatedBinding {
                instance,
                parameter,
            } => write!(
                f,
                "'{parameter}' of '{instance}' is bound more than once, where a document binds it once"
            ),
            Self::SeveralOutputs { designated } => write!(
                f,
                "{} outputs are designated ('{}'), where a document names one",
                designated.len(),
                designated.join("', '")
            ),
        }
    }
}

impl std::error::Error for UnwritableDefinition {}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases, for the reason given in id.rs.

#[cfg(test)]
use crate::catalogue::{TypeCatalogue, node_type_document};
#[cfg(test)]
use crate::reader::{Pools, any_written, catalogue, read_workflow};
#[cfg(test)]
use crate::wiring::validate_wiring;
#[cfg(test)]
use crate::workflow::{instance, node_type};
#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use std::collections::BTreeMap;

/// A definition as reading a written document back is compared with it: its
/// name and output, how many instances it holds, and each instance and its
/// bindings under their names rather than in order, since a document keeps its
/// own order (`DEC_DOCUMENT_KEEPS_ITS_ORDER`).
#[cfg(test)]
type ByName = (
    String,
    Vec<String>,
    usize,
    BTreeMap<String, (String, bool, BTreeMap<String, String>)>,
);

#[cfg(test)]
fn by_name(definition: &WorkflowDefinition) -> ByName {
    let instances = definition
        .instances
        .iter()
        .map(|instance| {
            let bindings = instance
                .bindings
                .iter()
                .map(|binding| (binding.parameter.clone(), binding.source.clone()))
                .collect();
            (
                instance.name.clone(),
                (instance.node_type.clone(), instance.entry, bindings),
            )
        })
        .collect();
    (
        definition.name.clone(),
        definition.designated_outputs.clone(),
        definition.instances.len(),
        instances,
    )
}

/// What `document` reads back as, with `catalogue`.
#[cfg(test)]
fn read_back(document: &WorkflowDocument, catalogue: &TypeCatalogue) -> WorkflowDefinition {
    let text = document.to_string();
    match read_workflow(document.document(), &text, catalogue) {
        Ok((definition, _)) => definition,
        Err(fault) => panic!("what was written does not read: {fault}\n{text}"),
    }
}

/// One change to a definition, drawn by index so that it applies to whatever
/// the definition holds when it comes.
#[cfg(test)]
#[derive(Clone, Debug)]
enum Change {
    Add(usize, usize, usize),
    Remove(usize),
    Rename(usize, usize),
    Bind(usize, usize, usize),
    Repoint(usize, usize, usize),
    Unbind(usize, usize),
    Output(usize),
    NoOutput,
    Name(usize),
    Entry(usize),
}

#[cfg(test)]
fn any_change() -> impl Strategy<Value = Change> {
    prop_oneof![
        (any::<usize>(), any::<usize>(), any::<usize>()).prop_map(|(a, b, c)| Change::Add(a, b, c)),
        any::<usize>().prop_map(Change::Remove),
        (any::<usize>(), any::<usize>()).prop_map(|(a, b)| Change::Rename(a, b)),
        (any::<usize>(), any::<usize>(), any::<usize>())
            .prop_map(|(a, b, c)| Change::Bind(a, b, c)),
        (any::<usize>(), any::<usize>(), any::<usize>())
            .prop_map(|(a, b, c)| Change::Repoint(a, b, c)),
        (any::<usize>(), any::<usize>()).prop_map(|(a, b)| Change::Unbind(a, b)),
        any::<usize>().prop_map(Change::Output),
        Just(Change::NoOutput),
        any::<usize>().prop_map(Change::Name),
        any::<usize>().prop_map(Change::Entry),
    ]
}

/// `change` applied to `definition`, keeping it a definition a document can
/// hold: a new name is always one no instance has, and a parameter is bound
/// only where it is not already. Names wired to may be no instance at all.
#[cfg(test)]
fn apply(definition: &mut WorkflowDefinition, change: &Change) {
    const FRESH: [&str; 5] = ["n1", "n2", "new one", "a", "fetch"];
    const PARAMETERS: [&str; 3] = ["input", "hint", "extra"];

    let names: Vec<String> = definition
        .instances
        .iter()
        .map(|instance| instance.name.clone())
        .collect();
    let wired_to = |pick: usize| {
        names
            .get(pick % (names.len() + 1))
            .cloned()
            .unwrap_or_else(|| "elsewhere".to_owned())
    };
    let fresh = |pick: usize| {
        (0..FRESH.len())
            .map(|offset| FRESH[(pick + offset) % FRESH.len()])
            .find(|name| names.iter().all(|taken| taken != name))
            .map(str::to_owned)
    };
    let count = definition.instances.len();

    match *change {
        Change::Add(name, declared, wire) => {
            if let Some(name) = fresh(name) {
                let mut added = instance(&name, ["source", "sink", "ghost"][declared % 3], &[]);
                if wire % 2 == 0 {
                    added.bindings.push(Binding {
                        parameter: "input".to_owned(),
                        source: wired_to(wire / 2),
                    });
                }
                definition.instances.insert(wire % (count + 1), added);
            }
        }
        Change::Remove(at) if count > 0 => {
            definition.instances.remove(at % count);
        }
        Change::Rename(at, name) if count > 0 => {
            if let Some(name) = fresh(name) {
                definition.instances[at % count].name = name;
            }
        }
        Change::Bind(at, parameter, source) if count > 0 => {
            let parameter = PARAMETERS[parameter % PARAMETERS.len()];
            let bound = &mut definition.instances[at % count].bindings;
            if bound.iter().all(|binding| binding.parameter != parameter) {
                bound.push(Binding {
                    parameter: parameter.to_owned(),
                    source: wired_to(source),
                });
            }
        }
        Change::Repoint(at, binding, source) if count > 0 => {
            let bound = &mut definition.instances[at % count].bindings;
            if !bound.is_empty() {
                let binding = binding % bound.len();
                bound[binding].source = wired_to(source);
            }
        }
        Change::Unbind(at, binding) if count > 0 => {
            let bound = &mut definition.instances[at % count].bindings;
            if !bound.is_empty() {
                let binding = binding % bound.len();
                bound.remove(binding);
            }
        }
        Change::Output(output) => definition.designated_outputs = vec![wired_to(output)],
        Change::NoOutput => definition.designated_outputs.clear(),
        Change::Name(name) => definition.name = format!("renamed {}", name % 3),
        Change::Entry(at) if count > 0 => {
            let instance = &mut definition.instances[at % count];
            instance.entry = !instance.entry;
        }
        _ => {}
    }
}

/// Names the documents changed here draw from, some of which resolve to
/// nothing - the writer writes names, and whether they resolve is not its
/// business.
#[cfg(test)]
const WRITTEN: Pools = Pools {
    types: &["source", "sink", "ghost"],
    parameters: &["input", "hint"],
    strangers: &["elsewhere"],
};

#[cfg(test)]
proptest! {
    /// For any document the test writes, in any of the forms TOML allows, and
    /// any sequence of changes to the definition read from it, writing the
    /// changed definition into the document and reading it back gives the
    /// changed name, instances, bindings and output.
    ///
    /// The changes add, remove and rename whole instances, which are the edits
    /// that write or delete a table rather than a value.
    #[test]
    fn every_change_reads_back(
        written in any_written(WRITTEN),
        changes in vec(any_change(), 1..=6),
    ) {
        let catalogue = catalogue();
        let text = written.text();
        let Ok((mut definition, mut document)) = read_workflow("changed.toml", &text, &catalogue) else {
            return Err(TestCaseError::fail(format!("the document written does not read:\n{text}")));
        };
        for change in &changes {
            apply(&mut definition, change);
        }

        prop_assert_eq!(write_workflow(&mut document, &definition), Ok(()));
        prop_assert_eq!(by_name(&read_back(&document, &catalogue)), by_name(&definition));
    }
}

#[test]
fn removals_leave_nothing_behind() {
    let text = "\
name = \"pipeline\"
output = \"summarise\"   # the result

# fetches the diff
[instances.fetch]
node_type = \"source\"
position = [0, 0]

[instances.summarise]
node_type = \"sink\"

[instances.summarise.bindings]
input = \"fetch\"
hint = \"fetch\"   # a second opinion

[instances.publish]
node_type = \"sink\"
bindings = { input = \"summarise\" }
";
    let catalogue = catalogue();
    let (mut definition, mut document) =
        read_workflow("pipeline.toml", text, &catalogue).expect("it reads");

    // An instance removed, a binding removed, the output removed, and an
    // instance renamed - each what an edit rewriting only what the definition
    // still has leaves behind.
    definition
        .instances
        .retain(|instance| instance.name != "fetch");
    for instance in &mut definition.instances {
        instance
            .bindings
            .retain(|binding| binding.parameter != "hint");
        if instance.name == "publish" {
            instance.name = "announce".to_owned();
        }
    }
    definition.designated_outputs.clear();

    write_workflow(&mut document, &definition).expect("it writes");
    assert_eq!(
        by_name(&read_back(&document, &catalogue)),
        by_name(&definition)
    );

    // And no trace of any of them in the text: not the removed instance's
    // table, the key the model does not name on it or the comment above it, not
    // the removed binding or its comment, not the output, and not the old name.
    let written = document.to_string();
    for trace in [
        "[instances.fetch]",
        "position",
        "# fetches the diff",
        "hint",
        "# a second opinion",
        "output",
        "# the result",
        "publish",
    ] {
        assert!(!written.contains(trace), "{trace:?} is left in:\n{written}");
    }
}

#[test]
fn unchanged_document_is_byte_identical() {
    // Comments, keys the model does not name, blank lines, uneven spacing,
    // strings quoted as literals, and instances written three ways.
    let text = "\
# a workflow, annotated by hand
name    = 'review'   # a literal string, spaced out
output = \"summarise\"
version = 3
instances.fetch.node_type = 'source'
instances.fetch.entry = true


[editor]
zoom   = 2   # the editor's own

# the summariser
[instances.summarise]
node_type = \"sink\"   # declared elsewhere
position = [ 10, 20 ]

[instances.summarise.bindings]
input = 'fetch'   # wired by hand
hint= \"fetch\"

[instances.publish]
node_type = 'sink'
entry = false
bindings = {input='summarise'}
";

    // Once with the line endings every editor on Windows writes.
    for text in [text.to_owned(), text.replace('\n', "\r\n")] {
        let (definition, mut document) =
            read_workflow("review.toml", &text, &catalogue()).expect("it reads");
        write_workflow(&mut document, &definition).expect("it writes");
        assert_eq!(document.to_string(), text);
    }
}

/// A document a person and an editor have both annotated: comments above and
/// after, keys the model does not name on the document and on every instance.
/// Instance `i` binds `p0`, `p1`, ... to the names in `sources[i].0`, inline
/// or under a header of its own as `sources[i].1` says; `changed` puts one
/// binding's new source in place of its old one, and nothing else.
#[cfg(test)]
fn annotated(sources: &[(Vec<String>, bool)], changed: Option<(usize, usize, &str)>) -> String {
    let mut text = "\
# annotated by hand
name = \"w\"   # the workflow's own name
version = 2

[editor]
zoom = 2   # the editor's own
"
    .to_owned();

    for (at, (wired, inline)) in sources.iter().enumerate() {
        text += &format!(
            "\n# about i{at}\n[instances.i{at}]\nnode_type = \"sink\"   # declared elsewhere\nposition = [{at}, {at}]\n"
        );
        let wired: Vec<(String, &str)> = wired
            .iter()
            .enumerate()
            .map(|(binding, source)| {
                let source = match changed {
                    Some((instance, changed, new)) if (instance, changed) == (at, binding) => new,
                    _ => source.as_str(),
                };
                (format!("p{binding}"), source)
            })
            .collect();
        if *inline {
            let pairs: Vec<String> = wired
                .iter()
                .map(|(parameter, source)| format!("{parameter} = \"{source}\""))
                .collect();
            text += &format!("bindings = {{ {} }}   # wired inline\n", pairs.join(", "));
        } else {
            text += &format!("\n[instances.i{at}.bindings]\n");
            for (parameter, source) in &wired {
                text += &format!("{parameter} = \"{source}\"   # wired by hand\n");
            }
        }
    }
    text
}

#[cfg(test)]
proptest! {
    /// For any annotated document, repointing any one binding and writing the
    /// definition back changes that binding's value and nothing else in the
    /// text: the expected text is the same document written with the new
    /// source in that one place.
    ///
    /// The binding that changes is always under a header of its own, with a
    /// comment after its value - the one comment a writer changing exactly the
    /// right value can still lose - and on an instance carrying keys of its own,
    /// since a writer rewriting that instance's table whole keeps every other
    /// instance and loses exactly those.
    #[test]
    fn unread_keys_survive_a_change(
        shapes in vec((vec(0..5usize, 1..=3), any::<bool>()), 1..=4),
        target in any::<usize>(),
        binding in any::<usize>(),
        pick in any::<usize>(),
    ) {
        let names: Vec<String> = (0..shapes.len())
            .map(|at| format!("i{at}"))
            .chain(["elsewhere".to_owned()])
            .collect();
        let mut sources: Vec<(Vec<String>, bool)> = shapes
            .iter()
            .map(|(wired, inline)| {
                (wired.iter().map(|&pick| names[pick % names.len()].clone()).collect(), *inline)
            })
            .collect();

        let target = target % sources.len();
        sources[target].1 = false;
        let binding = binding % sources[target].0.len();
        let old = sources[target].0[binding].clone();
        let candidates: Vec<&String> = names.iter().filter(|name| **name != old).collect();
        let new = candidates[pick % candidates.len()].clone();

        let original = annotated(&sources, None);
        let expected = annotated(&sources, Some((target, binding, &new)));

        let catalogue = catalogue();
        let Ok((mut definition, mut document)) = read_workflow("annotated.toml", &original, &catalogue) else {
            return Err(TestCaseError::fail(format!("the document written does not read:\n{original}")));
        };
        definition.instances[target].bindings[binding].source = new;

        prop_assert_eq!(write_workflow(&mut document, &definition), Ok(()));
        prop_assert_eq!(document.to_string(), expected);
    }
}

#[test]
fn unwritable_shapes_all_named() {
    let text = "\
name = \"w\"

[instances.a]
node_type = \"source\"

[instances.b]
node_type = \"sink\"
bindings = { input = \"a\" }
";
    let (mut definition, mut document) =
        read_workflow("w.toml", text, &catalogue()).expect("it reads");
    let before = document.to_string();

    // All three shapes at once, because a writer naming the first it meets
    // passes a case holding one. And a change a writer could make before it
    // noticed, so that one refusing halfway through leaves a trace.
    definition.name = "renamed".to_owned();
    definition.instances.push(instance("a", "sink", &[]));
    definition.instances[1].bindings.push(Binding {
        parameter: "input".to_owned(),
        source: "b".to_owned(),
    });
    definition.designated_outputs = vec!["a".to_owned(), "b".to_owned()];

    let refused = write_workflow(&mut document, &definition).expect_err("it is refused");
    assert_eq!(
        refused.shapes(),
        [
            UnwritableShape::RepeatedInstance {
                instance: "a".to_owned(),
            },
            UnwritableShape::RepeatedBinding {
                instance: "b".to_owned(),
                parameter: "input".to_owned(),
            },
            UnwritableShape::SeveralOutputs {
                designated: vec!["a".to_owned(), "b".to_owned()],
            },
        ]
    );
    assert_eq!(document.to_string(), before, "the document was edited");
}

#[test]
fn wiring_defects_are_written() {
    let catalogue = TypeCatalogue::gather(vec![node_type_document(
        "types.toml",
        vec![
            node_type("source", &[], "note"),
            node_type("sink", &[("input", "note")], "note"),
            node_type("differ", &[], "diff"),
        ],
    )])
    .expect("three distinct types gather");
    let text = "\
name = \"w\"
output = \"b\"

[instances.a]
node_type = \"source\"

[instances.b]
node_type = \"sink\"
bindings = { input = \"a\" }
";
    let (mut definition, mut document) =
        read_workflow("w.toml", text, &catalogue).expect("it reads");

    // An instance of an unknown type, a binding to an instance that is not
    // there, a wire whose ends disagree, and no output.
    definition
        .instances
        .push(instance("ghost", "not-declared", &[]));
    definition
        .instances
        .push(instance("c", "sink", &[("input", "deleted")]));
    definition.instances.push(instance("d", "differ", &[]));
    definition.instances[1].bindings[0].source = "d".to_owned();
    definition.designated_outputs.clear();
    // The control's own control: the definition really is defective.
    assert_eq!(validate_wiring(&definition).len(), 4);

    write_workflow(&mut document, &definition).expect("a wiring defect is written");
    assert_eq!(
        by_name(&read_back(&document, &catalogue)),
        by_name(&definition)
    );
}
