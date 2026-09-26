=========================================
Components of a workflow repeating itself
=========================================

The requirements ``ARCH_REPETITION`` allocates to the three components it
uses, each defined where its first feature put it: the topology reader in
``components/topology``, the run scheduler and the workflow run in
``components/run``. Each title is the grammatical subject of the requirements
allocated to it, and the gate in ``scripts/gates`` refuses a component
requirement whose subject is anything else.

An edge is a binding: one instance's parameter, and what fills it. Each edge
holds the contexts walked along it in the order they were walked, and an
instance takes from each the earliest it has not taken
(``DEC_EDGE_GENERATIONS``). An output stands when its node type declares it
(``DEC_STANDING_OUTPUTS``) or its instance cannot run again
(``DEC_ONCE_RUN_OUTPUTS_STAND``).

.. comp_req:: An activation takes the earliest new context from each edge
   :id: CREQ_SCHEDULER_TAKES_EARLIEST
   :derived_from: FEAT_REPEAT_ON_NEW_CONTEXTS
   :allocated_to: COMP_RUN_SCHEDULER
   :ears_pattern: ubiquitous
   :statement: Run scheduler shall give an instance's activation from each edge into it the earliest context that edge holds that the instance has not taken, or the context that stands on it when it holds none.

   Failure modes:

   - **The latest context taken**, and a pass whose draft is paired with the
     review of a later draft.
   - **One pass's contexts taken from edges that moved at different rates**,
     each edge's latest rather than each edge's next.
   - **A context taken twice**, and a node answering the same pass again.

.. comp_req:: An instance that has run is offered again on something new
   :id: CREQ_SCHEDULER_OFFERS_AGAIN
   :derived_from: FEAT_REPEAT_ON_NEW_CONTEXTS
   :allocated_to: COMP_RUN_SCHEDULER
   :ears_pattern: event
   :statement: When every edge into an instance that has run holds a context that stands or one it has not taken and at least one holds one it has not taken, Run scheduler shall offer that instance for activation.

   ``DEC_RUN_AGAIN_ON_SOMETHING_NEW``. An instance with no edge into it, or
   whose every edge stands, is offered once, by ``CREQ_SCHEDULER_READY_WHEN_BOUND``,
   and never again.

   Failure modes:

   - **An instance offered once and never again**, which is what the engine
     did, and a repetition that stops after its first pass.
   - **An instance offered on its standing inputs alone**, and a node running
     for ever on the same contexts until the budget stops it.
   - **An instance offered while an edge into it holds nothing new or
     standing**, and given an input it already answered.

.. comp_req:: An output goes along every edge out of its instance
   :id: CREQ_RUN_WALKS_EVERY_EDGE
   :derived_from: FEAT_REPEAT_ON_NEW_CONTEXTS
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: event
   :statement: When the caller reports the output of an instance whose node type does not route, Workflow run shall walk it along every edge out of that instance as the next context that edge holds.

   Failure modes:

   - **The output replacing what an edge holds**, and a pass not yet taken
     lost to the next.
   - **One edge out of several walked**, and a consumer waiting for a pass
     that went elsewhere.

.. comp_req:: A node type may declare its output standing
   :id: CREQ_READER_READS_STANDING
   :derived_from: FEAT_STANDING_SERVES_LATER_PASSES
   :allocated_to: COMP_TOPOLOGY_READER
   :ears_pattern: event
   :statement: When a node type declares standing = true, Topology reader shall read that type's output as standing.

   ``DEC_STANDING_OUTPUTS``: in the node type's declaration, fixed before any
   run. A value that is not a boolean is refused as any value of the wrong
   type is (``CREQ_READER_FAULT_LOCATED``).

   Failure modes:

   - **The key read past**, as keys the reader does not know are, and the
     output ending with its first pass.
   - **``standing = false`` read as standing.**

.. comp_req:: An output that stands serves every later activation
   :id: CREQ_SCHEDULER_STANDING_SERVES
   :derived_from: FEAT_STANDING_SERVES_LATER_PASSES
   :allocated_to: COMP_RUN_SCHEDULER
   :ears_pattern: event
   :statement: When an instance whose output stands has produced, Run scheduler shall give that output to every later activation reading it until the instance produces another.

   An output stands when its node type declares it or when its instance has no
   edge into it that does not stand (``DEC_ONCE_RUN_OUTPUTS_STAND``).

   Failure modes:

   - **A standing output taken once**, and a repetition that reads the brief
     stalling on its second pass.
   - **The first of a standing node's outputs kept after it produced a
     second.**
   - **The output of an instance the run gave every input taken once**, it
     being declared standing nowhere.

.. comp_req:: A context a run is given for a wired parameter is its edge's first
   :id: CREQ_RUN_ARGUMENT_FIRST_ON_ITS_EDGE
   :derived_from: FEAT_ARGUMENT_FIRST_ON_ITS_EDGE
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: event
   :statement: When a run is started with a context for a parameter a binding fills, Workflow run shall hold that context as the first that edge holds, before any context walked along it.

   ``DEC_ARGUMENT_FIRST_ON_ITS_EDGE``. It is generation 0 of the edge, and
   the binding's contexts are 1 onward, whenever they are walked. Two
   contexts given for one parameter are refused as they are for any
   (``CREQ_RUN_REFUSES_UNFILLED_SIGNATURE``).

   Failure modes:

   - **The context refused as a second source**, which is what the run did,
     and a loop given no way into its first pass.
   - **The context placed after what was walked first**, and what a pass is
     given turned on whether the binding's source happened to run before the
     instance.
   - **The context given for every pass**, as a parameter nothing binds is,
     and the wire's contexts never taken.
