=========================
Decisions about workflows
=========================

What a workflow is made of, and therefore what makes one invalid. These are the
choices the engine's first slice is written against: the requirement that a
broken graph never starts cannot be specified until it is settled what a graph
is and what "broken" means about one.

All of them were taken in the design that started this project, before any of it
was built, and they are migrated here now because the feature that needs them is
being written. That is the same order the context decisions were migrated in, and
the same rule applies: re-opening one means superseding it rather than quietly
disagreeing with it.

None of them rests on a measurement, and none carries evidence. They are
judgements about a model, and the honest record of a judgement is its reasoning
and what it turned down.

.. dec:: Node types and workflows are separate layers
   :id: DEC_TWO_LAYERS
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall define each node type once and build a workflow from instances of those types.

   A node type says what a kind of node needs and produces; a workflow says which
   instances exist and how they are wired. The alternative - a workflow of
   self-describing nodes, each carrying its own parameter list - was not taken,
   because then nothing states what a node of that kind ought to look like and a
   wiring mistake has nothing to be a mistake against.

   This is what makes validation possible at all: an instance is checked against
   its type, and the type is the shared statement of intent. It also keeps a
   workflow small enough to read, since the part that repeats lives in one place.

   The cost is an indirection: a workflow cannot be understood without the types
   it names, so a missing type is itself a defect a validator has to report.

.. dec:: A node type declares what it consumes
   :id: DEC_DECLARED_PARAMETERS
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall have every node type declare the parameters it requires, the parameters it accepts and the global context types it reads.

   Three lists rather than one, and each is load-bearing. Required parameters are
   what a validator checks a workflow against. Optional parameters are what lets a
   node be useful with less than everything wired. Requested global types are the
   one sanctioned way a node reads something that was not wired to it, and
   declaring them keeps that exception visible rather than ambient.

   The list is ordered and each entry is typed, which is what lets a node assemble
   its own inputs - the alternative, ordering inputs by edge, was rejected because
   there is no natural order across edges arriving from different nodes.

   A node that could accept anything would make wiring unfalsifiable, so the
   declaration is what turns "a node sees exactly what was wired to it" into
   something a check can enforce before a run.

.. dec:: A binding names a node's output, not a context
   :id: DEC_BINDING_BY_PORT
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall bind a parameter to the output of one named node instance rather than to a context identifier.

   A workflow is a definition instantiated many times, so the contexts it will
   carry do not exist when it is written. Binding to a context identifier would
   tie a definition to one run's values, which is the shape this model exists to
   avoid: one workflow per unit of work rather than one workflow invoked with
   arguments.

   What a binding names instead is a position in the graph - that instance's one
   output - which is stable across every instantiation and checkable without
   running anything.

   It follows that a parameter binds to exactly one output while an output may be
   bound by many parameters, and that a validator can resolve every binding
   statically, since both ends are named in the definition.

.. dec:: A binding is also a control edge
   :id: DEC_IMPLIED_CONTROL_EDGES
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall treat a binding into a node as a control edge into that node.

   A node cannot run before the values it consumes exist, so the dependency is
   already stated by the binding. Requiring the author to draw the control edge as
   well would mean saying the same thing twice, and the two copies would
   eventually disagree.

   Explicit control edges are kept for what a binding cannot express: an entry
   node with no data dependencies, an ordering between nodes that act on the world
   rather than on contexts, a router gating one branch over another, and a
   back edge closing a loop.

   The consequence for validation is that the control graph is derived rather than
   read, and a node that no derived or explicit edge can reach is unreachable -
   which a validator can see, and which no author intended.

.. dec:: Routing a context is not scheduling a node
   :id: DEC_ROUTING_SEPARATE_FROM_CONTROL
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall route a context independently of the order in which nodes run.

   A context may be carried to a distant part of the graph, past nodes that never
   read it, without that saying anything about when those nodes run. Folding the
   two into one graph would force a choice between a context taking a path nobody
   wants and a node running for no reason other than being on that path.

   Keeping them apart is also what makes the provenance claim precise: what a node
   saw is decided by its bindings alone, never by what happened to pass nearby.

   For a validator this means two questions rather than one, asked of the same
   definition: is every parameter bound, and can every node be reached.

.. dec:: A cycle is a legal workflow
   :id: DEC_BACK_EDGES_ALLOWED
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall accept a workflow whose edges form a cycle.

   Retrying a step until a test passes is an ordinary pipeline, and a model that
   refused cycles would force every loop to be expressed outside the workflow,
   where the provenance record does not reach.

   An explicit loop construct was the alternative and remains a recorded fallback
   rather than a rejected idea: a loop scope would own the iteration counter,
   which is most of the problem that tagging each activation otherwise has to
   solve. It was not taken because a scope constrains where a loop may be drawn,
   and the graphs this engine is for are wired rather than nested.

   Two consequences are not optional, and both are requirements elsewhere: a run
   that never terminates has to be stopped by a budget, and a run that cannot
   proceed has to be reported rather than waited on. For a validator the
   consequence is narrower: a cycle is not an error, so reachability is asked from
   the entry nodes rather than by looking for a topological order.

.. dec:: A workflow has a signature
   :id: DEC_WORKFLOW_SIGNATURE
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall give a workflow a signature of typed entry parameters and exactly one designated output.

   A workflow is invocable as a node, and a caller can only wire to something
   whose shape is declared. Entry nodes are that parameter list; the designated
   output is the single terminal result the caller binds to.

   The output half was new when this was decided: the model had entry nodes and no
   exit concept, so a workflow had no answer to "what did it produce". Designating
   one makes a workflow's shape the same shape as a node type's, which is what
   lets one be wired into another and checked by the same rules - including the
   rule that a node produces exactly one thing.

   A workflow with no designated output, or with more than one, is therefore
   invalid rather than merely unusual, and that is a check on the definition
   rather than on a run.
