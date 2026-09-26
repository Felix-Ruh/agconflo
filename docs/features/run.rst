==================
Running a workflow
==================

The first execution feature: a run over a definition that is already sound, from
the arguments it starts with to the one way it ends. Every requirement here
derives from a goal in ``stakeholder/execution`` or ``stakeholder/context``,
and each is written against the decisions in
``decisions/run`` rather than re-opening them.

What a run does not do here is what keeps the feature a slice rather than the
engine. It performs no activation itself - the caller does, under
``DEC_RUN_IS_DRIVEN`` - so nothing here reaches a provider, a script or a file.
This slice activated no instance twice (``DEC_ACTIVATION_ONCE_PER_RUN``), so it
had no loops, no routers and no activation tagging; ``features/repetition``
and ``features/routing`` add them, superseding that decision. A run reads no global context, and
nothing is persisted.

Each requirement was checked by hand against the question no rule can ask: could
this be false while its parent is true? The body of each says how. One of them
is unusually easy to answer, and it is worth saying why rather than leaving it
to be noticed: the crate today satisfies every one of the seven requirements
that say what makes a workflow invalid, and there is no run to refuse anything,
because nothing calls the validator. A goal met by a judgement nobody asks for
is the gap this feature closes.

Five of the first seven statements below use the ``unwanted`` pattern and one
uses ``event``. ``DEC_EARS_SIX_PATTERNS`` predicted that distribution would
arrive with a feature that has real triggers, and asked to be looked at again
when one had been written. Nothing here was hard to classify and none of the six
is still unused in a way that argues for a change, so the decision is left as it
stands.

The last two were added a day after the rest, once the run had been measured
accepting outputs that contradict what was declared about them
(``EVD_RUN_ACCEPTS_UNDECLARED_OUTPUT``, ``EVD_RUN_ACCEPTS_HELD_IDENTIFIER``).
Both are ``unwanted`` as well. They are about what the caller hands back rather
than about what the run hands out, which no requirement had looked at, because
until something could perform an activation nothing could hand back anything
surprising.

The tenth came the same day, from the first real caller: a context brought into
a run under an identifier the run already holds for another
(``EVD_RUN_ARGUMENTS_SHARE_IDENTIFIER``, ``EVD_RUN_PART_SHARES_IDENTIFIER``). It
is ``ubiquitous``, because it is a property of the whole run rather than of one
answer, and the component requirements under it say where it is enforced.

The feature's architecture closes the file. It realises all ten requirements
and names the components they are divided between, which are defined in
``components/run``.

.. feat_req:: A run of an invalid workflow does not start
   :id: FEAT_RUN_REFUSES_INVALID_WIRING
   :derived_from: STKH_WIRING_CHECKED
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a workflow carries a wiring defect, then Agconflo shall refuse to start a run of it.

   The enforcement half of its parent, which the seven requirements beside it
   deliberately do not cover. Each of those says what makes a workflow invalid
   and is answered by a validator when something asks it. This says the run asks.

   It can be false while all seven hold, and today it is. ``validate_wiring``
   reports every defect class the wiring feature names, and nothing outside its
   own tests calls it, so a workflow with five defects is rejected precisely when
   a person thinks to check. The parent's wording - reject an invalid workflow before any
   node in it runs - is satisfied by a validator that is never asked, because no
   node ever runs either.

   ``FEAT_WIRING_REQUIRED_BOUND`` already carries the phrase "before running any
   node of it", and it is about one defect class rather than about who does the
   refusing. This requirement is about the run: it is the first thing in this
   project that can be said to have started, which is what makes it the first
   place a refusal can come before anything.

   What the refusal carries is not this requirement's business. That a run
   refuses on all of a workflow's defects rather than the first belongs to the
   component allocated the work, where the same obligation on the validator
   already lives.

