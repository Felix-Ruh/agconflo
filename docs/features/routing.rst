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
   was given, or its own output, which is its decision
   (``DEC_ROUTER_OUTPUT_IS_ITS_DECISION``), so ``STKH_ONE_OUTPUT`` holds and
   deciding stays apart from producing.

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
   activation again: its output comes from the record
   (``CREQ_RECORD_CONTINUES_THE_RUN``). Which edges a finished router walked is
   not in that output - its decision may be a model's text that a script read
   - so without the names the resumed run cannot say which contexts each edge
   holds.

   It can be false while the parents hold only by running every finished
   router again on resuming and trusting it to choose as it did, which a
   script edited since would not.

.. feat_arch:: Taking a branch splits between the reader, the validator, the script host, the run and its record
   :id: ARCH_ROUTING
   :realises: FEAT_ROUTE_WALKS_CHOSEN_EDGES, FEAT_ROUTE_RECORDED
   :uses: COMP_TOPOLOGY_READER, COMP_WIRING_VALIDATOR, COMP_SCRIPT_HOST, COMP_WORKFLOW_RUN, COMP_RUN_RECORD
   :statement: Agconflo shall allocate taking a branch to the topology reader, the wiring validator, the script host, the workflow run and the run record.

   - The topology reader answers for what a document says: that a node type
     routes, and which of a router's inputs a binding takes.
   - The wiring validator answers for whether that is sound: a binding taking
     an input from a node that does not route, or one it does not have, is a
     defect before anything runs.
   - The script host answers for what a router's script says: the instances it
     names, once.
   - The workflow run answers for the walk: a router's edges into the
     instances named, and no other, and a naming it refuses.
   - The run record answers for keeping the names, and giving them back.

   The decisions it is built against are named here rather than linked:
   ``DEC_ONE_GRAPH``, ``DEC_ROUTER_DECLARED``,
   ``DEC_ROUTER_OUTPUT_IS_ITS_DECISION``, ``DEC_ROUTED_INPUT_BOUND_BY_TABLE``
   and ``DEC_ROUTE_RECORDED``.
