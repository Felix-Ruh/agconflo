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
No instance activates twice (``DEC_ACTIVATION_ONCE_PER_RUN``), so there are no
loops, no routers and no activation tagging. A run reads no global context, and
nothing is persisted.

Each requirement was checked by hand against the question no rule can ask: could
this be false while its parent is true? The body of each says how. One of them
is unusually easy to answer, and it is worth saying why rather than leaving it
to be noticed: the crate today satisfies every one of the seven requirements
that say what makes a workflow invalid, and there is no run to refuse anything,
because nothing calls the validator. A goal met by a judgement nobody asks for
is the gap this feature closes.

Five of the seven statements below use the ``unwanted`` pattern and one uses
``event``. ``DEC_EARS_SIX_PATTERNS`` predicted that distribution would arrive
with a feature that has real triggers, and asked to be looked at again when one
had been written. Nothing here was hard to classify and none of the six is still
unused in a way that argues for a change, so the decision is left as it stands.

The feature's architecture closes the file. It realises all seven requirements
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

   ``DEC_BINDING_IS_AWAITED`` settles it in the other direction, and the reading
   of "optional" it rests on is the one ``DEC_DECLARED_PARAMETERS`` already
   wrote down: optional says what a definition may leave unwired, not what may
   arrive late.

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