.. feat_req:: Every entry parameter is filled exactly once
   :id: FEAT_RUN_ENTRY_SOURCE_EXACT
   :derived_from: STKH_EXPLICIT_CONTEXT
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If an entry parameter of a run is not filled by exactly one context of its declared type, then Agconflo shall refuse to start that run.

   An entry instance's parameters are the workflow's own rather than wires
   (``DEC_WORKFLOW_SIGNATURE``), so they are the one place a node's inputs come
   from outside the graph. The parent says a node is given exactly the contexts
   wired to it, and this is the boundary where "wired" stops meaning anything.

   It can be false while the parent holds, in three ways, and one of them was
   measured. An argument may be missing, and the run then has nothing to give a
   parameter the declaration says is required. It may be of a context type the
   declaration does not name, which the wiring validator would refuse between
   two instances and cannot see here, because there is no binding to look at. Or
   the parameter may be both supplied and bound - sound wiring, recorded among
   the open shapes in ``components/wiring``, and exactly what a loop closing back
   onto an entry node draws - which gives it two sources and no ground to prefer
   either.

   ``DEC_ARGUMENTS_PER_ENTRY`` is what makes the first of those a real question
   rather than a bookkeeping one. Arguments are addressed by instance and
   parameter, so two entry instances declaring one parameter name are two
   parameters to fill, where the obvious reading of a workflow's parameter list
   made them one and was measured feeding both from a single context.

.. feat_req:: A node waits for everything wired to it
   :id: FEAT_RUN_WAITS_FOR_EVERY_BINDING
   :derived_from: STKH_EXPLICIT_CONTEXT
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall activate a node instance only once every parameter that instance binds has a context.

   The requirement this slice exists to get right, and the one measured wrong.

   It can be false while its parent holds, and the failure is not contrived - it
   is what a prototype did on the first attempt. Readiness asked of the required
   parameters alone lets an instance activate before an optional parameter's
   source has produced, so the node is given a strict subset of what was wired to
   it. Nothing in the parent is violated: every context the node received was
   wired to it. It simply did not receive one that was.

   ``EVD_RUN_OPTIONAL_BY_ORDER`` is what turns that from a possibility into the
   default. The instance that is ready first is found by scanning the
   definition, so which inputs a node receives is decided by the order its
   instances are written in: the same definition with one line moved gave one
   node both of its inputs and then only one, with no report either time.

   ``DEC_BINDING_IS_AWAITED`` settled it in the other direction, reading
   "optional" as what a definition may leave unwired and not what may arrive
   late. ``DEC_EVERY_INPUT_REQUIRED`` has since removed optional parameters, so
   every parameter an instance's type declares is bound and waited for.

.. feat_req:: A run stops at its budget
   :id: FEAT_RUN_BUDGET_STOPS
   :derived_from: STKH_STEP_BUDGET
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a run has activated as many nodes as its step budget allows, then Agconflo shall stop that run.

   The parent stops a run that exceeds its budget. This says the run stops at the
   budget rather than past it, which is a different claim and the one that
   matters.

   It can be false while the parent holds, and the way it fails is the ordinary
   off-by-one. A run that checks its count after handing out an activation has
   stopped a run that exceeded its budget - the parent is satisfied word for
   word - and has spent one more activation than was allowed. Under
   ``DEC_BUDGET_COUNTS_ACTIVATIONS`` an activation is the unit precisely because
   it is the thing that costs money, so one over budget is one unbudgeted call
   to a provider, every time, on every run that hits the limit.

   Counting activations rather than elapsed time is what makes this assertable at
   all. A budget in seconds could not be tested without a clock, and would be
   wrong about what it guards: a run parked awaiting a person costs nothing for
   an hour.

