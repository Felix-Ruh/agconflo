===========================
Authoring and extensibility
===========================

What someone building on Agconflo can define, change and automate. These goals
constrain the authoring surface rather than the engine's internals; the choices
about how each is realised are recorded as decisions instead.

.. stkh_req:: A node produces one thing
   :id: STKH_ONE_OUTPUT
   :stakeholder: user
   :statement: Agconflo shall restrict a node to exactly one output.

   "Decide where this goes" and "produce new content" are two separate tasks, and
   an LLM asked to do both in one call does both measurably worse. Keeping them
   apart is what makes a router a router and a transform a transform.

   It also keeps every result addressable: one output means one context with one
   identity, which can be routed to many consumers, inspected, and logged like
   any other. Several outgoing control edges are a fan-out rather than several
   outputs - no decision is computed there, so the rule holds unchanged.

.. stkh_req:: Behaviour changes without a rebuild
   :id: STKH_LIVE_BEHAVIOUR
   :stakeholder: user
   :statement: Agconflo shall let node behaviour be changed without recompiling the engine.

   Node types are the unit people extend, and requiring a rebuild to change one
   puts authoring out of reach of anyone without the toolchain while making
   experimentation expensive for everyone else. The engine is written in Rust;
   what a node does need not be.

.. stkh_req:: Topology is data, not script
   :id: STKH_TOPOLOGY_AS_DATA
   :stakeholder: user
   :statement: Agconflo shall store workflow topology as data that is validated without executing it.

   Three things follow from data rather than script, and none of them survive the
   alternative: a graph can be checked before anything runs, an editor can
   round-trip it without losing what it did not understand, and a machine can
   change it structurally rather than textually.

   It is also less implementation work rather than more, since loading needs no
   execution and serialisation does most of it.

.. stkh_req:: An agent can author a workflow
   :id: STKH_MACHINE_AUTHORING
   :stakeholder: agent
   :statement: Agconflo shall let an agent create and modify workflows through tool calls.

   Agents helping build agents is a first-class use case here rather than an
   afterthought, and it is the natural completion of a project whose target is to
   run its own development pipeline.

   Treating it as a requirement rather than a later product constrains the
   workflow model now: the format has to be machine-editable and checkable
   without executing anything, and errors have to be reported well enough that a
   model can correct itself from them.
