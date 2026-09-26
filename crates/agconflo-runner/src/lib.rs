//! Runs an Agconflo workflow from the documents that describe it.
//!
//! A manifest names a workflow document, its node type documents, a script
//! for each node type a script performs, the node types a person performs,
//! the node types a tool performs, and the run's budget and limits. The project reader reads all of it into
//! what a scripted run is started with. A model mapping names the model each
//! role is played by, and the model map reads it into the roster models are
//! called through. A grants file names what a run's tools may do, and the
//! grants reader reads it. The sandbox is the container a run's tool steps are
//! performed in, and the tool performer performs one step's action in it. The record keeper holds a run's record file for one run at
//! a time and replaces it with each record the run hands over. The runner
//! starts, resumes, answers and checks a run from all of them, performing its
//! tool steps as they come.

mod grants;
mod keeper;
mod model_map;
mod performer;
mod project;
mod runner;
mod sandbox;
mod text;

#[cfg(test)]
mod testing;

pub use grants::{Action, CommandLimits, Folder, FolderFault, Grants, GrantsFault, read_grants};
pub use keeper::{KeeperRefusal, RecordKeeper, Unkept};
pub use model_map::{ModelMap, ModelsFault, read_models};
pub use performer::{Container, parameters, perform};
pub use project::{Project, ProjectFault, Script, Tool, read_project};
pub use runner::{
    Argument, Finding, Refusal, Sources, Stopped, ToolFault, answer, check, resume, start,
};
pub use sandbox::{Done, EngineFailure, Environment, NotReady, SHARED, Sandbox};
pub use text::{FileFault, KeyFault, Place};