.. feat_req:: A run that can do nothing more ends
   :id: FEAT_RUN_QUIESCENCE_ENDS
   :derived_from: STKH_STUCK_RUN
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If no node instance of a run can activate and its designated output has produced no context, then Agconflo shall end that run.

   The parent reports a run in which no node can make further progress. This says
   what "no further progress" is a question about, and ends the run rather than
   reporting on one that continues.

   It can be false while the parent holds, in both directions, and one of them
   was measured. A run whose designated output has produced its result while
   instances sit idle has made no further progress by any reading of the graph,
   and it is finished rather than stuck: ``EVD_RUN_COMPLETES_WITH_IDLE`` recorded
   exactly that shape, an instance bound to its own output that could never run,
   in a workflow that completed. Reporting it as stuck would be a false alarm on
   every workflow with a branch the result does not depend on. In the other
   direction, a report that leaves the run open invites a caller to ask again,
   and nothing will ever change, because nothing is running that could change it.

   ``DEC_COMPLETION_IS_DESIGNATED_OUTPUT`` is what ties the question to the
   output rather than to the graph, and ``DEC_ROUTING_SEPARATE_FROM_CONTROL`` is
   why it can be tied there at all: a context's path and which nodes run are
   separate, so an instance no route to the result passes through has no claim
   on the run.

   A cycle written in bindings is the shape that reaches this honestly. It is
   legal (``DEC_BACK_EDGES_ALLOWED``) and cannot be run until explicit control
   edges exist, so every instance in it waits for the one before it, and the run
   is quiescent from the start.

.. feat_req:: A failed activation ends the run with its failure
   :id: FEAT_RUN_FAILURE_CARRIED
   :derived_from: STKH_TYPED_FAILURE
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If the caller reports an activation as failed, then Agconflo shall end that run carrying the failure reported for it.

   The parent says which failure occurred is reported when a node fails. This
   says the run's own ending is what carries it.

   It can be false while the parent holds, and the difference is the whole
   reason the parent was written. A failure printed to a log, or folded into a
   message, has been reported - and a caller that has to parse prose to learn
   which failure occurred cannot assert on it, which is what the testing policy
   requires of every expected failure mode. Carrying it in the ending makes it a
   value.

   It can also be false by continuing. A run that records the failure and goes on
   activating whatever else is ready has reported which failure occurred and has
   spent further activations on a run whose result is already unreachable.

   ``DEC_RUN_IS_DRIVEN`` is what makes this expressible before any node behaviour
   exists. The caller performs the activation and hands back either a context or
   a failure, so a failure is a value the engine receives rather than a panic in
   code it called - and the placeholder this parent supersedes, an erroring node
   panicking and the run stopping, could not have carried anything.

.. feat_req:: A run's result is its designated output
   :id: FEAT_RUN_COMPLETES_ON_DESIGNATED
   :derived_from: STKH_WORKFLOW_AS_NODE
   :ears_pattern: event
   :verification_method: test
   :statement: When the instance a workflow designates has produced its output, Agconflo shall end that run with that context as its result.

   Its parent needs a workflow to have the shape of a node type, and the half
   that makes one usable by a caller is that invoking it yields a value. The two
   requirements already under that parent are about the designation being well
   formed - exactly one, and naming an instance that exists. This is what the
   designation is for.

   It can be false while both hold. A workflow may designate exactly one output,
   that output may resolve to a real instance, and a run may end by reporting
   that every instance has been activated, leaving a caller to work out which
   context was the result. Nothing static is wrong with such a workflow, and
   nothing could bind it into another one.

   It can also be false by ending too late. A run that keeps activating whatever
   remains ready after the designated output has produced its context yields the
   same result, having spent activations on nodes nothing was waiting for -
   which under ``DEC_COMPLETION_IS_DESIGNATED_OUTPUT`` is not what a run is for.

