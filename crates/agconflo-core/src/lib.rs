//! The engine core of Agconflo: contexts, their identity and their lineage,
//! whether a workflow's wiring is sound before any of it runs, workflows read
//! from TOML documents and written back into them, and a run over one - driven
//! by its caller, which performs each activation and reports what came of it.
//!
//! Everything public is re-exported here from private modules.
//!
//! A comment starting with `// @` is a code marker, read into the requirements
//! graph by `docs/code/agconflo-core.rst`, which gives its shapes.

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
    Arguments, Call, CallFault, CallRefusal, Exchange, ExchangeRefusal, NothingOutstanding,
    OutputRefusal, Run, RunEnding, SignatureFault, StartRefusal, Step,
};
pub use scheduler::Activation;
pub use wiring::validate_wiring;
pub use workflow::{Binding, NodeInstance, NodeType, Parameter, WorkflowDefinition};
pub use writer::{UnwritableDefinition, UnwritableShape, write_workflow};
