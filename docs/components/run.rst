================================
Components of running a workflow
================================

The two parts ``ARCH_RUN`` divides running a workflow into, and the requirements
allocated to each. A component is an object rather than a level: nothing derives
from it, and it exists so that a requirement has one subject answerable for it -
which is also what lets a requirement's grammatical subject be checked against
the title of the component it is allocated to.

Each title below is that subject, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

.. comp:: Workflow run
   :id: COMP_WORKFLOW_RUN
   :crate: agconflo-core

   One run of one workflow definition, from what it was started with to the one
   way it ended. It holds what has been produced, how many activations have been
   spent, and which activation is outstanding.

   Everything about a run's history belongs here: the refusals that stop one
   before it starts, the outputs it refuses once it has, the count the budget is
   measured against, and each of the four endings. None of them can be decided by
   reading a definition, which is what separates this from the scheduler it asks.

   It performs no activation (``DEC_RUN_IS_DRIVEN``). It hands one out, takes
   back a context or a failure, and that is the whole of its contact with node
   behaviour.

.. comp:: Run scheduler
   :id: COMP_RUN_SCHEDULER
   :crate: agconflo-core

   The walk that answers, from a definition and the outputs produced so far,
   which instance may activate next and what that activation is given. It has no
   history in it: the same definition and the same produced outputs give the same
   answer, however the run reached them.

   Its requirements are about readiness and about what an activation carries,
   and they are true or false of that answer taken alone - which is what
   separates it from the run that keeps asking.

   This is where the one measured failure of the slice lives
   (``EVD_RUN_OPTIONAL_BY_ORDER``): a walk that asks readiness of the required
   parameters alone lets an instance activate before an optional parameter's
   source has produced, and which instance that is depends on the order the
   definition carries them in.

.. comp_req:: An instance is offered only once everything it binds has arrived
   :id: CREQ_SCHEDULER_READY_WHEN_BOUND
   :derived_from: FEAT_RUN_WAITS_FOR_EVERY_BINDING
   :allocated_to: COMP_RUN_SCHEDULER
   :ears_pattern: ubiquitous
   :statement: Run scheduler shall offer an instance for activation only once every parameter that instance binds has a context.

   Readiness is asked of what an instance binds, not of what its node type
   requires (``DEC_BINDING_IS_AWAITED``). An entry instance's parameters are
   filled from the run's arguments instead, which are present before the run
   starts or it does not start at all.

   Failure modes, each of which produces a well-formed wrong answer rather than
   an error:

   - **Readiness asked of the required parameters alone.** The measured one
     (``EVD_RUN_OPTIONAL_BY_ORDER``). An instance is offered before an optional
     parameter's source has produced, and is given a strict subset of what was
     wired to it. Nothing reports it, both runs complete, and which subset
     arrives depends on the order the definition carries its instances in.
   - **Readiness asked of every parameter the type declares.** The opposite
     error, and it deadlocks instead: an optional parameter that the definition
     deliberately leaves unbound never gets a context, so the instance is never
     offered and the run is quiescent.
   - **A binding whose source is not an instance treated as filled.** A source
     naming nothing cannot arrive, so an instance bound to it is never ready.
     Treating an unresolvable name as absent and therefore ignorable would offer
     the instance with a parameter missing.
   - **An instance offered again after producing.** No instance activates twice
     (``DEC_ACTIVATION_ONCE_PER_RUN``), and an instance that has produced has a
     context for everything it binds, so this is the shape that loops forever.

   Must pass unoffered and unreported: an instance whose optional parameter is
   unbound in the definition, which is ready as soon as its bound parameters have
   contexts, and an entry instance, which is ready from the start.

