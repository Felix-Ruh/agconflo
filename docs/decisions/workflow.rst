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

The three before the last were taken by the maintainer on 2026-09-26, and
supersede four of the first: a workflow is one graph whose edges carry contexts,
it repeats part of itself only by walking those edges again, and a node type
declares no optional parameter. The one after them follows from the last of
those: a node type document still declaring optional parameters is refused
rather than read without them. The last was the maintainer's too: a list is
handled item by item inside one node.

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
   :dec_status: superseded
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
   :dec_status: superseded
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
   read, so which nodes are reachable is a question the definition answers by
   itself. What follows from an unreachable node - refuse it, report it, ignore it
   - is a requirement's business rather than this decision's.

.. dec:: Routing a context is not scheduling a node
   :id: DEC_ROUTING_SEPARATE_FROM_CONTROL
   :dec_status: superseded
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
   :dec_status: superseded
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
   :dec_status: superseded
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

.. dec:: A workflow's signature is what nothing in it binds
   :id: DEC_SIGNATURE_IS_WHAT_NOTHING_BINDS
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supersedes: DEC_WORKFLOW_SIGNATURE
   :statement: Agconflo shall take a workflow's signature to be the typed parameters of its nodes that nothing binds and exactly one designated output.

   A caller can only wire to something whose shape is known, and a workflow's
   shape is still a node type's: typed parameters and one output
   (``DEC_WORKFLOW_SIGNATURE``). What changes is where the parameters come
   from. They were the parameters of instances marked as entries; they are
   now every parameter the graph leaves unbound, read from the wiring rather
   than declared beside it (``STKH_RUN_FROM_ANY_PARAMETER``). Each is a pair
   of an instance and a parameter (``DEC_ARGUMENTS_PER_PARAMETER``).

   Marking inputs, on the instance or on the parameter, was the alternative.
   It catches a forgotten binding as a defect of the graph, where reading the
   signature from the wiring catches it only when a run starts
   (``DEC_RUN_REFUSED_UNLESS_EVERY_INPUT_GIVEN``). The maintainer chose the
   reading for being the more general: a workflow can then be entered at
   whichever of its nodes the wiring leaves open, and a flag has to be kept in
   step with every change to the wiring.

   The designated output is unchanged, and so is its check: a workflow with no
   designated output, or with more than one, is invalid.

.. dec:: A workflow repeats part of itself by walking its edges again
   :id: DEC_REPETITION_BY_EDGES
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supersedes: DEC_BACK_EDGES_ALLOWED
   :statement: Agconflo shall accept a workflow whose edges form a cycle and repeat part of a workflow only by walking its edges again, with no loop construct.

   What ``DEC_BACK_EDGES_ALLOWED`` decided stands: retrying a step until a test
   passes is an ordinary pipeline, a cycle is legal, and reachability is asked
   from the entry nodes. What it kept as a fallback does not: there is no loop,
   no scope, and no iteration counter. An edge pointing back to a node that has
   run is an edge like any other, and walking it again is all repetition is.
   The counter a loop scope would have owned is the generation each edge keeps
   (``DEC_EDGE_GENERATIONS``).

   Drawn, such a workflow shows its data flow and nothing else: every edge is a
   context's path. The maintainer chose it over the loop scope for the reason
   the superseded decision gave for not taking one: a scope constrains where a
   loop may be drawn, and these graphs are wired rather than nested.

