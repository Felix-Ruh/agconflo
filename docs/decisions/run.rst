==================================
Decisions about running a workflow
==================================

What a run is, how it is driven, and what makes one end. ``decisions/workflow``
settles what a workflow is made of and ``decisions/topology`` how it is written
down; these are the choices the first execution slice is written against, and
they are about what happens once a definition is sound.

Four of them rest on the prototype recorded in ``evidence/run``, which is a
different footing from the decisions above. Those were judgements about a model,
taken before anything was built. These were taken after building the thing once
and reading what it did, and each of the four names the finding that decided
it. The other four are judgements, and none of them is a measurement wearing a
judgement's clothes - where a decision below says a shape was rejected, it was
rejected on an argument, and the argument is written out.

What this slice does not settle is as load-bearing as what it does, so it is
named here rather than left to be inferred. Nothing below decides whether node
behaviour is eventually asynchronous, whether the engine acquires a runtime, how
a node's behaviour is written, how a context reaches a provider, or how a run
survives a restart. Nothing in this project records a decision on any of them,
so none is being superseded, and the first decision below is deliberately shaped
to leave every one of them open.

.. dec:: A run is driven by its caller
   :id: DEC_RUN_IS_DRIVEN
   :dec_status: accepted
   :decided_on: 2026-09-22
   :statement: Agconflo shall run a workflow by handing its caller one activation at a time rather than by running node behaviour itself.

   The engine decides what may run and what a node is given; performing the
   activation is the caller's, and the result comes back as a value. Nothing in
   the core opens a file, calls a provider, spawns a thread or awaits anything.

   Two alternatives were rejected, and neither on taste.

   A trait the core calls - ``NodeBehaviour``, implemented by the host - puts the
   behaviour boundary inside the core before ``STKH_LIVE_BEHAVIOUR`` has a
   feature to shape it, and the shape is exactly what that goal is about. Made
   asynchronous, which is what a provider call needs, it also forces the choice
   of a runtime now, for a slice that performs no input or output at all.

   An asynchronous engine throughout was the other, and it costs the same
   runtime for the same absent benefit. Every test of this slice would become a
   test of a runtime, and what is being tested here - which instance may
   activate, what it is given, how a run ends - has nothing asynchronous in it.

   Three things this buys, and the first is why it is worth a decision rather
   than an implementation note. A failure becomes a value the caller hands back,
   which is what makes ``STKH_TYPED_FAILURE`` expressible at all before there is
   any node behaviour to fail. A run that is waiting is a run nobody has fed, so
   a person taking part in one and a run surviving a restart stay reachable
   without redesigning this. And a test needs no runtime, no clock and no
   provider to drive a run to any of its endings.

   What it does not settle is named above, and the reason it does not is that
   the caller is free to be asynchronous without the core knowing. That freedom
   is the point, and it is the claim most worth disproving early: it rests on an
   argument rather than on a measurement, because the prototype's caller was
   synchronous.

.. dec:: A bound parameter is waited for
   :id: DEC_BINDING_IS_AWAITED
   :dec_status: accepted
   :decided_on: 2026-09-22
   :supported_by: EVD_RUN_OPTIONAL_BY_ORDER
   :statement: Agconflo shall wait for every parameter an instance binds before activating that instance, whether the parameter is declared required or optional.

   Optional says what a definition may leave unwired, not what may arrive late.
   ``DEC_DECLARED_PARAMETERS`` puts it in those words - optional parameters are
   "what lets a node be useful with less than everything wired" - so the
   declaration is about wiring, and once a binding exists the wire is there and
   the value is coming.

   The alternative is to ask readiness of the required parameters alone, which
   is the obvious reading and was what the prototype did. It was measured giving
   one definition two different answers depending on where a line sat in the
   document (``EVD_RUN_OPTIONAL_BY_ORDER``): a consumer written below its
   optional producer received that input, and written above it did not. Nothing
   reported the difference and both runs completed.

   That is the failure this project exists to rule out rather than an
   inconvenience. What a node saw would be decided by the order instances happen
   to be written in, which no author controls deliberately and no reader would
   suspect, and the run log would record an input that another arrangement of
   the same file would not have produced.

   The cost is accepted and worth stating: an optional parameter that is bound
   can no longer make a node run sooner. Nothing is lost that a definition
   cannot express, since leaving the parameter unbound says exactly that.

