==================
Running a workflow
==================

What happens when a workflow actually runs: what it can be made of, which way
it goes and how often it goes round, what it may integrate with, the ways a run
is allowed to end, and what a person needs to run one. Several of these
constrain each other, or a goal in ``stakeholder/context``, and the bodies say
where.

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

   Three consequences follow and none is optional: a workflow needs a
   signature - the typed parameters nothing in it binds, and exactly one
   designated output; the sub-workflow must not become an opaque box,
   so its internal activations appear in the run log scoped under the invoking
   one, or composition silently destroys the guarantee it was added to serve; and
   recursion becomes possible, for which the step budget below is the valve.

.. stkh_req:: A run starts from contexts given for any node
   :id: STKH_RUN_FROM_ANY_PARAMETER
   :stakeholder: user
   :statement: Agconflo shall let a run start from contexts given for the parameters of any of a workflow's nodes.

   Everything a node is given is a context, and what a run is started with is
   no different: contexts, each for one parameter of one node. No node is
   marked as where a run begins. A workflow is invoked by giving each
   parameter nothing in it binds its context, and whichever nodes then hold
   everything they need are the ones that run first.

   This settles where it meets ``STKH_WIRING_CHECKED``, as the maintainer
   decided on 2026-09-26. A parameter nothing binds is not a broken wire: it
   is one of the workflow's inputs, and the workflow is not invalid for having
   it. Leaving one without its context is the invocation's fault, and it is
   refused when the run starts - still before any node runs. A forgotten
   binding therefore shows as an input nobody gave rather than as a defect of
   the graph, which is the cost of not marking inputs.

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

.. stkh_req:: A node chooses the branch a run takes
   :id: STKH_ROUTING
   :stakeholder: user
   :statement: Agconflo shall let a node decide which of the branches after it a run takes.

   A pipeline often goes one of several ways depending on what has happened
   in it: a review that accepts a change goes on to describing it, one that
   rejects it goes back to the draft. Wiring says what can happen, not which
   of it does in a given run. Without this every such choice is made outside
   the workflow - a script doing both branches' work, or a person starting a
   second run - where nothing the run keeps records which way it went.

   The node decides because the choice is made from contexts, and a node is
   what is given contexts (``STKH_EXPLICIT_CONTEXT``). Deciding is a step of
   its own, apart from producing content: ``STKH_ONE_OUTPUT`` keeps the two
   apart as what makes a router a router and a transform a transform, so a
   router passes on the contexts it was given rather than making new ones.

   It names no mechanism: how a branch is taken is a decision's to say. Nor
   does it say what makes the decision: a script, a model or a person each
   performs a node. And a router is not a type Agconflo ships, but a node
   type any workflow declares (``STKH_NO_PRIVILEGED_TYPES``).

   A run already completes whatever instances have not run
   (``DEC_COMPLETION_IS_DESIGNATED_OUTPUT``), so a branch not taken leaves
   nothing unfinished. The stuck-run report below still has to tell the
   instances of a branch not taken from instances that cannot proceed, and
   the two requirements have to agree about that.

.. stkh_req:: A workflow repeats part of itself
   :id: STKH_REPETITION
   :stakeholder: user
   :statement: Agconflo shall let a workflow repeat part of itself until a node decides it is done.

   Retrying a step until a test passes is an ordinary pipeline
   (``DEC_REPETITION_BY_EDGES``), and so are drafting, reviewing and drafting
   again, or fixing and building until the build passes. Without this the
   repetition happens inside one node - a model's own calls within one
   activation - or outside the run, and the workflow no longer shows the
   step that was repeated or how often.

   The end of a repetition is a decision like any other: go round again or go
   on. So it leans on ``STKH_ROUTING``, and a repetition ended by a count is a
   node deciding on a count. It is separate from that goal because routing
   holds without it: branches that never return are useful on their own.

   It names no mechanism: how a part of a workflow is repeated, and how one
   pass's contexts are kept apart from the next's, are decisions' to say.

   It does not bound itself. A repetition whose node never decides it is done
   is what the step budget below stops.

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

.. stkh_req:: A model can yield for more context
   :id: STKH_MODEL_YIELDS
   :stakeholder: user
   :statement: Agconflo shall let a model yield during its node's activation to a node or workflow that node is declared to call and continue with the context that call produces.

   A model working on a step often cannot know in advance everything it will
   need: it looks something up, asks a tool, consults another workflow, and
   carries on with what it got. Wiring can only supply what was known when the
   workflow was written, so without this every such step is either impossible
   or done outside the engine, where nothing records it.

   The shape is a yield rather than a loop inside the node, and that is the
   point of the requirement rather than a detail of it. An earlier view had an
   LLM node run its own tool loop and record each call and result as it went.
   A yield hands control back to the run instead: the thing called runs as a
   step of the run, its output is a context like any other, and the model
   continues with it added. Traceability then comes from how the call happens
   rather than from a log kept beside it, and a call is a point the run can be
   written down at and resumed from, rather than work inside an activation
   that an interruption throws away.

   What may be called is declared for the node, and only that. Which of the
   declared calls a model makes, and when, is the model's choice at run time -
   that is the compromise this requirement makes, and the one it limits.

   This constrains ``STKH_EXPLICIT_CONTEXT``: a node is to be given exactly the
   contexts wired to it, and a context a model yielded for was chosen during
   the run. The two have to agree, and the declaration of what a node may call
   is where they meet. It also leans on ``STKH_STEP_BUDGET``, since a call is
   work the run does, and on ``STKH_WORKFLOW_AS_NODE``, since a workflow is one
   of the things a model may call.

