========================
A model yields to a node
========================

A model working inside a node's activation calling another node type for
something it needs, the call running as a step of the run, and the model going
on with what the call produced. Every requirement here derives from a goal in
``stakeholder/execution`` or ``stakeholder/context``, and each is written
against the decisions in ``decisions/yield``.

What this does not do is what keeps it a slice. A model can call node types and
nothing else: a workflow cannot be invoked yet, so it cannot be called, and a
node type reaching a tool outside the engine needs a behaviour a Lua script
cannot be, which is the next slice. A call reaches one level: the node type
called has no instance, so it declares no calls of its own. And a call the
engine refuses ends the activation rather than being explained to the model,
which is left for ``STKH_REFLECTION``.

Much of what a call needs is already required of every activation, in words
that already cover a called node type, and is not said again here. A call
counts against the budget (``FEAT_RUN_BUDGET_STOPS``, which counts nodes
activated), its output is refused if its type contradicts the node type's
declaration (``FEAT_RUN_OUTPUT_OF_DECLARED_TYPE``), a called node type with
nothing to perform it refuses the run before it starts
(``FEAT_BEHAVIOUR_REFUSED_BEFORE_START``, since a call names the node type from
an instance), a person can perform one (``FEAT_PERSON_STEP_HANDED_OVER``), every
request to a model counts against the node's limit
(``FEAT_MODEL_CALLS_LIMITED``, whose model calls are requests, a continuation
after a call among them: ``DEC_EVERY_TURN_COUNTED``), and a workflow's calls
survive being written
back (``FEAT_TOPOLOGY_WRITES``, whose instances carry them). Each gains test
cases in ``tests/yield`` and nothing else.

Each requirement was checked by hand against the two questions every body here
answers: could it be false while its parents hold, and could they hold while it
is false? Three statements are ``unwanted``, two ``ubiquitous`` and two
``event``.

The feature's architecture closes the file. It realises all seven requirements
and names the components they are divided between; ``components/yield`` holds
the requirements this feature allocates to them.

.. feat_req:: A declared call is performed as a step and the model goes on with its output
   :id: FEAT_YIELD_CALL_PERFORMED
   :derived_from: STKH_MODEL_YIELDS
   :ears_pattern: event
   :verification_method: test
   :statement: When a model calls a node type its node's instance declares a call to, Agconflo shall perform that call as a step of the run and continue the model with the call's output.

   The parent's own sentence, at the level where it can be tested: the model
   yields, the thing called runs, and the model continues with the context that
   produced.

   It can be false while the parent's statement holds. A host that performs the
   call itself, inside the activation, lets the model continue with the
   context, and nothing of the run sees it: no budget spent, no record, no
   identifier checked. The parent's body calls that shape the one it exists to
   rule out - "the thing called runs as a step of the run" - so "as a step" is
   the parent's claim rather than more than it.

   It claims no mechanism: not how the node type is performed, not how the
   output reaches the model, not how many calls one answer may make.

.. feat_req:: A call that is not to a declared node type, filled in, is refused
   :id: FEAT_YIELD_UNDECLARED_REFUSED
   :derived_from: STKH_MODEL_YIELDS
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a model makes a call that is not to a node type its node's instance declares a call to or does not fill that node type's parameters with text, then Agconflo shall refuse that call.

   The parent lets a model yield "to a node or workflow that node is declared
   to call", and its body closes it: what may be called is declared for the
   node, "and only that". A call to something else is not one the parent
   allows. A call that leaves a required parameter empty, names one the node
   type does not declare, or gives it something other than text is not a call
   to that node type as declared either: the node type would be activated
   without what its declaration says it needs.

   It can be false while the parent holds, and was measured so: ``genai``
   passes through a name never offered and arguments of the wrong shape
   (``EVD_GENAI_PASSES_MALFORMED_CALLS``), so an engine that performs what comes
   back has let the model reach whatever it names.

   "Refuse" is the whole of it. That a refused call ends the activation is a
   decision (``DEC_MALFORMED_CALL_FAILS``), and sending the refusal back to the
   model would meet this as well.