.. comp_req:: An activation carries what was wired to its instance
   :id: CREQ_SCHEDULER_ACTIVATION_CARRIES
   :derived_from: FEAT_RUN_WAITS_FOR_EVERY_BINDING
   :allocated_to: COMP_RUN_SCHEDULER
   :ears_pattern: ubiquitous
   :statement: Run scheduler shall give each activation it offers the context bound to every parameter of its instance, in the order that instance's node type declares them.

   The other half of readiness, and inseparable from it: knowing every bound
   parameter has a context is gathering them. Declared order is what lets a node
   assemble its own inputs (``DEC_DECLARED_PARAMETERS``), so it is part of what
   an activation is rather than a convenience.

   Failure modes:

   - **Parameters in binding order rather than declared order.** A definition may
     write bindings in any order, so the node receives its inputs shuffled and
     assembles a different prompt from the same workflow.
   - **Parameters sorted by name.** Deterministic and still wrong, and it passes
     every test whose parameters happen to be declared alphabetically.
   - **A context reaching a parameter it was not bound to.** Two parameters of
     one type make this invisible to any type check.
   - **A global context included.** A node type declares the global types it
     reads and nothing supplies them, so anything arriving under that heading was
     invented.
   - **An optional parameter bound but omitted from the activation.**
     ``CREQ_SCHEDULER_READY_WHEN_BOUND`` waited for it, and dropping it here
     throws away what the waiting was for.

.. comp_req:: The scheduler says when nothing may activate
   :id: CREQ_SCHEDULER_NONE_READY
   :derived_from: FEAT_RUN_QUIESCENCE_ENDS
   :allocated_to: COMP_RUN_SCHEDULER
   :ears_pattern: unwanted
   :statement: If every instance of a workflow has either produced an output or lacks a context for a parameter it binds, then Run scheduler shall report that no instance may activate.

   Quiescence is reached by asking every instance rather than by inspecting the
   graph, which is what makes it true of cycles nobody detected and of workflows
   whose arguments were fine. The run decides what to do about it
   (``CREQ_RUN_ENDS_QUIESCENT``); the scheduler only reports the fact.

   Failure modes:

   - **Reported while an instance is still offerable.** The run then ends a
     workflow that could have produced its result, which is the worst outcome
     available here because the result was reachable.
   - **Never reported, because a produced instance stays offerable.** The run
     cycles forever over instances that have already run, and the budget becomes
     the only thing that ends it - a sound run reported as a runaway.
   - **Reported only when no instance has any context at all.** A cycle whose
     instances each hold some of their inputs is then never quiescent.
   - **Asked of the designated instance alone.** That is the run's question, not
     this one, and it would report quiescence on a workflow with work left to do.

.. comp_req:: A run of a defective workflow does not start
   :id: CREQ_RUN_REFUSES_DEFECTS
   :derived_from: FEAT_RUN_REFUSES_INVALID_WIRING
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If a workflow carries a wiring defect, then Workflow run shall refuse to start a run of that workflow.

   The enforcement half of ``STKH_WIRING_CHECKED``, and the first place in this
   project where a refusal can come before anything has happened.

   Failure modes:

   - **The validator is never asked.** The state the crate is in today, and the
     one this requirement exists to leave. Everything looks correct, because a
     workflow with defects runs until it hits one.
   - **Asked after the first activation.** A node has run, which is precisely
     what the parent goal forbids, and for a node that writes to the world the
     refusal comes too late to matter.
   - **Asked, and the answer discarded when it is not empty.** A green run on a
     defective workflow is indistinguishable from a green run on a sound one.
   - **A refusal reported as an ending.** A caller handling the four endings
     would treat a workflow it must fix as a run that happened
     (``DEC_RUN_REFUSED_BEFORE_IT_STARTS``).