.. stkh_req:: Tools are reached through nodes
   :id: STKH_TOOLS_AS_NODES
   :stakeholder: user
   :statement: Agconflo shall let a model reach an external tool or MCP server only through a node or workflow that wraps it.

   Tools and MCP servers are how models reach the world, and the obvious way to
   support them is as a feature of their own beside the graph. That is exactly
   what is not wanted: a second way for context to reach a model, with its own
   rules, that the graph neither shows nor records.

   Wrapped in a node or a workflow, a tool is called the way anything else a
   model yields to is called, and what it returns is a context with a place in
   the run. The wrapper is ordinary - a node type anyone could write - which is
   what ``STKH_NO_PRIVILEGED_TYPES`` asks of everything Agconflo ships.

   It is separate from the requirement above because either can hold without
   the other. A model could yield to nodes and still be handed tools directly
   on the side, and tools could be wrapped in nodes that no model can call.

.. stkh_req:: A workflow runs from its documents alone
   :id: STKH_RUN_FROM_DOCUMENTS
   :stakeholder: user
   :statement: Agconflo shall let a person run a workflow from the documents that describe it without writing a program that hosts the run.

   A workflow is written as documents: its topology, the node types it uses,
   and the scripts that perform them. Running it has meant writing one thing
   more - a program against the engine's libraries that reads those documents,
   starts the run, keeps its record and hands a person their step - and
   building it before anything runs. Everything a workflow is made of is out
   of reach of the engine's toolchain but the one step that uses it.

   The program meant is the host of a run, not a node's script. A script is
   one of the documents, and writing it is authoring the workflow; what this
   rules out is having to write the host as well.

   Running a workflow includes carrying the run to its end. Resuming it after
   an interruption (``STKH_RESUMABLE_RUN``) and a person answering the step it
   awaits (``STKH_HUMAN_IN_RUN``) are part of running it, so a way of running
   that can start a run but not resume it or take the answer does not meet
   this.

   It is separate from ``STKH_LIVE_BEHAVIOUR`` because either can hold without
   the other. A node's behaviour changes without recompiling the engine while
   every run still needs a host written for it, and a host could be supplied
   for workflows whose behaviour is compiled in. It is also the first thing
   ``STKH_SELF_HOSTING`` asks, which is that the engine be usable before it
   builds itself.

   It names no interface. A command line, a website and a tool a model calls
   can each meet it, and which of them is the main one is a goal of its own.

.. stkh_req:: A tool changes nothing it was not granted
   :id: STKH_TOOLS_CONFINED
   :stakeholder: user
   :statement: Agconflo shall keep a tool a model uses from changing anything outside what the person running the workflow granted it.

   The models this is wanted with are often small local ones, and a small
   model given a tool that can write files or run commands will sooner or
   later use it wrongly: delete a folder it was not asked about, overwrite a
   file it misread, run a command it made up. A workflow that reaches the
   world through tools is only worth running with such a model if its mistakes
   stay inside what the person meant it to touch.

   What is granted is the person's to say when they run the workflow - which
   folders a tool may read, which it may change, what it may run - and not the
   workflow's or the model's. Inside a grant a tool may do anything the grant
   allows, mistakes included; this goal is about everything outside it.

   Accident, not attack. The goal is met against a model that misuses a tool
   it was given, not against code built to break out of whatever confines it -
   an exploit of the operating system's kernel, say. A requirement below it
   that claimed the second would claim more than this goal needs.

   It is separate from ``STKH_TOOLS_AS_NODES`` because either can hold without
   the other. A tool wrapped in a node is reached the way anything else a model
   calls is, and can still change whatever the process running it can; and a
   tool confined to its grant could still be handed to a model directly. It
   leans on ``STKH_NO_PRIVILEGED_TYPES``: granting is the person's for any node
   type alike, one Agconflo ships included, and ``STKH_RUN_FROM_DOCUMENTS``
   says who that person is - the one running the workflow, not a program they
   wrote.

.. stkh_req:: A tool runs in the environment its workflow names
   :id: STKH_TOOL_ENVIRONMENT
   :stakeholder: user
   :statement: Agconflo shall let a workflow name the environment each of its tools runs in, from what the person running it grants.

   Tools differ in what they need to do their work. A node that builds a Rust
   project needs a Rust toolchain, one that runs a project's tests needs its
   interpreter, and one that reads a file needs neither. With one environment
   for every tool of a run, each tool carries everything any of them needs,
   or some of them cannot work at all. And a build and the tests of what it
   built may need to share what the build left, while a third tool should
   see none of it, even where the three use the same environment.

   The workflow names the environment because it knows what its tools do,
   and a workflow handed to someone else says what its tools need in its own
   documents (``STKH_RUN_FROM_DOCUMENTS``). What a tool may touch stays the
   person's to grant (``STKH_TOOLS_CONFINED``), and so does which
   environments a run may use at all: the workflow asks, the person grants.

   It names no mechanism. What an environment is - an image, a container
   shared by name - is for the decisions below it to say.

   It is separate from ``STKH_TOOLS_AS_NODES`` and ``STKH_TOOLS_CONFINED``
   because both can hold without it: tools wrapped in nodes and kept to their
   grant can all run in the one environment the person chose, which is where
   the first slice of tools left them.
