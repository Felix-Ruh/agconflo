===============================
A model choosing a node's route
===============================

A node's script asking a model to choose among named options, and being given
the option chosen. It derives from ``STKH_ROUTING``, and is written against
the decisions in ``decisions/routing``.

What the first requirement does not do is what kept it a slice. It gives a
script a model's choice; how a router then takes the branch it chose is the
routing mechanism ``DEC_ONE_GRAPH`` describes, which the requirements after
its architecture specify. A choice is the one
kind of question: a score or a yes-or-no, which the decisions model measured
also answers (``EVD_DECISIONS_MODEL_SHAPE``), waits for a goal that needs one.

A decision is a model call (``DEC_DECISION_IS_A_MODEL_CALL``), so what is
already required of every model call a script makes holds of it unchanged:
the model its role is mapped to answers it, it is shown exactly the contexts it
is made with, its answer is a context, it counts against the call limit, a
failure ends the activation carrying the role and the provider's answer, and
the run's record holds it. What is added is the choice itself.

The requirement was checked by hand against the two questions every body here
answers: could it be false while its parent holds, and could the parent hold
while it is false? The statement is ``event``. The feature's architecture
closes the file.

.. feat_req:: A router's script is given the option a model chose
   :id: FEAT_ROUTE_CHOSEN_BY_MODEL
   :derived_from: STKH_ROUTING
   :ears_pattern: event
   :verification_method: test
   :statement: When the script of a node whose instance declares no calls asks a model to choose among named options, Agconflo shall give the script the option the model chose.

   The parent lets a node decide which branch a run takes, and names a model
   among what may make that decision. A model asked in words answers in words,
   and a script then reads a branch out of prose. Given the option as a value,
   a router branches on what the model chose rather than on what a script made
   of its text.

   It can be false while the parent holds. A host offering only
   ``host.complete`` meets the parent - a node can still decide, from a
   model's text - and leaves every model-made route resting on a parse.

   It claims no more than the parent. The node is the parent's router, which
   "passes on the contexts it was given rather than making new ones": an
   instance declaring calls has its model call node types, whose outputs are
   new contexts, and is a transform, not a router. Which options there are,
   and what they mean, is the script's to say.

.. feat_arch:: A model's choice splits between the model roster, the script host and the model map
   :id: ARCH_ROUTE_BY_MODEL
   :realises: FEAT_ROUTE_CHOSEN_BY_MODEL
   :uses: COMP_MODEL_ROSTER, COMP_SCRIPT_HOST, COMP_MODEL_MAP
   :statement: Agconflo shall allocate a model's choice to the model roster, the script host and the model map.

   Each answers for what the others cannot:

   - The model map answers for which decisions model a role reaches, read
     from the person's mapping, as it does for chat models.
   - The model roster answers for a decision on the wire: where it is sent,
     that it reaches a decisions model and no chat model, and that the answer
     chose among the options asked.
   - The script host answers for what a script may ask and what it is given:
     the questions it may put, where it may put them, and the choice and the
     answer it gets back.

   The decisions it is built against are named here rather than linked:
   ``DEC_DECISION_IS_A_MODEL_CALL``, ``DEC_QUESTIONS_ARE_CONTEXTS``,
   ``DEC_CHOICE_QUESTIONS_ONLY``, ``DEC_DECIDING_WITHOUT_CALLS``,
   ``DEC_DECIDE_GIVES_ANSWER_AND_CHOICES``, ``DEC_DECISIONS_ROLE_IN_THE_MAP``,
   ``DEC_DECISIONS_THROUGH_THEIR_ENDPOINT`` and ``DEC_DECISION_ANSWER_CHECKED``.

.. feat_req:: A router's contexts go along the edges into the instances it names
   :id: FEAT_ROUTE_WALKS_CHOSEN_EDGES
   :derived_from: STKH_ROUTING, STKH_ONE_OUTPUT
   :ears_pattern: event
   :verification_method: test
   :statement: When a router's activation names the instances its run is to go on to, Agconflo shall walk its edges into those instances and no other.

   The parent lets a node decide which branch a run takes, and a branch is
   the edges out of the node (``DEC_ONE_GRAPH``). A router creates no content:
   what goes along an edge into the branch it chose is one of the contexts it
   was given (``DEC_ROUTER_PASSES_ON_ITS_INPUTS``), so ``STKH_ONE_OUTPUT``
   holds and deciding stays apart from producing.

   It can be false while the parent holds. A node deciding in its output text,
   with every branch run and each discarding what it was not meant for, meets
   the parent in its letter; the run then does every branch's work, and its
   record no longer says which way it went.