.. comp_req:: A refusal carries every defect there is
   :id: CREQ_RUN_REFUSAL_NAMES_EVERY_DEFECT
   :derived_from: FEAT_RUN_REFUSES_INVALID_WIRING
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: ubiquitous
   :statement: Workflow run shall carry every wiring defect of a workflow in its refusal to run that workflow.

   ``FEAT_WIRING_ALL_DEFECTS`` already requires the validator to report all of
   them, and a run that forwards one of five undoes that at the last step. The
   author fixes one defect, is refused again, and learns the workflow's faults
   one round trip at a time - which is the failure ``STKH_MACHINE_AUTHORING``
   cares about most, since an agent correcting its own workflow pays that cost
   per defect.

   Failure modes:

   - **The first defect only.** The tidy-looking version, and the one that turns
     a single refusal into five.
   - **Defects deduplicated by instance or by parameter.** Five parameters bound
     to one deleted instance are five wires to repoint, and this is the shape
     ``CREQ_VALIDATOR_BINDING_RESOLVES`` already argues against collapsing.
   - **Defects rendered into one message.** A caller then parses prose to learn
     what to fix, which is what carrying them as values exists to avoid.

.. comp_req:: A run whose signature is not filled does not start
   :id: CREQ_RUN_REFUSES_UNFILLED_SIGNATURE
   :derived_from: FEAT_RUN_ENTRY_SOURCE_EXACT
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If an entry parameter of a workflow is not filled by exactly one context of its declared type, then Workflow run shall refuse to start a run of that workflow.

   Arguments are addressed by the entry instance and parameter each fills
   (``DEC_ARGUMENTS_PER_ENTRY``), so this is a question about pairs rather than
   about names, and two entry instances declaring one parameter name are two
   parameters to fill.

   Failure modes:

   - **A required entry parameter with no argument.** Measured reading as a stuck
     run (``EVD_RUN_MISSING_ARGUMENT_QUIESCES``), which names every waiting
     instance instead of the one argument to supply.
   - **An argument of a context type the declaration does not name.** The wiring
     validator refuses this between two instances and cannot see it here, because
     there is no binding to look at.
   - **An entry parameter both supplied and bound.** Sound wiring, recorded among
     the open shapes in ``components/wiring``, and what a loop closing back onto
     an entry node draws. Taking the argument and ignoring the wire is what the
     prototype did, silently.
   - **An argument naming an instance or parameter the workflow does not have.**
     A typo in a parameter name then leaves the real parameter unfilled while the
     caller believes it supplied it.
   - **An argument for an optional entry parameter treated as required.** An
     entry instance's optional parameter may legitimately go unsupplied, and
     refusing it would make a workflow unusable that is not wrong.

   Must pass unreported: one argument per required entry parameter of the
   declared type, and two entry instances of the same node type, whose parameters
   share names and are two parameters all the same.

.. comp_req:: A run stops at its budget rather than past it
   :id: CREQ_RUN_STOPS_AT_BUDGET
   :derived_from: FEAT_RUN_BUDGET_STOPS
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If a run has activated as many instances as its budget allows, then Workflow run shall end that run without offering another activation.

   The count is of activations, because that is what costs money
   (``DEC_BUDGET_COUNTS_ACTIVATIONS``), and the run keeps it exactly because
   every activation passes through it (``DEC_RUN_IS_DRIVEN``).

   Failure modes:

   - **Checked after the activation is offered.** One activation over budget on
     every run that reaches the limit, which for an LLM node is one unbudgeted
     call, every time.
   - **The outstanding activation counted before it is reported.** The mirror of
     the above, ending a run one activation early and reporting a budget failure
     on a run that was within it.
   - **Counted per instance rather than per activation.** Indistinguishable while
     no instance activates twice, and wrong the moment loops exist - which is
     exactly the point at which the budget starts to matter.
   - **A budget of nought treated as unbounded.** A run that may activate nothing
     is a legitimate thing to ask for, and reading it as no limit inverts the
     requirement.
   - **The budget ending a run whose result already exists.** Completion is
     checked first, or a run that finished on its last permitted activation is
     reported as a runaway.

