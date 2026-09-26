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

Those are the first eight. The ninth was taken a day later on measurements of
the run itself rather than of the prototype: what a run does with an output that
contradicts a declaration, once it has been shown to accept two kinds of them
without a word. The tenth, last in the file, was taken the same day on two more
such measurements: where a run's identifiers are kept unambiguous.

The last two were taken by the maintainer on 2026-09-26, when a node became able
to run more than once: how a run pairs the contexts on a node's edges pass by
pass, and which contexts stay the same for every pass. They supersede the first
decision below, which said no instance ran twice, and the one that waited for
every bound parameter whether required or optional.

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
   :dec_status: superseded
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
   :dec_status: superseded
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

.. dec:: A refused output leaves its activation outstanding
   :id: DEC_REFUSED_OUTPUT_OUTSTANDING
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_RUN_ACCEPTS_UNDECLARED_OUTPUT, EVD_RUN_ACCEPTS_HELD_IDENTIFIER
   :statement: Agconflo shall refuse an output that contradicts a declaration by keeping its activation outstanding rather than by ending the run.

   Both measured defects are an output the run should not have accepted: one of
   a type its node type does not declare (``EVD_RUN_ACCEPTS_UNDECLARED_OUTPUT``),
   and one carrying an identifier the run already held
   (``EVD_RUN_ACCEPTS_HELD_IDENTIFIER``). That the run refuses them is what the
   requirements derived from them say. This settles what the run does instead.

   Two alternatives were rejected, and neither on taste.

   Ending the run was the first, and it has no ending to end with. A node's
   failure carries the caller's own failure type (``CREQ_RUN_ENDS_ON_FAILURE``),
   which the run cannot construct, and a fifth ending would supersede
   ``DEC_RUN_ENDS_ONE_WAY`` for a case the caller can already express: it
   reports the activation failed, with a failure of its own saying why.

   Checking in the caller alone was the other. Every caller would carry the same
   two checks, and the measurements were taken with a caller that carried
   neither - which is the ordinary state of a caller written by somebody who
   did not know the checks were theirs to make.

   Keeping the activation outstanding is what stops a refusal being ignored. A
   caller that asks for the next step is handed the same activation again, so a
   run cannot move past an output it refused. The caller then does one of the
   two things it already can: it reports an output the run accepts, or it
   reports the activation failed and the run ends carrying that failure.

   The activation is counted against the budget once, when it was offered, and
   not again on the answer that follows a refusal. The node was not offered a
   second time, so nothing a budget exists to count has happened twice.

.. dec:: An identifier names one context in a run
   :id: DEC_IDENTIFIER_NAMES_ONE_CONTEXT
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_RUN_ARGUMENTS_SHARE_IDENTIFIER, EVD_RUN_PART_SHARES_IDENTIFIER
   :statement: Agconflo shall refuse whatever would bring a second context under an identifier a run already holds rather than keep a run to one identifier source.

   Two contexts under one identifier were measured twice, once among a run's
   arguments (``EVD_RUN_ARGUMENTS_SHARE_IDENTIFIER``) and once as a part of an
   output (``EVD_RUN_PART_SHARES_IDENTIFIER``), and both times the run completed
   and its result's lineage lost a context. Every question about where a context
   came from is asked by identifier, so a run holding two contexts under one
   identifier answers them wrongly without saying so.

   The cause is two identifier sources, and ``IdSource`` already says that
   preventing them belongs to whatever owns a run. Three ways of doing that were
   weighed, and two lost on an argument.

   The run owning the source was the first. It would keep the contexts a caller
   makes during a run to one source, and not the ones it brings: arguments are
   made before a run exists, and may come from another run, a document or a
   cache. Anything a run is started with can carry any identifier.

   Identifiers unique across sources was the second - a counter shared by every
   source in a process, or identifiers too large to repeat. Within one process
   that would make the collision impossible, and across processes it would not,
   which is where a resumed run's identifiers will come from. How identifiers
   survive a process is ``STKH_RESUMABLE_RUN``'s to decide, and this is
   deliberately a check that holds whatever they become.

   So the run refuses the collision where it enters: at the start, among the
   arguments and everything they were composed from, and at each output, among
   everything the output was composed from that the run does not already hold.
   "Holds" means the very value, not an equal one: a context passed on by
   reference is the one the run holds, and passing an input on by composing it
   is the sanctioned shape (``DEC_COMPOSITION_BY_REFERENCE``). A different
   context under a held identifier is what is refused, and telling the two
   apart needs the value's identity, because by identifier they are the same.

.. dec:: Every edge numbers the contexts walked along it, and a node takes the lowest of each
   :id: DEC_EDGE_GENERATIONS
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supersedes: DEC_ACTIVATION_ONCE_PER_RUN, DEC_BINDING_IS_AWAITED
   :statement: Agconflo shall number the contexts walked along each edge into an instance from 0 and activate the instance once every edge into it holds its next unprocessed generation, taking the lowest unprocessed generation from each.

   An edge may be walked more than once: a router can send contexts back along
   an edge to a node that has run (``DEC_REPETITION_BY_EDGES``). Each walk adds
   the next generation on that edge - 0, then 1 - and each edge into a node is a
   queue of its own. A node that has not run waits for generation 0 on every
   edge into it, however many later generations one edge already holds, and
   every later activation takes the next generation of each. So pass ``n`` of a
   node is given the ``n``-th context of each of its edges, and nothing else.

   This is the tagging ``DEC_ACTIVATION_ONCE_PER_RUN`` deferred until an
   instance could activate twice: without it a join pairs one pass's context
   with another's, "a well-formed wrong answer". Generations are what make the
   pairing exact, and ``DEC_IDENTITY_PER_ACTIVATION`` still gives each context
   its own identifier; the generation is where on its edge it was walked, not
   what it is.

   Every edge is waited for, as ``DEC_BINDING_IS_AWAITED`` had it for every
   bound parameter, and for the reason that decision measured
   (``EVD_RUN_OPTIONAL_BY_ORDER``): what a node is given must not depend on the
   order anything is written in. There are no optional parameters any more to
   distinguish (``DEC_EVERY_INPUT_REQUIRED``).

   A pass is given no history. What earlier passes produced reaches a later one
   only by being wired to it, like any other context. The code still activates
   each instance once; it follows the superseded decisions until the repetition
   feature is built.

.. dec:: A node type declares an output standing, which serves every later generation
   :id: DEC_STANDING_OUTPUTS
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall let a node type declare its output standing in its definition, so that each edge carrying it holds that context for every later generation until the node produces another.

   Some contexts do not change from one pass to the next: the brief of a task,
   a set of rules, an example. Walking them along an edge again for every pass
   would mean running their node again to produce the same thing. A standing
   output is walked once and then serves every generation from there on, until
   its node runs again and produces a new one, which serves from its own
   generation onward.

   It is fixed where the node type is defined, before any run, and never
   decided while one runs: whether a context stands is part of what the node
   is, as its parameters are, so a workflow's definition says everything about
   how its contexts are paired (``DEC_EDGE_GENERATIONS``).

   A standing context also keeps its identity pass after pass, which is what
   the prefix store needs to find it again at the head of the next call
   (``DEC_PREFIX_STORE``).
