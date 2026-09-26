//! The type catalogue: the declarations of every node type document it is
//! given, gathered into one set keyed by type name.

use std::collections::HashMap;
use std::fmt;

use crate::reader::NodeTypeDocument;
use crate::workflow::NodeType;

/// Every node type the documents it was gathered from declare, each under a
/// name no other declares. A catalogue neither searches for documents nor
/// watches them.
// @Documents given by the caller,TRACE_CATALOGUE_DOCUMENTS,trace,[],[DEC_TYPES_IN_OWN_DOCUMENTS]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeCatalogue {
    node_types: Vec<NodeType>,
}

impl TypeCatalogue {
    /// The declarations of every one of `documents`, or a refusal naming every
    /// type name more than one of them declares.
    ///
    /// Nothing is kept when a name repeats, identical declarations included, and
    /// every repeated name is named with every document declaring it.
    ///
    /// The declarations are held in the order they were given: document by
    /// document, and within one in the order it declares them.
    // @Every repeated type name gathered with every document declaring it,IMPL_CATALOGUE_GATHER,impl,[CREQ_CATALOGUE_DECLARED_ONCE]
    pub fn gather(
        documents: impl IntoIterator<Item = NodeTypeDocument>,
    ) -> Result<Self, RepeatedTypes> {
        let mut node_types = Vec::new();
        let mut declared_by: Vec<RepeatedType> = Vec::new();
        let mut position: HashMap<String, usize> = HashMap::new();

        for document in documents {
            for node_type in document.node_types {
                match position.get(&node_type.name) {
                    Some(&seen) => declared_by[seen].documents.push(document.document.clone()),
                    None => {
                        position.insert(node_type.name.clone(), declared_by.len());
                        declared_by.push(RepeatedType {
                            type_name: node_type.name.clone(),
                            documents: vec![document.document.clone()],
                        });
                    }
                }
                node_types.push(node_type);
            }
        }

        let repeated: Vec<RepeatedType> = declared_by
            .into_iter()
            .filter(|declared| declared.documents.len() > 1)
            .collect();
        if repeated.is_empty() {
            Ok(Self { node_types })
        } else {
            Err(RepeatedTypes { repeated })
        }
    }

    /// Every declaration gathered, in the order it was given.
    pub fn node_types(&self) -> &[NodeType] {
        &self.node_types
    }
}

/// Node type documents that declare one type name more than once between them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatedTypes {
    repeated: Vec<RepeatedType>,
}

impl RepeatedTypes {
    /// Every name declared more than once, in the order each was first declared.
    pub fn repeated(&self) -> &[RepeatedType] {
        &self.repeated
    }
}

/// One type name, and every document declaring it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepeatedType {
    type_name: String,
    documents: Vec<String>,
}

impl RepeatedType {
    /// The name declared more than once.
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Every document declaring it, in the order they were given - once for
    /// each declaration, so a document handed over twice is named twice.
    pub fn documents(&self) -> &[String] {
        &self.documents
    }
}

impl fmt::Display for RepeatedTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (position, repeated) in self.repeated.iter().enumerate() {
            if position > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{repeated}")?;
        }
        Ok(())
    }
}

impl fmt::Display for RepeatedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let documents: Vec<String> = self
            .documents
            .iter()
            .map(|document| format!("'{document}'"))
            .collect();
        write!(
            f,
            "the node type '{}' is declared by each of {}",
            self.type_name,
            documents.join(", ")
        )
    }
}

impl std::error::Error for RepeatedTypes {}

// --- test builders -----------------------------------------------------------