.. comp_req:: A run that can do nothing more ends and says what was waiting
   :id: CREQ_RUN_ENDS_QUIESCENT
   :derived_from: FEAT_RUN_QUIESCENCE_ENDS
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If no instance of a run may activate and its designated instance has produced no output, then Workflow run shall end that run naming every instance that produced none.

   Both halves are needed and they come from different places. That nothing may
   activate is the scheduler's answer (``CREQ_SCHEDULER_NONE_READY``). That the
   designated instance produced nothing is the run's, and it is what separates a
   stuck run from a finished one (``DEC_COMPLETION_IS_DESIGNATED_OUTPUT``).

   Failure modes:

   - **Quiescence alone treated as stuck.** Measured as the shape that makes this
     wrong (``EVD_RUN_COMPLETES_WITH_IDLE``): a workflow whose result is produced
     while an instance that can never run sits idle is finished, and reporting it
     as stuck would fire on every workflow with a branch the result does not
     depend on.
   - **The run left open rather than ended.** A caller asks again and nothing
     changes, because nothing is running that could change it.
   - **Ended without naming what was waiting.** The instances that produced
     nothing are what a person reads to find the cycle or the missing wire, and a
     bare report of stuckness sends them to the whole graph.
   - **Naming instances that did produce.** The list then includes nodes that did
     their work, which points at the wrong part of the workflow.

.. comp_req:: A failed activation ends the run with its failure
   :id: CREQ_RUN_ENDS_ON_FAILURE
   :derived_from: FEAT_RUN_FAILURE_CARRIED
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If an activation is reported as failed, then Workflow run shall end that run carrying the reported failure and the instance it was reported for.

   The caller performs the activation and reports back either a context or a
   failure (``DEC_RUN_IS_DRIVEN``), so the failure is a value the run receives.
   What it was is the caller's to say; that it ends the run, and that the ending
   carries it, is this requirement's.

   Failure modes:

   - **The run continues with whatever else is ready.** Further activations are
     spent on a run whose result is unreachable, and for an LLM node each one
     costs.
   - **The failure rendered into a message.** A caller cannot assert on which
     failure occurred, which is what every error-path test in this project is
     required to do, and it is the exact defect ``STKH_TYPED_FAILURE`` supersedes
     a panic for.
   - **The instance not named.** One failure in a workflow of twenty nodes says
     nothing about where to look, and two instances of one node type are
     indistinguishable without it.
   - **A failure reported for no outstanding activation.** Nothing was offered,
     so there is no instance to name, and inventing one would file the failure
     against a node that never ran.

.. comp_req:: A completed run's result is its designated output
   :id: CREQ_RUN_COMPLETES
   :derived_from: FEAT_RUN_COMPLETES_ON_DESIGNATED
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: event
   :statement: When the instance a workflow designates has produced its output, Workflow run shall end that run with that context before offering another activation.

   Exactly one instance is designated and it names one that exists, both of which
   the validator has already refused a workflow for, so by the time a run starts
   the designation resolves.

   What this adds to its parent is the moment. The parent says what a completed
   run ends with; this says the run ends there rather than carrying on and
   arriving at the same result later, which is the failure below that no
   assertion on the result alone can catch.

   Failure modes:

   - **The run continues until nothing is ready.** The same result, reached after
     spending activations on nodes nothing was waiting for.
   - **Completion checked before the designated instance has produced.** A run
     ends with no result and calls it success.
   - **The last context produced returned as the result.** Correct whenever the
     designated instance happens to activate last, and wrong for every workflow
     with a branch beside the result.
   - **The result returned by identifier rather than as the context.** A caller
     binding a workflow into another as a node needs the value, and an identifier
     is only usable by something holding the run.

