===========================
A workflow repeating itself
===========================

A node activated again on each pass of a repetition, given every input from
the pass it is on, and a workflow whose nodes could not be given that refused
before it runs. Every requirement here derives from ``STKH_REPETITION`` and is
written against the decisions in ``decisions/run`` and ``decisions/workflow``,
which record the model each node's passes are worked out by
(``DEC_PASS_CLOCKS``, ``DEC_CYCLE_STARTED_BY_A_FIRST``).

What decides that a repetition ends is not here: it is a router's, in
``features/routing``, and a repetition no router ends is what the step budget
stops. Nor is a list handled item by item, which one node does by looping
over the list itself (``DEC_LIST_HANDLED_IN_ONE_NODE``).

Each requirement was checked by hand against the two questions every body here
answers: could it be false while its parent holds, and could the parent hold
while it is false? The first three are ``event``, the last ``unwanted``. The
third derives from ``STKH_WIRING_CHECKED`` as well: it is how a
repetition's first pass is given what its later passes are given by the
edges, said of the workflow, where a check can see it. The feature's
architecture closes the file.

.. feat_req:: A node runs again on each pass its inputs reach
   :id: FEAT_REPEAT_ON_NEW_CONTEXTS
   :derived_from: STKH_REPETITION
   :ears_pattern: event
   :verification_method: test
   :statement: When the inputs of a node instance hold the contexts of a pass it has not run, Agconflo shall activate the instance with the contexts of that pass.

   The parent lets a workflow repeat part of itself, and a part is repeated by
   walking its edges again (``DEC_CYCLE_STARTED_BY_A_FIRST``): a node runs again
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

.. feat_req:: A first context a binding declares comes first
   :id: FEAT_FIRST_CONTEXT_DECLARED
   :derived_from: STKH_REPETITION, STKH_WIRING_CHECKED
   :ears_pattern: event
   :verification_method: test
   :statement: When a binding declares its first context, Agconflo shall give that context to the binding's instance before any context walked along the binding.

   ``STKH_REPETITION`` lets a workflow repeat part of itself, and a node in a
   loop reads what comes back round, which cannot come back before the node
   has run: something has to come before it. ``STKH_WIRING_CHECKED`` rejects
   an invalid workflow before any node runs, and a loop nothing starts is
   one, whose nodes can never run; the check can tell only if what starts a
   loop is part of the workflow. A binding saying what comes first is both
   (``DEC_FIRST_CONTEXT_DECLARED``). It is the binding's pass 0, and what the
   binding carries its passes after (``DEC_PASS_CLOCKS``).

   It can be false while both parents hold only if a loop could start from
   something else a check could see, which would be a declaration by another
   name.

   It claims no more than they need: it says what the declared context is
   given before, not how many passes follow, which the edges decide, nor
   what the declaration is written as, which the decision says.

   It replaces ``FEAT_ARGUMENT_FIRST_ON_ITS_EDGE``, which took the first
   context from the run's arguments
   (``DEC_CHANGE_ARGUMENT_FIRST_ON_ITS_EDGE``).

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

.. feat_arch:: Repetition splits between the topology reader, the wiring validator, the run scheduler and the workflow run
   :id: ARCH_REPETITION
   :realises: FEAT_REPEAT_ON_NEW_CONTEXTS, FEAT_ENCLOSING_PASS_SERVES, FEAT_FIRST_CONTEXT_DECLARED, FEAT_PASSES_PAIRED_BEFORE_RUN
   :uses: COMP_TOPOLOGY_READER, COMP_WIRING_VALIDATOR, COMP_RUN_SCHEDULER, COMP_WORKFLOW_RUN
   :statement: Agconflo shall allocate repetition to the topology reader, the wiring validator, the run scheduler and the workflow run.

   - The topology reader answers for what the workflow document declares:
     the first context a binding gives.
   - The wiring validator answers for what the wiring allows: a cycle no
     declared first context starts, and an instance whose inputs share no
     pass, each a defect before anything runs.
   - The run scheduler answers for passes: which passes each instance runs
     on, worked out from the wiring, and for each activation the contexts of
     its pass, a binding's declared first context on its pass 0.
   - The workflow run answers for what it holds: each output as its
     instance's next pass, and each declared first context, made when the
     run starts.

   Its links were changed by ``DEC_CHANGE_ARCH_REPETITION``,
   ``DEC_CHANGE_ARCH_REPETITION_PASSES`` and
   ``DEC_CHANGE_ARCH_REPETITION_FIRST``.

   The decisions it is built against are named here rather than linked:
   ``DEC_PASS_CLOCKS``, ``DEC_PAIRING_IS_WIRING``,
   ``DEC_CYCLE_STARTED_BY_A_FIRST``, ``DEC_FIRST_CONTEXT_DECLARED`` and
   ``DEC_RUN_IS_DRIVEN``.
