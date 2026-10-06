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

.. stkh_req:: A workflow's scripts share code written once
   :id: STKH_SHARED_SCRIPT_CODE
   :stakeholder: user
   :statement: Agconflo shall let the scripts of a workflow share code that is written once.

   The node types of one workflow do related work, and their scripts need the
   same helpers: checking an answer and asking again, reading a model's reply
   into its fields, saying what was wrong. Written into each script, every
   copy has to be found and changed alike, and the copies drift. The first
   development workflow, proposed in #71, worked around that by generating
   its scripts from a program in another language, which put the source of
   its behaviour outside its own documents and a second toolchain into the
   repository.

   Shared code is part of the behaviour, as a script is. It is supplied with
   the workflow's documents and changes as they do, which is what
   ``STKH_LIVE_BEHAVIOUR`` asks of a script. It is not an input: what a node
   is given is still what was wired to it (``STKH_EXPLICIT_CONTEXT``), and
   code shared by two scripts gives neither of them anything to read that the
   other's activation could change.

   It names no mechanism. How a script reaches code it shares is for the
   decisions below it to say.

   It is separate from ``STKH_LIVE_BEHAVIOUR`` because either can hold without
   the other: scripts changed without a rebuild can each carry their own
   copy, and shared code could be compiled into the engine.

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

.. stkh_req:: The engine explains itself to a model
   :id: STKH_REFLECTION
   :stakeholder: agent
   :statement: Agconflo shall give a model one defined interface through which it can discover, inspect, call and edit the workflows and node types available to it.

   A model that can yield to a workflow needs to know which workflows exist,
   what each takes and returns, and how to call it; a model that helps build
   workflows needs to read and change them. Both are the same question asked of
   the same components, and answering it twice, differently, is how the two
   drift apart.

   So the framework is meant to be reflective: its own components are what a
   model reads and acts on, through one interface that is defined and
   documented as such, rather than whatever each feature happens to expose.
   Tool calls are how a model uses it.

   This widens ``STKH_MACHINE_AUTHORING``, which asks that an agent can create
   and modify workflows through tool calls, to discovering, inspecting and
   calling as well, and asks that all of it be one interface. It is recorded
   now so that the yield and the tools above are built against it, though what
   it covers is expected to be built later.
