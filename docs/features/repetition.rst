===========================
A workflow repeating itself
===========================

A node activated again on each pass of a repetition, given every input from
the pass it is on, and a workflow whose nodes could not be given that refused
before it runs. Every requirement here derives from ``STKH_REPETITION`` and is
written against the decisions in ``decisions/run`` and ``decisions/workflow``,
which record the model each node's passes are worked out by
(``DEC_PASS_CLOCKS``, ``DEC_REPETITION_BY_EDGES``).

What decides that a repetition ends is not here: it is a router's, in
``features/routing``, and a repetition no router ends is what the step budget
stops. Nor is a list handled item by item, which one node does by looping
over the list itself (``DEC_LIST_HANDLED_IN_ONE_NODE``).

Each requirement was checked by hand against the two questions every body here
answers: could it be false while its parent holds, and could the parent hold
while it is false? The first three are ``event``, the last ``unwanted``. The
third derives from ``STKH_RUN_FROM_ANY_PARAMETER`` as well: it is how a
repetition's first pass is given what its later passes are given by the
edges. The feature's architecture closes the file.

.. feat_req:: A node runs again on each pass its inputs reach
   :id: FEAT_REPEAT_ON_NEW_CONTEXTS
   :derived_from: STKH_REPETITION
   :ears_pattern: event
   :verification_method: test
   :statement: When the inputs of a node instance hold the contexts of a pass it has not run, Agconflo shall activate the instance with the contexts of that pass.

   The parent lets a workflow repeat part of itself, and a part is repeated by
   walking its edges again (``DEC_REPETITION_BY_EDGES``): a node runs again
   when what it reads has moved on to another pass. The parent leaves how one
   pass's contexts are kept apart from the next's to a decision
   (``DEC_PASS_CLOCKS``); what it needs of every way of keeping them is that a
   pass is given its own, for a repeated node given two passes' contexts
   answers a question nobody asked (``EVD_PASSES_MISPAIRED_ACROSS_A_BRANCH``).

   It can be false while the parent holds. A run activating each node once,
   which is what the engine did, meets the parent for a workflow repeating by
   some other means - a node looping inside one activation - and leaves the
   workflow unable to show the step that was repeated.

   It claims no more than the parent: it says when a node runs again and what
   it is given, not how many times, which is a router's to decide and the
   budget's to bound.

   Restated by ``DEC_CHANGE_REPEAT_ON_NEW_CONTEXTS``.

.. feat_req:: A repeated node reads what was made outside its passes from the pass it is on
   :id: FEAT_ENCLOSING_PASS_SERVES
   :derived_from: STKH_REPETITION
   :ears_pattern: event
   :verification_method: test
   :statement: When a node instance reads an output made on passes enclosing its own, Agconflo shall give each of its activations the context made on the enclosing pass that activation belongs to.

   A pass reads what changes and what does not: a draft revised each time,
   and the brief it was drafted from, made once before the repetition began.
   A node on a branch the repetition takes on some passes reads what was made
   on every pass of it: the draft of the pass the branch was taken. Each is
   made on passes enclosing the reader's own - the run's one pass encloses
   every other, and a router's passes its branches' - and the reader's pass
   belongs to exactly one of them.

   It can be false while the parent holds. A workflow can repeat a part that
   reads nothing from outside it, and meets the parent; every part that reads
   something from outside - nearly every useful one - then runs once and
   stops, or is given the brief or the draft of another pass.

   It replaces ``FEAT_STANDING_SERVES_LATER_PASSES``, which met the same need
   through an output declared standing (``DEC_CHANGE_STANDING_SERVES_LATER_PASSES``).

.. feat_req:: A context a run is given for a wired parameter comes first
   :id: FEAT_ARGUMENT_FIRST_ON_ITS_EDGE
   :derived_from: STKH_RUN_FROM_ANY_PARAMETER, STKH_REPETITION
   :ears_pattern: event
   :verification_method: test
   :statement: When a run is started with a context for a parameter a binding fills, Agconflo shall give that context to the parameter's instance before any context walked along the binding.

   ``STKH_RUN_FROM_ANY_PARAMETER`` lets a run be given a context for any
   node's parameter, and a parameter a binding fills is one of them. Given a
   context there, something has to say which comes first, it or what the wire
   carries; saying nothing refuses one or loses the other.
   ``STKH_REPETITION`` needs the answer to be "it": a node in a loop reads
   what comes back round, which cannot come back before the node has run
   (``DEC_ARGUMENT_FIRST_ON_ITS_EDGE``). It is the first pass of the passes the
   binding gives the node (``DEC_PASS_CLOCKS``).

   It can be false while both parents hold. A run could refuse a context for
   a wired parameter, as it did, and every other parameter could still be
   given one; the loop would then have no way into its first pass, and would
   repeat nothing.

   It claims no more than they need: it says which context comes first, not
   how many passes follow, which the edges decide.

.. feat_req:: A workflow whose nodes cannot be given one pass's contexts does not run
   :id: FEAT_PASSES_PAIRED_BEFORE_RUN
   :derived_from: STKH_REPETITION, STKH_WIRING_CHECKED
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a node instance's inputs cannot be given contexts of one pass, then Agconflo shall refuse to start the run before any node in it runs.

   ``FEAT_REPEAT_ON_NEW_CONTEXTS`` gives a node the contexts of one pass,
   and some wiring leaves a node none: a node joining two branches no route
   takes together, or a loop's own node reading what a branch of it makes on
   some passes alone. Run, such a node waits for ever or is given two passes'
   contexts. ``STKH_WIRING_CHECKED`` rejects an invalid workflow before any
   node runs, and this is the invalidity repetition adds.

   It is a refusal of a run rather than of a workflow: which passes a loop
   has depends on the parameters the run gives a first context to
   (``DEC_PAIRING_CHECKED_AT_START``). Either way, nothing has run.

   It can be false while both parents hold only if a node could always be
   given one pass's contexts, which the first case above shows it cannot.

.. feat_arch:: Repetition splits between the run scheduler and the workflow run
   :id: ARCH_REPETITION
   :realises: FEAT_REPEAT_ON_NEW_CONTEXTS, FEAT_ENCLOSING_PASS_SERVES, FEAT_ARGUMENT_FIRST_ON_ITS_EDGE, FEAT_PASSES_PAIRED_BEFORE_RUN
   :uses: COMP_RUN_SCHEDULER, COMP_WORKFLOW_RUN
   :statement: Agconflo shall allocate repetition to the run scheduler and the workflow run.

   - The run scheduler answers for passes: which passes each instance runs
     on, worked out from the wiring and the contexts the run is given, which
     instances cannot be given one pass's contexts, and for each activation
     the contexts of its pass.
   - The workflow run answers for what it holds and when it starts: each
     output as its instance's next pass, a context it was given for a wired
     parameter as that binding's first, and a refusal before anything runs of
     a workflow whose nodes the scheduler found unpaired.

   Its links were changed by ``DEC_CHANGE_ARCH_REPETITION`` and
   ``DEC_CHANGE_ARCH_REPETITION_PASSES``.

   The decisions it is built against are named here rather than linked:
   ``DEC_PASS_CLOCKS``, ``DEC_PAIRING_CHECKED_AT_START``,
   ``DEC_REPETITION_BY_EDGES``, ``DEC_ARGUMENT_FIRST_ON_ITS_EDGE`` and
   ``DEC_RUN_IS_DRIVEN``.
