//! Node behaviour for Agconflo, run as Lua 5.5 scripts.
//!
//! `agconflo-core` decides what may activate and what it is given, and hands
//! each activation to its caller to perform. This crate is such a caller: it
//! performs an activation by running the script its node type was given, and
//! tells the run what came of it.
//!
//! A script is handed over as text, with the name of the document it came from;
//! nothing here opens a file. It runs as the body of
//! its node's behaviour, in a Lua state made for that one activation, with
//! nothing to reach but its activation, the context API and the models its
//! caller mapped, and under a limit on the instructions it executes, the memory
//! it allocates and the model calls it makes.
//!
//! A script's model may call the node types its instance declares calls to. Each
//! call is performed as an activation of the run, by the called node type's own
//! script or by a person, and the model is asked again with the call's output.
//!
//! A node type can instead be named as performed by a person. A scripted run
//! then stops at each of its activations and hands it to its caller, and goes
//! on from its record once the caller supplies the person's answer as text.
//!
//! A comment starting with `// @` is a code marker, read into the requirements
//! graph by `docs/code/agconflo-lua.rst`, which gives its shapes.

// @A caller of the core run in its own crate,TRACE_LIB_CALLER,trace,[],[DEC_RUN_IS_DRIVEN, DEC_BEHAVIOUR_OWN_CRATE, DEC_SCRIPTS_FROM_CALLER]
mod behaviours;
mod host;
mod models;
mod scripted;

pub use behaviours::{BehaviourFault, Behaviours};
pub use host::{Limits, ModelCallFault, ScriptFailure};
pub use models::{ModelFailure, Roster};
pub use scripted::{Outcome, ScriptedRefusal, answer_scripted, resume_scripted, run_scripted};