.. comp_req:: An output of a type its node type does not declare is refused
   :id: CREQ_RUN_REFUSES_UNDECLARED_OUTPUT
   :derived_from: FEAT_RUN_OUTPUT_OF_DECLARED_TYPE
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If an output reported for an activation is not of the context type its instance's node type declares, then Workflow run shall refuse that output naming the instance, the declared type and the reported type.

   The comparison is with the declaration, which the run can read, rather than
   with whatever consumes the output, which it need not have: the designated
   instance's output goes to the caller, and nothing in the workflow need
   consume it.

   Failure modes:

   - **Not compared at all.** The state measured
     (``EVD_RUN_ACCEPTS_UNDECLARED_OUTPUT``): the output is accepted, the
     consumer is handed a context of a type its parameter was never declared for,
     and the run completes.
   - **Compared with the consuming parameters' types.** Agrees with the
     declaration on every sound workflow whose output has a consumer, and checks
     nothing for the designated instance, whose result a caller binding this
     workflow into another is about to rely on.
   - **Compared with the type of the instance's inputs.** A node that passes its
     input's type on happens to agree, and every node whose type changes the type
     - which is most of them - is refused for being right.
   - **Refused without saying why.** A caller that learns only that its output
     was refused cannot tell a wrong type from a reused identifier, and has two
     things to go and look at.
   - **The output recorded before it is compared.** A refusal reported after the
     output has been stored is the measured defect with a message beside it.

   Must pass unreported: an output of the declared type, including a composition
   whose parts are of other types, since a composition's type is the one it was
   declared with whatever its parts' types are (``CREQ_VALUE_DECLARED_TYPE``).

.. comp_req:: An output the run already holds is refused
   :id: CREQ_RUN_REFUSES_HELD_IDENTIFIER
   :derived_from: FEAT_RUN_OUTPUT_IS_NEW
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If an output reported for an activation carries an identifier the run holds, then Workflow run shall refuse that output naming the instance and the identifier.

   What a run holds is its arguments, the outputs it has accepted, and every
   context any of them was composed from. A context the caller built and never
   reported is not held, and an identifier it carries is one no activation has
   yet been credited with.

   The statement first named only arguments and accepted outputs, because that
   was all a run kept. Once a run kept everything they were composed from
   (``CREQ_RUN_REFUSES_SHARED_OUTPUT_IDENTIFIER``), a part of an input handed
   back unchanged became the one held context the old wording let through,
   credited to a node that did not make it. The parent already said "an
   identifier the run already holds"; this now says the same.

   Failure modes:

   - **Not compared at all.** Measured (``EVD_RUN_ACCEPTS_HELD_IDENTIFIER``): an
     argument passed through, and another instance's output passed through, were
     both accepted and the run completed.
   - **Outputs compared and arguments not.** The first half of the measurement
     then still passes: an entry instance handing back its own argument is
     credited with a context the caller made.
   - **Compared with the instance's own inputs alone.** A caller holding any
     context of the run can hand it back, including the output of an instance
     that is not wired to this one, and nothing about the wiring stops it.
   - **A composition holding a held context refused.** Its identifier is new and
     its parts are held by reference, which is the one sanctioned way to pass an
     input on; refusing it leaves no way at all.
   - **Refused without naming the identifier.** Which context was reused is what
     tells the caller whether it passed through an input or confused two
     activations.

   Must pass unreported: an output composed of the instance's inputs, and an
   output built from nothing the run holds.

   Its statement was widened in #22 from "the identifier of an argument or of
   an output the run has accepted" to "an identifier the run holds", which is
   its parent's own word; kept under ``DEC_CHANGE_RUN_REFUSES_HELD_IDENTIFIER``.

