//! The engine core of Agconflo: contexts, their identity and their lineage.
//!
//! Everything public is re-exported here from private modules. The modules are
//! named to fit the test case ids in `docs/tests/`, which is a reason to keep
//! them out of the paths callers write.
//!
//! A comment of the form `// @<title>,<IMPL id>,impl,[<requirement ids>]` is a
//! trace marker: ubc reads it into the requirements graph as the place where
//! those component requirements are met (`docs/code/agconflo-core.rst`). It is
//! checked with the rest of the graph, so a marker naming a requirement that
//! does not exist fails the documentation check, in the commit hook and in CI.

mod context;
mod id;
mod lineage;

#[cfg(test)]
mod compile_fail;

pub use context::{Context, ContextType, InvalidTypeName};
pub use id::{ContextId, IdSource, SourceExhausted};