.. feat_req:: A node's output is of the type its node type declares
   :id: FEAT_RUN_OUTPUT_OF_DECLARED_TYPE
   :derived_from: STKH_WIRING_CHECKED
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If the caller reports an output whose context type differs from the output type its node type declares, then Agconflo shall refuse that output.

   The parent refuses an invalid workflow before any node in it runs, and the
   check it is met by compares declarations: the output type one node type
   declares against the parameter type it is bound to. That comparison is a
   statement about a run only while every output is of the type declared for
   it, and nothing held a run to that.

   It can be false while the parent holds, and it was measured false
   (``EVD_RUN_ACCEPTS_UNDECLARED_OUTPUT``). The workflow passed validation, so
   the parent was met, and a parameter declared for ``prompt`` was then handed a
   ``banana``. The parent's own body puts the value of the check in what it
   spares - a wiring mistake costing a rejection rather than half of an
   expensive run - and an output contradicting its declaration spends the rest
   of the run on inputs whose types nobody has checked.

   This derivation is a judgement rather than a reading, so the reason for this
   parent is worth giving. ``STKH_EXPLICIT_CONTEXT`` is the other candidate and
   is not true of it: the mistyped context was the one wired to the parameter,
   so the node was given exactly what was wired to it. What failed is the claim
   the check before the run made, and that claim is this parent's.

   That the refusal leaves the activation outstanding rather than ending the run
   is ``DEC_REFUSED_OUTPUT_OUTSTANDING``, and belongs to the component allocated
   the work.

.. feat_req:: A node's output is a context new to the run
   :id: FEAT_RUN_OUTPUT_IS_NEW
   :derived_from: STKH_PROVENANCE
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If the caller reports an output whose identifier the run already holds, then Agconflo shall refuse that output.

   The parent records which context each byte of a node's input came from, and
   the record is kept by identifier. ``DEC_IDENTITY_PER_ACTIVATION`` is what
   makes an identifier worth recording: it names the one activation that
   produced the context. What a run holds is its arguments and the outputs it
   has accepted, and an output carrying one of their identifiers credits one
   context to two producers.

   It can be false while the parent holds. A record can be kept faithfully and
   still be wrong, because the identifiers it faithfully records are ambiguous -
   measured (``EVD_RUN_ACCEPTS_HELD_IDENTIFIER``): a run completed with the
   context one instance produced recorded as another's result.

   Passing an input on is legitimate, and there is a shape for it that keeps the
   record true. A composition holding the input is a new context with an
   identifier of its own, and holds its part by reference rather than by copy
   (``DEC_COMPOSITION_BY_REFERENCE``), so the input stays addressable as what it
   was. The same measurement took that shape as its control, and it was accepted.

.. feat_req:: One identifier names one context in a run
   :id: FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER
   :derived_from: STKH_PROVENANCE
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall hold no two different contexts under one identifier in a run.

   The parent records which context each byte of a node's input came from, and
   every such record is kept, and every such question asked, by identifier. Two
   contexts under one identifier make each answer about either of them wrong,
   and wrong in the way that is not noticed: the walk over a result's lineage
   reported one context where there were two, and nothing failed.

   It can be false while the parent holds, and was measured false twice
   (``EVD_RUN_ARGUMENTS_SHARE_IDENTIFIER``, ``EVD_RUN_PART_SHARES_IDENTIFIER``).
   ``FEAT_RUN_OUTPUT_IS_NEW`` was already met in both: every output's own
   identifier was new. What it does not reach is the contexts an output is
   composed of, nor the arguments a run starts with, and those are where the
   second context came in.

   One identifier source per run would prevent it, and a run cannot insist on
   one, since what it is started with was made before it existed
   (``DEC_IDENTIFIER_NAMES_ONE_CONTEXT``). So it is held where contexts enter.