/// A document called `document`, declaring `node_types`, as the reader would
/// hand it over.
#[cfg(test)]
pub(crate) fn node_type_document(document: &str, node_types: Vec<NodeType>) -> NodeTypeDocument {
    NodeTypeDocument {
        document: document.to_owned(),
        node_types,
    }
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases, for the reason given in id.rs.

#[cfg(test)]
use crate::workflow::node_type;
#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::prelude::*;

/// A repetition as the refusal names it.
#[cfg(test)]
fn repeated(type_name: &str, documents: &[&str]) -> RepeatedType {
    RepeatedType {
        type_name: type_name.to_owned(),
        documents: documents
            .iter()
            .map(|&document| document.to_owned())
            .collect(),
    }
}

#[test]
fn repeated_name_names_every_document() {
    // Three documents, each declaring the type differently.
    let documents = vec![
        node_type_document(
            "a.toml",
            vec![
                node_type("review", &[("diff", "diff")], "summary"),
                node_type("fetch", &[], "diff"),
            ],
        ),
        node_type_document("b.toml", vec![node_type("review", &[], "summary")]),
        node_type_document(
            "c.toml",
            vec![
                node_type("publish", &[("text", "summary")], "note"),
                node_type("review", &[("notes", "note")], "note"),
            ],
        ),
    ];

    // The names declared once are not in the refusal: it names what is wrong,
    // and only that.
    assert_eq!(
        TypeCatalogue::gather(documents).map_err(|refused| refused.repeated),
        Err(vec![repeated("review", &["a.toml", "b.toml", "c.toml"])])
    );
}

#[test]
fn identical_copies_refused() {
    let declaration = node_type("review", &[("diff", "diff")], "summary");
    let documents = vec![
        node_type_document("mine.toml", vec![declaration.clone()]),
        node_type_document("theirs.toml", vec![declaration]),
    ];

    // Harmless today, and the way two definitions of one type start to drift.
    assert_eq!(
        TypeCatalogue::gather(documents).map_err(|refused| refused.repeated),
        Err(vec![repeated("review", &["mine.toml", "theirs.toml"])])
    );
}

#[cfg(test)]
proptest! {
    /// Documents each declaring a subset of a pool of four names, so that
    /// several names repeat at once in most sets: a catalogue stopping at the
    /// first repetition passes every set with only one.
    ///
    /// What should be refused is worked out by counting, for each name in the
    /// pool, the documents whose subset holds it - a different computation from
    /// the walk under test. Its order is worked out apart from the walk too:
    /// nothing requires one, but `RepeatedTypes::repeated` says it is the order
    /// of first declaration, and a documented order nothing checks is a claim.
    #[test]
    fn every_repeated_name_reported(subsets in vec(any::<[bool; 4]>(), 1..=5)) {
        const POOL: [&str; 4] = ["fetch", "review", "publish", "notify"];

        let documents: Vec<NodeTypeDocument> = subsets
            .iter()
            .enumerate()
            .map(|(position, subset)| {
                let declared = POOL
                    .iter()
                    .zip(subset)
                    .filter(|&(_, &declares)| declares)
                    .map(|(&name, _)| node_type(name, &[], "note"))
                    .collect();
                node_type_document(&format!("d{position}.toml"), declared)
            })
            .collect();

        // Each name with the first document declaring it and its place in the
        // pool, which within a document is the order it declares names in - so
        // sorting by the two is the order of first declaration.
        let mut expected: Vec<(usize, usize, RepeatedType)> = Vec::new();
        for (index, name) in POOL.iter().enumerate() {
            let declaring: Vec<usize> = subsets
                .iter()
                .enumerate()
                .filter(|(_, subset)| subset[index])
                .map(|(position, _)| position)
                .collect();
            if declaring.len() > 1 {
                let documents = declaring.iter().map(|position| format!("d{position}.toml")).collect();
                expected.push((declaring[0], index, RepeatedType { type_name: (*name).to_owned(), documents }));
            }
        }
        expected.sort_by_key(|&(first, index, _)| (first, index));
        let expected: Vec<RepeatedType> = expected.into_iter().map(|(_, _, repeated)| repeated).collect();

        let every_declaration: Vec<NodeType> = documents
            .iter()
            .flat_map(|document| document.node_types.clone())
            .collect();
        match TypeCatalogue::gather(documents) {
            Ok(catalogue) => {
                prop_assert_eq!(expected, Vec::new(), "a repetition was held");
                prop_assert_eq!(catalogue.node_types, every_declaration);
            }
            Err(refused) => prop_assert_eq!(refused.repeated, expected),
        }
    }
}

#[test]
fn distinct_declarations_held() {
    // Two documents declaring different types, one of them declaring several,
    // and types that no workflow names - there is no workflow here at all.
    let fetch = node_type("fetch", &[], "diff");
    let review = node_type("review", &[("diff", "diff"), ("hint", "note")], "summary")
        .with_globals(&["policy"]);
    let publish = node_type("publish", &[("text", "summary")], "note");

    let gathered = TypeCatalogue::gather(vec![
        node_type_document("sources.toml", vec![fetch.clone(), review.clone()]),
        node_type_document("sinks.toml", vec![publish.clone()]),
    ]);

    assert_eq!(
        gathered.map(|catalogue| catalogue.node_types),
        Ok(vec![fetch, review, publish])
    );
}