.. feat_req:: A model is offered no tool but the node types its node may call
   :id: FEAT_YIELD_OFFERS_DECLARED
   :derived_from: STKH_TOOLS_AS_NODES, STKH_EXPLICIT_CONTEXT
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall offer a model no tool but the node types its node's instance declares calls to.

   Two parents, each necessary. ``STKH_TOOLS_AS_NODES`` lets a model reach a
   tool only through a node that wraps it, so nothing but node types may be
   offered: a tool the engine provides beside them is the second way for
   context to reach a model that the parent refuses. ``STKH_EXPLICIT_CONTEXT``
   gives a node exactly what was wired to it, and an offer is text the node's
   model reads. What was wired, for a call, is the declaration:
   ``STKH_MODEL_YIELDS`` names it as the place the two requirements meet. So of
   the node types, only the declared ones. Either parent alone leaves the
   other's half open: every node type in the catalogue offered meets the first,
   and a declared node type offered beside a built-in search meets the second.

   It can be false while the refusal above holds. An offer is not a call, and a
   model offered a node type it may not call is refused when it calls it; what
   the refusal does not keep is the window, which then holds a tool nobody
   declared for the node, and a model that spends a turn finding that out.

.. feat_req:: A call to a node type the workflow lacks is rejected before the run
   :id: FEAT_WIRING_CALL_RESOLVES
   :derived_from: STKH_WIRING_CHECKED
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If an instance of a workflow declares a call to a node type that no node type document supplied with the workflow declares, then Agconflo shall reject that workflow.

   The parent rejects an invalid workflow before any node in it runs. A
   declared call to a node type that does not exist is a workflow that cannot
   do what it says, and the moment it shows is a model calling it, halfway
   through a paid run.

   It can be false while the parent holds, if "invalid" is read as the wiring
   defects there were before calls existed: every one of them absent, and the
   workflow still names something it cannot call. This is ``FEAT_WIRING_BINDING_RESOLVES``
   asked of a call instead of a binding.

.. feat_req:: A call to a node type whose name a provider refuses is rejected before the run
   :id: FEAT_WIRING_CALL_NAME_PORTABLE
   :derived_from: STKH_WIRING_CHECKED, STKH_PROVIDER_CHOICE
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If an instance of a workflow declares a call to a node type whose name OpenAI's or Anthropic's documentation does not allow as a tool's name, then Agconflo shall reject that workflow.

   Two parents, each necessary. ``STKH_PROVIDER_CHOICE`` has the same workflow
   run against different providers, and a node type's name is the tool's name
   (``DEC_TOOL_NAMES_PORTABLE``): a name one provider refuses is a workflow that
   runs on one and not the other. ``STKH_WIRING_CHECKED`` puts the refusal
   before any node runs, where the first provider to refuse the name would put
   it at the first paid call.

   It can be false while each run of the workflow succeeds: run only against a
   local model, which accepts any name (``EVD_LM_STUDIO_TOOL_NAMES_UNCHECKED``),
   a workflow with a name OpenAI refuses never fails anywhere it was tried. The
   parent's claim is about the providers it was not tried on. The two named are
   the two formats this engine speaks (``CREQ_ROSTER_ANSWER_AS_SENT``), and their
   rules were read in their documentation, not measured (``EVD_TOOL_NAME_RULES``).

.. feat_req:: Every model call's window and answer are recorded
   :id: FEAT_MODEL_EXCHANGE_RECORDED
   :derived_from: STKH_PROVENANCE
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall hold in a run's record the contexts each model call of the run was made with and the answer each received.

   The parent records where each byte of a node's input came from, and calls
   itself "the second half" of the question ``STKH_EXPLICIT_CONTEXT`` asks:
   what was in this call's window, and where did every byte come from - the
   half that survives after the run is over. A model call's window is exactly
   what that question is about, and after a call it holds bytes no wiring
   supplied.

   It can be false while the parent holds for every node as it was read before
   calls: each output's inputs recorded, and nothing of what the node sent a
   model or got back. The output composes the answer, so the answer's bytes are
   traced to the answer and stop there; the window that produced it is gone
   with the activation.

   Every model call, not only those that yield: an answer to a prompt is as
   much a byte's origin as an answer after a call.