.. dec:: A run's arguments are held per entry parameter
   :id: DEC_ARGUMENTS_PER_ENTRY
   :dec_status: accepted
   :decided_on: 2026-09-22
   :supported_by: EVD_RUN_ENTRY_NAME_SHARED
   :statement: Agconflo shall hold a run's arguments under the entry instance and parameter each one fills rather than under a parameter name alone.

   ``DEC_WORKFLOW_SIGNATURE`` makes a workflow's entry nodes its typed parameter
   list, and the obvious reading of that is one argument per parameter name.
   Measured, that reading hands one context to two parameters that merely share
   a name (``EVD_RUN_ENTRY_NAME_SHARED``): two entry instances each declaring
   ``seed``, for context types ``Seed`` and ``Other``, both received the same
   argument and the run completed.

   Two instances of one node type are ordinary rather than exceptional, and they
   necessarily declare the same parameter names, so the collision is the common
   case and not a contrived one.

   Refusing a definition whose entry instances share a parameter name was the
   alternative. It was rejected for placing the refusal where it cannot be
   explained: the wiring is sound, an entry instance's parameters are not wired
   at all, and what a definition declares is what ``ARCH_WIRING`` keeps out of
   that feature. Addressing an argument by the instance it fills makes the
   question disappear rather than answered.

   It follows that a workflow's signature is a list of instance-and-parameter
   pairs rather than of names, which is what a caller binding to a workflow as a
   node will eventually have to name. That is not settled here.

   One shape this leaves is not a collision and has to be said outright, because
   it is the one an implementation gets wrong without noticing. An entry
   instance may also carry a binding: ``components/wiring`` records it among the
   shapes still open, the validator checks it like any other binding for its
   source and its type, and a loop closing back onto an entry node draws exactly
   that. The parameter then has two sources, the argument and the wire, and no
   ground to prefer either. Taking the argument and ignoring the wire is what
   the prototype did, silently, which is the answer this decision exists to
   refuse, so a run is refused instead, under the decision below.

.. dec:: A run that cannot start is refused rather than stuck
   :id: DEC_RUN_REFUSED_BEFORE_IT_STARTS
   :dec_status: accepted
   :decided_on: 2026-09-22
   :supported_by: EVD_RUN_MISSING_ARGUMENT_QUIESCES
   :statement: Agconflo shall refuse to start a run whose workflow is invalid or whose entry parameters do not each have exactly one source of their declared type rather than reporting either as a run unable to proceed.

   A refusal and an ending are different kinds of answer. A refusal is about what
   the caller supplied, is known before any instance has been looked at, and
   names the one thing to fix. An ending is about what happened.

   Three faults are refusals here: the workflow carries wiring defects, an
   argument for a required entry parameter is missing or is of a context type
   the declaration does not name, and an entry instance's parameter is bound as
   well as supplied, which gives it two sources and is argued under
   ``DEC_ARGUMENTS_PER_ENTRY``.

   Measured, the prototype conflated them (``EVD_RUN_MISSING_ARGUMENT_QUIESCES``):
   started with no arguments, a two-node workflow reported the same quiescence,
   naming both instances as waiting, as two instances bound to each other in a
   cycle. The cycle is a property of the definition and quiescence is the honest
   way to find it; the missing argument is a fault in the call, and reporting it
   by listing every waiting instance buries it.

   This is also where ``STKH_WIRING_CHECKED`` stops being about the
   representation and starts being about enforcement. The validator says what
   makes a workflow invalid and answers when it is asked; that goal says the
   engine refuses, and refuses before anything has happened. A run is the first
   thing in this project that can be said to have started, so it is the first
   place that half can be met.

.. dec:: A started run ends in exactly one way
   :id: DEC_RUN_ENDS_ONE_WAY
   :dec_status: accepted
   :decided_on: 2026-09-22
   :statement: Agconflo shall end a run it has started in exactly one of completion, a node's failure, an exhausted budget and quiescence.

   Four endings, and the set is closed on purpose. A run that ended has one
   answer to what became of it, and a caller handling four cases has handled all
   of them.

   Each is a different thing that happened, which is what stops the set being
   arbitrary: the workflow produced its result; a node the caller ran reported a
   failure; the run was stopped for doing too much; the run could do nothing
   more. The last two are the pair ``STKH_STEP_BUDGET`` and ``STKH_STUCK_RUN``
   already name as counterparts.

   One error type covering endings and refusals together was the alternative.
   It was rejected because a refusal happened before the run started and is the
   caller's to fix, while an ending is the run's own history, and a caller that
   cannot tell them apart without reading a message has been given one type
   holding two meanings.

   A run parked awaiting a person is deliberately none of the four. It has not
   ended, and under ``DEC_RUN_IS_DRIVEN`` it is a run whose caller has not yet
   reported an activation's outcome, which is not a state the engine is in at
   all. That is what keeps this consistent with ``STKH_HUMAN_IN_RUN`` before
   anything has been built for it.

