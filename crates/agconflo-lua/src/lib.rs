//! Node behaviour for Agconflo, run as Lua 5.5 scripts.
//!
//! `agconflo-core` decides what may activate and what it is given, and hands
//! each activation to its caller to perform (`DEC_RUN_IS_DRIVEN`). This crate is
//! such a caller: it performs an activation by running the script its node type
//! was given, and tells the run what came of it. The core gains no dependency on
//! a script language by it (`DEC_BEHAVIOUR_OWN_CRATE`).
//!
//! A script is handed over as text, with the name of the document it came from
//! (`DEC_SCRIPTS_FROM_CALLER`); nothing here opens a file. It runs as the body of
//! its node's behaviour, in a Lua state made for that one activation, with
//! nothing to reach but its activation, the context API and the models its
//! caller mapped, and under a limit on the instructions it executes, the memory
//! it allocates and the model calls it makes.
//!
//! A node type can instead be named as performed by a person. A scripted run
//! then stops at each of its activations and hands it to its caller, and goes
//! on from its record once the caller supplies the person's answer as text.
//!
//! A comment of the form `// @<title>,<IMPL id>,impl,[<requirement ids>]` is a
//! trace marker, read into the requirements graph by
//! `docs/code/agconflo-lua.rst`; see `agconflo-core` for what that checks.

mod behaviours;
mod host;
mod models;
mod scripted;

pub use behaviours::{BehaviourFault, Behaviours};
pub use host::{Limits, ScriptFailure};
pub use models::{ModelFailure, Roster};
pub use scripted::{Outcome, ScriptedRefusal, answer_scripted, resume_scripted, run_scripted};