.. feat_arch:: Running splits into a run and a scheduler
   :id: ARCH_RUN
   :realises: FEAT_RUN_REFUSES_INVALID_WIRING, FEAT_RUN_ENTRY_SOURCE_EXACT, FEAT_RUN_WAITS_FOR_EVERY_BINDING, FEAT_RUN_BUDGET_STOPS, FEAT_RUN_QUIESCENCE_ENDS, FEAT_RUN_FAILURE_CARRIED, FEAT_RUN_COMPLETES_ON_DESIGNATED, FEAT_RUN_OUTPUT_OF_DECLARED_TYPE, FEAT_RUN_OUTPUT_IS_NEW, FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER
   :uses: COMP_WORKFLOW_RUN, COMP_RUN_SCHEDULER
   :statement: Agconflo shall allocate running a workflow to the workflow run and the run scheduler.

   Two components, each answerable for what the other cannot guarantee:

   - The workflow run answers for what becomes of a run. The refusals before it
     starts, the budget it is held to, the outputs it refuses, and each of the
     four ways it may end are statements about one run's history, which nothing
     that reads a definition can make. Whether an output's identifier is already
     held is a question about exactly that history.
   - The run scheduler answers for which instance may activate and what that
     activation carries. That is a question about a definition and the outputs
     produced so far, with no history in it, and it fails in ways a run's
     bookkeeping does not - the one measured failure of this slice
     (``EVD_RUN_OPTIONAL_BY_ORDER``) is entirely inside it.

   ``FEAT_RUN_QUIESCENCE_ENDS`` is the one requirement split across both, and the
   split is the clearest statement of the division. That no instance can activate
   is the scheduler's answer, reached by asking every instance. That the run
   therefore ends, with no result, is the run's - and it depends on the
   designated output having produced nothing, which the scheduler has no reason
   to know.

   A third component was planned and dropped, which is worth recording because
   the reasoning is the test the other two passed. A node activation, as a value
   beside the two in the shape of ``COMP_WIRING_DEFECT``, turns out to answer for
   nothing: there is no requirement that is true or false of one activation taken
   alone. What an activation carries cannot usefully be separated from deciding
   that its instance is ready, because knowing every bound parameter has a
   context is gathering them: the prototype answered both in one pass, and
   splitting them would mean walking the same bindings twice to reach the same
   answer. The wiring defect earns its place by exactly the opposite test - what
   a defect says about itself is true independently of what found it.

   The decisions this is built against are named here rather than linked:

   - ``DEC_RUN_IS_DRIVEN``: the run hands its caller one activation at a time and
     takes back a context or a failure. Neither component performs an activation,
     which is what keeps a provider, a script and a runtime out of this feature.
   - ``DEC_EVERY_INPUT_REQUIRED``, superseding ``DEC_BINDING_IS_AWAITED``:
     every parameter a type declares is required, bound and waited for.
   - ``DEC_ARGUMENTS_PER_ENTRY``: an argument is addressed by the entry instance
     and parameter it fills, so two entry instances declaring one name are two
     parameters.
   - ``DEC_RUN_REFUSED_BEFORE_IT_STARTS``: a workflow with a defect, and a
     signature whose parameters do not each have exactly one source, are refusals
     rather than endings.
   - ``DEC_RUN_ENDS_ONE_WAY``: the four endings are closed, so the run has one
     answer to what became of it.
   - ``DEC_BUDGET_COUNTS_ACTIVATIONS``: the budget is a count the run keeps,
     which it can keep exactly because every activation passes through it.
   - ``DEC_COMPLETION_IS_DESIGNATED_OUTPUT``: completion and quiescence are both
     questions about the designated output rather than about the graph.
   - ``DEC_ACTIVATION_ONCE_PER_RUN``, since superseded by
     ``DEC_EDGE_GENERATIONS``: no instance activated twice, so the scheduler
     needed no activation tag and the run no epoch.
   - ``DEC_REFUSED_OUTPUT_OUTSTANDING``: an output the run refuses leaves its
     activation outstanding, so the four endings stay closed and the caller
     answers again or reports a failure of its own.
   - ``DEC_IDENTIFIER_NAMES_ONE_CONTEXT``: a second context under a held
     identifier is refused where it enters, at the start and at each output.

   Two absences, recorded so they are not read as oversights. There is no run log
   component: what a run records for provenance is ``STKH_PROVENANCE``'s business
   and belongs to a feature that has a run to record. And no component supplies a
   global context, because a node type declares the global types it reads and
   nothing yet provides them.

   ``FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER`` was added to what it realises in #22;
   kept under ``DEC_CHANGE_ONE_CONTEXT_PER_IDENTIFIER_PLACED``.