.. feat_req:: A run's record holds where each router sent it
   :id: FEAT_ROUTE_RECORDED
   :derived_from: STKH_RESUMABLE_RUN, STKH_ROUTING
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall hold in a run's record the instances each router's activation named for its run to go on to.

   A run resumed from its record goes on from where it stopped, and where it
   stopped depends on every branch taken before. Resuming runs no finished
   activation again: what it gave comes from the record
   (``CREQ_RECORD_CONTINUES_THE_RUN``). A router gives no output, only the
   instances it named (``DEC_ROUTER_PASSES_ON_ITS_INPUTS``) - its decision may
   be a model's text that a script read and kept nowhere - so without the
   names the resumed run cannot say which contexts each edge holds.

   It can be false while the parents hold only by running every finished
   router again on resuming and trusting it to choose as it did, which a
   script edited since would not.

.. feat_req:: A router names one of the branches its instance declares
   :id: FEAT_ROUTE_IS_A_DECLARED_BRANCH
   :derived_from: STKH_ROUTING, STKH_REPETITION
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a router's activation names instances that are not those of one branch its instance declares, then Agconflo shall refuse that naming.

   ``STKH_ROUTING`` lets a node decide which of the branches after it a run
   takes: the branches are the workflow's, and the node picks among them.
   ``STKH_REPETITION`` leaves how one pass's contexts are kept apart from the
   next's to a decision, and the one taken works out each node's passes from
   the wiring before the run starts (``DEC_PASS_CLOCKS``). Which instances a
   router sends a run to together is part of that wiring
   (``DEC_ROUTER_BRANCHES_DECLARED``): a naming that is not one of its
   branches puts instances on passes the run was not checked for.

   It can be false while ``STKH_ROUTING`` holds: a router taking any set of
   the instances after it still decides which way the run goes. It cannot be
   while ``STKH_REPETITION`` holds as decided, for a node reading two
   instances a router can send a run to apart could be given two passes'
   contexts (``EVD_PASSES_MISPAIRED_ACROSS_A_BRANCH``).

   Naming nothing is not a branch and is not refused: the run goes nowhere
   from there, as before.

.. feat_req:: A router's declared branches are checked against its edges before anything runs
   :id: FEAT_BRANCHES_CHECKED
   :derived_from: STKH_WIRING_CHECKED, STKH_ROUTING
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If an instance declares branches that leave out an instance its edges enter or name one they do not enter or repeat a branch or belong to no router, then Agconflo shall reject the workflow before any node in it runs.

   ``STKH_ROUTING`` has a node decide among the branches after it, and
   ``STKH_WIRING_CHECKED`` rejects an invalid workflow before any node runs.
   Each case here is a workflow whose branches are not the branches after
   the node: an edge a router can never walk, a branch naming an instance it
   has no edge into, two names for one choice, and branches on a node that
   cannot choose. Found when the router first runs, each costs a run's work
   up to there, and one of them - an edge in no branch - waits for ever
   rather than failing.

   It can be false while both parents hold only if branches were not part of
   the workflow, which ``FEAT_ROUTE_IS_A_DECLARED_BRANCH`` makes them.

