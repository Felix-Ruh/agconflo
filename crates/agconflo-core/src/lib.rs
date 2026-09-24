//! The engine core of Agconflo: contexts, their identity and their lineage,
//! whether a workflow's wiring is sound before any of it runs, workflows read
//! from TOML documents and written back into them, and a run over one - driven
//! by its caller, which performs each activation and reports what came of it.
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

mod catalogue;
mod context;
mod defect;
mod id;
mod lineage;
mod reader;
mod record;
mod run;
mod scheduler;
mod wiring;
mod workflow;
mod writer;

#[cfg(test)]
mod compile_fail;

pub use catalogue::{RepeatedType, RepeatedTypes, TypeCatalogue};
pub use context::{Context, ContextType, InvalidTypeName};
pub use defect::WiringDefect;
pub use id::{ContextId, IdSource, SourceExhausted};
pub use reader::{
    FaultKind, NodeTypeDocument, ReadFault, WorkflowDocument, read_node_types, read_workflow,
};
pub use record::{Divergence, RecordFault, RecordFaultKind, ResumeRefusal};
pub use run::{
    Arguments, NothingOutstanding, OutputRefusal, Run, RunEnding, SignatureFault, StartRefusal,
    Step,
};
pub use scheduler::Activation;
pub use wiring::validate_wiring;
pub use workflow::{Binding, NodeInstance, NodeType, Parameter, WorkflowDefinition};
pub use writer::{UnwritableDefinition, UnwritableShape, write_workflow};
