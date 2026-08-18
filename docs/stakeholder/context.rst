======================
Context and provenance
======================

What a node is given, what it may do with it, and what can later be said about
where it came from. These are the goals the engine's core value type exists to
serve, and the first feature is derived from them.

.. stkh_req:: A node sees only what was wired to it
   :id: STKH_EXPLICIT_CONTEXT
   :stakeholder: user
   :statement: Agconflo shall give a node exactly the contexts wired to it.

   Without this property the project is a worse LangGraph. The point of a
   context-centric engine is that "what was in this call's context window, and
   where did every byte come from?" has an exact answer, so any feature that
   makes a node's visible context implicit or unauditable needs a very good
   reason. Global contexts are the one sanctioned exception, and are recorded in
   the run log as genuine inputs precisely so that they do not become one.

.. stkh_req:: Every byte is traceable to where it came from
   :id: STKH_PROVENANCE
   :stakeholder: user
   :statement: Agconflo shall record which context each byte of a node's input came from.

   The second half of the question above, and the half that survives after the
   run is over. Provenance is the product here rather than a debugging aid: it is
   what decides whether an output can be trusted, and what a later run is
   compared against.

   The need was demonstrated rather than assumed. The design session that
   produced this project hit its own context limit, and what filled the window
   was reasoning traces and verbatim tool output from early messages that no
   longer mattered, while the conclusions worth keeping were small. Knowing which
   bytes are which is the precondition for discarding the right ones.

.. stkh_req:: A context does not change once it exists
   :id: STKH_IMMUTABLE_CONTEXT
   :stakeholder: user
   :statement: Agconflo shall not alter a context after it has been created.

   Distinct from provenance, which says where a value came from: this says that
   it is still the same value. Without it a run log records what a context was
   called rather than what it held, and one node can change what another has
   already consumed.

   It is also what makes a context safe to pass by reference rather than by copy,
   which everything built on composition depends on.