.. feat_arch:: Taking a branch splits between the reader, the validator, the script host, the run and its record
   :id: ARCH_ROUTING
   :realises: FEAT_ROUTE_WALKS_CHOSEN_EDGES, FEAT_ROUTE_RECORDED, FEAT_ROUTE_IS_A_DECLARED_BRANCH, FEAT_BRANCHES_CHECKED
   :uses: COMP_TOPOLOGY_READER, COMP_WIRING_VALIDATOR, COMP_SCRIPT_HOST, COMP_WORKFLOW_RUN, COMP_RUN_RECORD
   :statement: Agconflo shall allocate taking a branch to the topology reader, the wiring validator, the script host, the workflow run and the run record.

   - The topology reader answers for what a document says: that a node type
     routes, which of a router's inputs a binding takes, and the branches a
     router's instance declares.
   - The wiring validator answers for whether that is sound: a binding taking
     an input from a node that does not route, or one it does not have, and
     branches that are not the branches after the router, are defects before
     anything runs.
   - The script host answers for what a router's script says: the instances it
     names, once; and for a person's answer naming no branch, refused so that
     they can answer again.
   - The workflow run answers for the walk: a router's edges into the
     instances named, and no other, and a naming it refuses, one that is no
     branch among them.
   - The run record answers for keeping the names, and giving them back.

   ``FEAT_ROUTE_IS_A_DECLARED_BRANCH`` and ``FEAT_BRANCHES_CHECKED`` were added
   to what it realises by ``DEC_CHANGE_ARCH_ROUTING``.

   The decisions it is built against are named here rather than linked:
   ``DEC_ONE_GRAPH``, ``DEC_ROUTER_DECLARED``,
   ``DEC_ROUTER_PASSES_ON_ITS_INPUTS``, ``DEC_ROUTED_INPUT_BOUND_BY_TABLE``,
   ``DEC_ROUTE_RECORDED_ALONE`` and ``DEC_ROUTER_BRANCHES_DECLARED``.
   ``DEC_ROUTER_OUTPUT_IS_ITS_DECISION`` and ``DEC_ROUTE_RECORDED``, which it
   was first built against, are superseded by the second and the fifth.

.. feat_req:: Nothing goes along a router's edges but what the router was given
   :id: FEAT_ROUTER_MAKES_NO_CONTEXT
   :derived_from: STKH_ROUTING, STKH_ONE_OUTPUT
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall carry along an edge out of a router no context but one that router's activation was given.

   ``STKH_ROUTING`` has a router pass on the contexts it was given rather
   than make new ones, and ``STKH_ONE_OUTPUT`` keeps deciding apart from
   producing. This is the two together at the level of an edge: whatever a
   router's branch is given, its provenance is that of the node that made it,
   never the router's.

   It can be false while both parents hold. A router whose script writes its
   verdict as text, walked along an edge as the router's output, decides
   which branches run - routing, in the letter of the first parent - and has
   made a context, which every node after it then reads as if something other
   than a router had produced it. That is what a router's output was until
   ``DEC_ROUTER_PASSES_ON_ITS_INPUTS``.

.. feat_req:: A workflow taking from a router what it does not make is rejected
   :id: FEAT_ROUTER_OUTPUT_TAKEN_REJECTED
   :derived_from: STKH_WIRING_CHECKED, STKH_ROUTING
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a workflow binds a router's output or designates a router as its output or declares a call to a router, then Agconflo shall reject that workflow.

   A router makes no context (``FEAT_ROUTER_MAKES_NO_CONTEXT``), so each of
   the three asks a router for something it never gives: an edge that would
   carry nothing, a run with no result to end on, a call with nothing to give
   back. ``STKH_WIRING_CHECKED`` rejects such a workflow before any node in it
   runs, rather than leaving it to wait for ever on what never comes.

   It can be false while both parents hold. A run that started such a
   workflow and waited at the edge would never run a node that could not
   proceed, and would report the run stuck - checked, in a sense, but after
   every node before the edge was paid for.

.. feat_arch:: A router making no context is held by the reader, the validator, the script host and the run
   :id: ARCH_ROUTER_NO_CONTEXT
   :realises: FEAT_ROUTER_MAKES_NO_CONTEXT, FEAT_ROUTER_OUTPUT_TAKEN_REJECTED
   :uses: COMP_TOPOLOGY_READER, COMP_WIRING_VALIDATOR, COMP_SCRIPT_HOST, COMP_WORKFLOW_RUN
   :statement: Agconflo shall allocate a router making no context to the topology reader, the wiring validator, the script host and the workflow run.

   - The topology reader refuses a router's node type that declares an
     output (``DEC_ROUTER_DECLARES_NO_OUTPUT``).
   - The wiring validator reports a binding of a router's output, a router
     designated as a workflow's output, and a call declared to a router.
   - The script host reports a router's route and no output.
   - The workflow run refuses an output reported for a router, and walks a
     router's edges with its inputs alone.

   The last two were already ``ARCH_ROUTING``'s, through requirements whose
   statements change under ``DEC_ROUTER_PASSES_ON_ITS_INPUTS`` and which now
   answer to this feature as well.
