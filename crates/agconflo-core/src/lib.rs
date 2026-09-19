//! The engine core of Agconflo: contexts, their identity and their lineage.
//!
//! Everything public is re-exported here from private modules. The modules are
//! named to fit the test case ids in `docs/tests/`, which is a reason to keep
//! them out of the paths callers write.

mod context;
mod id;
mod lineage;

#[cfg(test)]
mod compile_fail;

pub use context::{Context, ContextType, InvalidTypeName};
pub use id::{ContextId, IdSource, SourceExhausted};