.. dec:: A workflow is one graph of edges carrying contexts, and a router routes along them
   :id: DEC_ONE_GRAPH
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supersedes: DEC_IMPLIED_CONTROL_EDGES, DEC_ROUTING_SEPARATE_FROM_CONTROL
   :statement: Agconflo shall keep a workflow as one graph whose edges carry contexts, and route a run by the edges a router walks the contexts it was given along, with no separate graph of control edges.

   ``DEC_IMPLIED_CONTROL_EDGES`` kept explicit control edges for four cases and
   ``DEC_ROUTING_SEPARATE_FROM_CONTROL`` kept the path a context takes apart
   from the order nodes run in: two graphs over one definition. There is now
   one. A node runs when its edges hold what it needs (``DEC_EDGE_GENERATIONS``),
   and that is the whole of when.

   The four cases each need no graph of their own. An entry node is marked as
   one. An ordering between nodes that act on the world is an edge: the node
   that must come second is given the first one's output. A back edge is an
   edge walked again (``DEC_REPETITION_BY_EDGES``). And a router gates a branch
   by which edges it walks.

   A router creates no content. Its script decides which of the edges after it
   to walk, and the contexts walked along them are the ones it was given,
   unchanged, so a routed context's provenance is its producer's
   (``STKH_ONE_OUTPUT``: deciding where something goes is apart from producing
   it). A branch no router walks receives no generation, and its nodes do not
   run; a run already completes whatever instances have not
   (``DEC_COMPLETION_IS_DESIGNATED_OUTPUT``).

.. dec:: Every parameter a node type declares is required, and an empty one is an empty context
   :id: DEC_EVERY_INPUT_REQUIRED
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supersedes: DEC_DECLARED_PARAMETERS, DEC_BINDING_IS_AWAITED
   :statement: Agconflo shall have every node type declare the parameters it requires and the global context types it reads and no optional parameter, a parameter with nothing to carry being given an empty context.

   What ``DEC_DECLARED_PARAMETERS`` decided stands but for its middle list: each
   parameter is typed and ordered, and requested global types stay the one
   sanctioned way to read what was not wired. Optional parameters go. Every
   context a node type names is one it needs, and a producer with nothing to
   say gives an empty context - the text ``""`` - which is explicit and
   recorded, where an unbound parameter said nothing either way.

   It also settles what ``DEC_BINDING_IS_AWAITED`` had to argue for: with
   nothing optional, every edge into a node is waited for without an
   exception to reason about (``DEC_EDGE_GENERATIONS``). Optional parameters
   were removed from the code, the readers and the requirements' bodies in a
   change of their own, which also refuses a document still declaring them
   (``DEC_OPTIONAL_PARAMETERS_REFUSED``).

.. dec:: A node type still declaring optional parameters is refused
   :id: DEC_OPTIONAL_PARAMETERS_REFUSED
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall refuse a node type document that declares an optional list on a node type, at that list's key.

   With no optional parameters (``DEC_EVERY_INPUT_REQUIRED``), the ``optional``
   key means nothing. The topology reader reads past keys it does not know, so
   that a document keeps what a later version or another tool wrote there, and
   read that way a type still declaring ``optional = { hint = "note" }`` would
   lose ``hint`` without a word: the wiring would still check, and the script
   would meet a name with no value. Refusing the key at its place tells the
   author which declaration to rewrite, and how.

   Reading an optional list as required parameters was the alternative, and
   would change what an old document means without its author seeing it:
   every instance binding none of those parameters would stop starting, for a
   reason the document no longer shows. Ignoring the key was the other, and is
   the silent loss above.

.. dec:: A list is handled item by item inside one node
   :id: DEC_LIST_HANDLED_IN_ONE_NODE
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall have a node handle each item of a list it is given within its own activation, with no edge that walks a list's items as passes of their own.

   Taken by the maintainer, when a workflow's research step had to look into
   each of several decisions apart and a later step needed all the results.
   A node given the list loops over it in its script, asking a model once per
   item with that item and whatever else it needs, and its one output is the
   list of results. The record keeps each call's window and answer, so a run
   interrupted part way resumes without asking any item again
   (``DEC_RECORD_AFTER_EACH_ANSWER``), and a call is sent exactly the contexts
   its script composed for it (``FEAT_MODEL_WINDOW_IS_THE_PROMPT``).

   An edge walking each item as a pass of its own, and one gathering the
   passes back into a list, was the alternative: the items would be nodes a
   router could send apart, at the cost of two new kinds of edge and of
   pairing each gathered item with the list it came from. It is left until an
   item needs a route of its own. What it costs now is that one activation
   spends the call limit for every item.