.. feat_req:: A resumed run asks no model again for an answer its record holds
   :id: FEAT_RESUME_REPEATS_NO_ANSWER
   :derived_from: STKH_RESUMABLE_RUN
   :ears_pattern: event
   :verification_method: test
   :statement: When a run is resumed from its record, Agconflo shall continue it without asking a model again for any answer the record holds.

   The parent resumes an interrupted run so that the work done is not thrown
   away. An answered model call is work done, and paid for.

   It can be false while the parent holds, and was measured so: a run resumed
   after an interruption in the middle of an activation made again the model
   call it had already been answered (``EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS``).
   The run was resumed; the answer was bought twice. ``FEAT_RESUME_REPEATS_NO_OUTPUT``
   rules this out for an activation whose output the record holds, and says
   nothing of one in progress, which is where a call puts a run for as long as
   the called node type takes.

.. feat_arch:: A model yielding splits across the run, its record, the script host and the model roster
   :id: ARCH_YIELD
   :realises: FEAT_YIELD_CALL_PERFORMED, FEAT_YIELD_UNDECLARED_REFUSED, FEAT_YIELD_OFFERS_DECLARED, FEAT_WIRING_CALL_RESOLVES, FEAT_WIRING_CALL_NAME_PORTABLE, FEAT_MODEL_EXCHANGE_RECORDED, FEAT_RESUME_REPEATS_NO_ANSWER
   :uses: COMP_WIRING_VALIDATOR, COMP_WORKFLOW_RUN, COMP_RUN_RECORD, COMP_SCRIPT_HOST, COMP_MODEL_ROSTER
   :statement: Agconflo shall allocate a model's yielding to the wiring validator, the workflow run, the run record, the script host and the model roster.

   Five components the earlier features named, each answerable for the same
   kind of thing it was there:

   - The wiring validator answers for a workflow that cannot run, before it
     runs: a call to a node type that is not there, or whose name a provider
     refuses.
   - The workflow run answers for every activation, and a call is one
     (``DEC_CALL_IS_AN_ACTIVATION``): it refuses a call its instance does not
     declare or does not fill, offers the called node type's activation,
     counts it, and gives its output back to the caller alone.
   - The run record answers for what a record holds and what a resume replays:
     each exchange, and each call among the outputs.
   - The script host answers for one activation: what its model is offered,
     turning a model's answer into calls and the next window, and answering a
     replayed activation's model calls from its record.
   - The model roster answers for a request: sending each part of a window as
     the message its provenance makes it, and each offered node type as a tool,
     and handing back each call an answer makes.

   The topology reader and writer are not among them. Reading an instance's
   calls is part of reading a workflow (``FEAT_TOPOLOGY_READS``), and writing
   them back part of writing one (``FEAT_TOPOLOGY_WRITES``); the requirement
   they need is allocated under that feature. The behaviour set checks a
   called node type as it checks any node type an instance names
   (``FEAT_BEHAVIOUR_REFUSED_BEFORE_START``), with nothing added.

   The decisions this is built against are named here rather than linked:

   - ``DEC_CALLS_DECLARED_ON_THE_INSTANCE``: an instance lists the node types
     its model may call.
   - ``DEC_CALL_IS_AN_ACTIVATION``: the run offers a call as an activation for
     the calling instance.
   - ``DEC_WINDOW_IS_A_CONTEXT``: a window is one context, its parts sent as
     their provenance says.
   - ``DEC_CALL_CARRIES_STRING_VALUES``: a call is its string values, one text
     context each.
   - ``DEC_TOOLS_OFFERED_AS_CONTEXTS``: the offer is contexts made from the
     declaration.
   - ``DEC_TOOL_NAMES_PORTABLE``: the name rule both providers accept.
   - ``DEC_MALFORMED_CALL_FAILS`` and ``DEC_CALLS_IN_ORDER``: what is refused,
     and how several calls go.
   - ``DEC_EVERY_TURN_COUNTED``: every request counts against the limit.
   - ``DEC_RECORD_HOLDS_EXCHANGES``, ``DEC_RESUME_REPLAYS_CALLS``,
     ``DEC_SCRIPT_REPLAYED_FROM_ITS_RECORD`` and
     ``DEC_RECORD_AFTER_EACH_ANSWER``: what the record holds, and how a run
     goes on from it.