.. comp_req:: A refused output leaves its activation outstanding
   :id: CREQ_RUN_REFUSED_OUTPUT_OUTSTANDING
   :derived_from: FEAT_RUN_OUTPUT_OF_DECLARED_TYPE, FEAT_RUN_OUTPUT_IS_NEW, FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If an output reported for an activation is refused, then Workflow run shall keep that activation outstanding without counting it against the budget again.

   ``DEC_REFUSED_OUTPUT_OUTSTANDING`` is why: the four endings stay closed, and
   the caller does one of the two things it already can - reports an output the
   run accepts, or reports the activation failed.

   Failure modes:

   - **The run ended by the refusal.** There is no ending to end it with: a
     node's failure carries the caller's failure type, which the run cannot make.
   - **The activation dropped.** The instance has not produced, so the scheduler
     offers it again as a new activation, and the same node is counted twice
     against the budget - a run within its budget reported as a runaway.
   - **A failure reported after a refusal refused.** The activation is still
     outstanding, so a caller giving up on it must be able to say so, and being
     told nothing is outstanding would leave it no way to end the run.

   ``FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER`` became its third parent in #22, and
   its statement suits all three; kept under
   ``DEC_CHANGE_ONE_CONTEXT_PER_IDENTIFIER_PLACED``.

.. comp_req:: Arguments sharing an identifier stop the run starting
   :id: CREQ_RUN_REFUSES_SHARED_ARGUMENT_IDENTIFIER
   :derived_from: FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If two different contexts among a run's arguments and the contexts they were composed from share an identifier, then Workflow run shall refuse to start that run naming every identifier they share.

   Arguments are made before the run exists, so they are where a second source
   enters first (``EVD_RUN_ARGUMENTS_SHARE_IDENTIFIER``). "Different" is the
   value's identity: one context supplied to two parameters is one context, and
   is not refused (``DEC_IDENTIFIER_NAMES_ONE_CONTEXT``).

   Asked after the wiring and the signature, as a third refusal before the run
   starts: a run refused for its wiring has no arguments worth checking against
   each other.

   Failure modes:

   - **Not asked.** Measured: the run started, completed, and its result's
     lineage lost an argument.
   - **Only the arguments themselves compared.** Two arguments with different
     identifiers, one composed of a context that repeats the other's, pass it,
     and the lineage is as wrong as before.
   - **The same context supplied twice refused.** Passing one context to two
     entry parameters is ordinary, and it is one context under one identifier.
   - **The first shared identifier only.** A caller whose arguments came from two
     sources has as many collisions as the smaller source issued, and learns
     them one refusal at a time.
   - **Reported as a signature fault.** Every signature fault names an entry
     instance and a parameter; a shared identifier belongs to no one parameter,
     and filing it under one sends the caller to the wrong place.

.. comp_req:: An output bringing in a second context under a held identifier is refused
   :id: CREQ_RUN_REFUSES_SHARED_OUTPUT_IDENTIFIER
   :derived_from: FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If a context an output was composed from shares its identifier with a different context of the run or of that output, then Workflow run shall refuse that output naming the instance and the identifier.

   ``CREQ_RUN_REFUSES_HELD_IDENTIFIER`` asks about the output's own identifier;
   this asks about everything the output was composed from, which is where the
   measured part came in (``EVD_RUN_PART_SHARES_IDENTIFIER``). What the run holds
   grows by everything an accepted output was composed from, so a later output
   is compared with those too.

   A context the run already holds, reached by reference, is not a second
   context, and the walk stops there: its own parts were compared when it was
   accepted, so each output costs what it brings in rather than its whole
   ancestry.

   Failure modes:

   - **Only the output's own identifier asked.** The measured shape: a new part
     under an argument's identifier, accepted.
   - **Compared with the arguments alone.** A part repeating an earlier output's
     part passes, and so does one repeating a part of an argument.
   - **Compared with the run and not within the output.** Two new parts under
     one identifier, both new to the run, pass each other.
   - **A held context reached by reference refused.** An output composing its
     input, or a part of its input, holds contexts the run holds already, and
     they are the same contexts. Refusing them refuses the sanctioned way to
     pass an input on (``DEC_COMPOSITION_BY_REFERENCE``).
   - **What an accepted output brought in forgotten.** The run then compares the
     next output with its arguments and outputs, and not with the parts those
     outputs were composed from.

   Must pass unreported: an output composed of its inputs, of their parts, and of
   new contexts from the run's one source.
