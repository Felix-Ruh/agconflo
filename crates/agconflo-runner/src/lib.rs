//! Runs an Agconflo workflow from the documents that describe it.
//!
//! A manifest names a workflow document, its node type documents, a script
//! for each node type a script performs, the node types a person performs,
//! and the run's budget and limits. The project reader reads all of it into
//! what a scripted run is started with.

mod project;
mod text;

#[cfg(test)]
mod testing;

pub use project::{Project, ProjectFault, Script, read_project};
pub use text::{FileFault, KeyFault, Place};
