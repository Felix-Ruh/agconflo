==================
Running a workflow
==================

What happens when a workflow actually runs: what it can be made of, what it may
integrate with, and the ways a run is allowed to end. Two of these constrain each
other, and the bodies say where.

.. stkh_req:: The provider is not baked in
   :id: STKH_PROVIDER_CHOICE
   :stakeholder: user
   :statement: Agconflo shall let the same workflow run against different LLM providers.

   A workflow encodes reasoning, not a vendor relationship. The model that suits
   a step today may be the wrong price or the wrong shape tomorrow, and rewiring
   the pipeline to follow it would mean the pipeline was never the asset.

   Providers do differ in ways that matter - caching regimes and tool-call
   protocols among them. The requirement is not that those differences vanish,
   but that they stop at the engine rather than reaching the workflow.

.. stkh_req:: A person can take part in a run
   :id: STKH_HUMAN_IN_RUN
   :stakeholder: user
   :statement: Agconflo shall let a person supply a context while a run is in progress.

   Review, approval and correction are steps in real pipelines rather than
   exceptions to them. Splitting a workflow at every human touchpoint into
   separate runs would fragment the provenance chain, which defeats the point of
   having one.

   This constrains the stuck-run report below: a run parked awaiting a person is
   live, not deadlocked, and the two requirements have to agree about that.

.. stkh_req:: A workflow is invocable as a node
   :id: STKH_WORKFLOW_AS_NODE
   :stakeholder: user
   :statement: Agconflo shall let a workflow be invoked as a node of another workflow.

   Without it every reusable sequence has to be copied, and the copies drift.

   Three consequences follow and none is optional: a workflow needs a declared
   signature, so its entry nodes are a typed parameter list and exactly one
   terminal output is designated; the sub-workflow must not become an opaque box,
   so its internal activations appear in the run log scoped under the invoking
   one, or composition silently destroys the guarantee it was added to serve; and
   recursion becomes possible, for which the step budget below is the valve.

.. stkh_req:: A run survives an interruption
   :id: STKH_RESUMABLE_RUN
   :stakeholder: user
   :statement: Agconflo shall resume a run that was interrupted before it finished.

   Load-bearing rather than a nicety, and it is the human gate that makes it so. A
   run may sit awaiting a person for hours or days, and a process restart inside
   that window must not throw the work away.

.. stkh_req:: A broken graph never starts
   :id: STKH_WIRING_CHECKED
   :stakeholder: user
   :statement: Agconflo shall reject an invalid workflow before any node in it runs.

   Adjacent to the requirement that topology is data validated without executing
   it, and the boundary is worth stating because the two are easily read as one.
   That one is about the representation: the format permits checking without
   running. This one is about enforcement and timing: the engine actually
   refuses, and refuses before anything has happened.

   Either can hold without the other. A topology stored as data that nobody ever
   checks is perfectly possible, which is what this requirement rules out. The
   value is that a wiring mistake costs a rejection rather than half of an
   expensive run.

.. stkh_req:: A runaway run is stopped
   :id: STKH_STEP_BUDGET
   :stakeholder: user
   :statement: Agconflo shall stop a run that exceeds its configured step budget.

   Back-edges are allowed and loops are legal, so a runaway is a question of when
   rather than whether - and a runaway loop through an LLM spends real money
   while it happens. This is also the valve for the recursion that invoking a
   workflow as a node makes possible, which is why no separate mechanism is
   wanted for it.

.. stkh_req:: A stuck run is reported
   :id: STKH_STUCK_RUN
   :stakeholder: user
   :statement: Agconflo shall report a run in which no node can make further progress.

   The counterpart to the budget above: that one catches a run doing too much,
   this one a run doing nothing.

   One caveat decides whether it is correct at all. A node awaiting something
   external - a person, a network call - has to count as live rather than
   blocked, or a run legitimately parked on a human gate is reported as
   deadlocked. That is the agreement this requirement owes to the human-in-the-run
   requirement above.

.. stkh_req:: A failure says which failure it was
   :id: STKH_TYPED_FAILURE
   :stakeholder: maintainer
   :statement: Agconflo shall report which failure occurred when a node fails.

   This supersedes a placeholder rather than filling a gap. The design so far has
   an erroring node panic and the whole run stop, with no retries, no timeouts and
   no error edges - recorded openly as something to revisit once anything actually
   ran. A panic cannot say which failure occurred.

   The testing policy is what forces the issue: every expected failure mode gets a
   test asserting which error happened and how the system behaved afterwards,
   because a failure mode nobody can assert on is a failure mode nobody designed.
   That is impossible against a panic, so a real typed error model is needed
   earlier than the placeholder assumed, and this is the requirement every
   error-path test case is written against.
