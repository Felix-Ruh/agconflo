===========================
A workflow repeating itself
===========================

A node activated again each time the edges into it carry it something new,
and a context that stays for every later pass. Every requirement here derives
from ``STKH_REPETITION`` and is written against the decisions in
``decisions/run`` and ``decisions/workflow``, which recorded the model before
any of it was built (``DEC_EDGE_GENERATIONS``, ``DEC_STANDING_OUTPUTS``,
``DEC_REPETITION_BY_EDGES``).

What decides that a repetition ends is not here: it is a router's, in
``features/routing``, and a repetition no router ends is what the step budget
stops. Nor is a list handled item by item, which one node does by looping
over the list itself (``DEC_LIST_HANDLED_IN_ONE_NODE``).

Each requirement was checked by hand against the two questions every body here
answers: could it be false while its parent holds, and could the parent hold
while it is false? Both statements are ``event``. The feature's architecture
closes the file.

.. feat_req:: A node runs again when every edge into it carries something new
   :id: FEAT_REPEAT_ON_NEW_CONTEXTS
   :derived_from: STKH_REPETITION
   :ears_pattern: event
   :verification_method: test
   :statement: When every edge into a node instance that has run holds a context that stands or one it has not taken and at least one holds one it has not taken, Agconflo shall activate that instance again.

   The parent lets a workflow repeat part of itself, and a part is repeated by
   walking its edges again (``DEC_REPETITION_BY_EDGES``): a node that has run
   runs again when what it reads has moved on. Taking from each edge the
   earliest context it has not taken keeps one pass's contexts together
   (``DEC_EDGE_GENERATIONS``), and requiring one of them to be new is what
   stops a node whose inputs all stand from running for ever.

   It can be false while the parent holds. A run activating each node once,
   which is what the engine did, meets the parent for a workflow repeating by
   some other means - a node looping inside one activation - and leaves the
   workflow unable to show the step that was repeated, which is what the
   parent exists for.

   It claims no more than the parent: it says when a node runs again, not how
   many times, which is a router's to decide and the budget's to bound.

.. feat_req:: An output that stands serves every later pass
   :id: FEAT_STANDING_SERVES_LATER_PASSES
   :derived_from: STKH_REPETITION
   :ears_pattern: event
   :verification_method: test
   :statement: When a node instance produces an output that stands, Agconflo shall give that output to every later activation reading it until the instance produces another.

   A pass reads what changes and what does not: a draft revised each time,
   and the brief it was drafted from. The brief is produced once, so without
   an output that stands the second pass waits for a second brief that never
   comes. An output stands when its node type declares it
   (``DEC_STANDING_OUTPUTS``), or when its instance cannot run again
   (``DEC_ONCE_RUN_OUTPUTS_STAND``).

   It can be false while the parent holds. A workflow can repeat a part that
   reads nothing from outside it, and meets the parent; every part that reads
   something from outside - nearly every useful one - then runs once and
   stops.

.. feat_arch:: Repetition splits between the topology reader, the run scheduler and the workflow run
   :id: ARCH_REPETITION
   :realises: FEAT_REPEAT_ON_NEW_CONTEXTS, FEAT_STANDING_SERVES_LATER_PASSES
   :uses: COMP_TOPOLOGY_READER, COMP_RUN_SCHEDULER, COMP_WORKFLOW_RUN
   :statement: Agconflo shall allocate repetition to the topology reader, the run scheduler and the workflow run.

   - The topology reader answers for what a node type declares: that its
     output stands.
   - The run scheduler answers for which instance may activate and what it is
     given: the earliest context each edge holds that its instance has not
     taken, or the one that stands.
   - The workflow run answers for the edges: each output walked along every
     edge out of its instance, as that edge's next context.

   The decisions it is built against are named here rather than linked:
   ``DEC_EDGE_GENERATIONS``, ``DEC_STANDING_OUTPUTS``,
   ``DEC_REPETITION_BY_EDGES``, ``DEC_ONCE_RUN_OUTPUTS_STAND`` and
   ``DEC_RUN_IS_DRIVEN``.