.. dec:: A budget counts activations
   :id: DEC_BUDGET_COUNTS_ACTIVATIONS
   :dec_status: accepted
   :decided_on: 2026-09-22
   :statement: Agconflo shall count a run's budget in activations rather than in elapsed time.

   ``STKH_STEP_BUDGET`` exists because a runaway loop through an LLM spends real
   money while it happens, and what it spends is proportional to how many times
   a node ran rather than to how long the run took.

   Elapsed time was the alternative and loses twice. It is not reproducible, so
   no test can assert on it, and it is wrong about the thing being guarded:
   a run sitting for an hour awaiting a person has cost nothing and a hundred
   activations in a second have cost a hundred calls.

   Counting activations also puts the count somewhere the engine already is.
   Under ``DEC_RUN_IS_DRIVEN`` every activation passes through the engine's own
   hands, so nothing has to be instrumented or reported back for the budget to
   be exact.

.. dec:: A run completes on its designated output
   :id: DEC_COMPLETION_IS_DESIGNATED_OUTPUT
   :dec_status: accepted
   :decided_on: 2026-09-22
   :supported_by: EVD_RUN_COMPLETES_WITH_IDLE
   :statement: Agconflo shall complete a run once the instance its definition designates has produced an output, whatever other instances have not run.

   ``DEC_WORKFLOW_SIGNATURE`` designates exactly one output as the workflow's
   own result, so that is what a run is for and what finishing means.

   Measured, a run completed while holding an instance bound to its own output,
   which can never become ready and never ran (``EVD_RUN_COMPLETES_WITH_IDLE``).
   That reads like a defect and is not one.
   ``DEC_ROUTING_SEPARATE_FROM_CONTROL`` keeps a context's path apart from which
   nodes run, so an instance that no route to the designated output passes
   through has no claim on the run, and waiting for it would report a stuck run
   on a workflow that had produced its result.

   Requiring every instance to run was therefore the alternative, and it
   contradicts a decision already taken rather than merely costing something.

   The consequence is the one worth carrying forward: quiescence is a statement
   about the designated output and not about the graph. A run is stuck when that
   output can no longer be produced, however many instances are idle, and an
   idle instance on its own is not a defect to report.

.. dec:: An instance activates at most once in a run
   :id: DEC_ACTIVATION_ONCE_PER_RUN
   :dec_status: accepted
   :decided_on: 2026-09-22
   :statement: Agconflo shall activate each node instance at most once in a run.

   A boundary rather than an ambition, in the shape of ``DEC_NO_CONTENT_ADDRESSING``:
   it says what is not there yet so that nothing assumes otherwise.

   Loops are legal (``DEC_BACK_EDGES_ALLOWED``) and cannot presently be run. A
   loop needs a back edge, a back edge is an explicit control edge, and a
   definition carries no explicit control edges - ``DEC_IMPLIED_CONTROL_EDGES``
   reserves them for exactly the four cases none of which is representable yet.
   A cycle written in bindings alone is legal to write, and every instance in it
   waits for the one before it, so the run is quiescent. That is the right
   report rather than a gap: the decision allowing cycles says in as many words
   that a run which cannot proceed is reported rather than waited on.

   What this defers is activation and epoch tagging, and deferring it is the
   whole reason to write this down. With each instance activating once, no join
   can pair one pass's context with another's, which is the well-formed wrong
   answer tagging exists to prevent. The moment an instance can activate twice,
   tagging is required before anything else is built on top, and its mechanism
   stays open rather than guessed at - the same treatment
   ``DEC_METADATA_TRANSFORMED`` gives its default join, and for the same reason.

   Two further absences follow from having no second pass and are recorded here
   so they are not mistaken for oversights. A node type declares the global
   context types it reads (``DEC_DECLARED_PARAMETERS``) and nothing supplies
   them, so a run reads none. And a router decides which of its outgoing edges
   fire, which is a choice about control edges that do not exist, so no instance
   is skipped by a decision.
