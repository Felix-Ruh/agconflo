==============
Run test cases
==============

How each requirement in ``components/run`` is to be checked. Results are never
written here: they are imported from the test runner, and each case records only
what it asserts.

Every failure mode listed in ``components/run`` has a case that catches it, and
every "must pass unreported" list has a positive case naming its shapes. The two
pull in opposite directions on purpose. Most of this feature's requirements say
what must be refused or must not be offered, and a run that refuses everything
and offers nothing satisfies almost all of them - so the positive cases and
``TEST_RUN_WELL_FORMED_RUNS_COMPLETE`` are the half that keeps the rest honest.

A case's id is the path of the Rust test that implements it, uppercased, so the
scheduler's cases live in one module and the run's in another and a renamed test
breaks the build as a dead link rather than going quietly unrecorded.

Nine failure modes have no case of their own, because each is an interaction
between requirements or an assertion that belongs inside another case. They are
named here so the derivation can be audited rather than taken on trust:

- ``CREQ_SCHEDULER_ACTIVATION_CARRIES``'s "an optional parameter bound but
  omitted from the activation" is asserted by
  ``TEST_SCHEDULER_BOUND_OPTIONAL_IS_AWAITED``, whose whole point is that the
  waiting was for something: it checks that the context arrives in the
  activation and not merely that the instance waited.
- ``CREQ_SCHEDULER_ACTIVATION_CARRIES``'s "parameters sorted by name" is caught
  by ``TEST_SCHEDULER_INPUTS_IN_DECLARED_ORDER``, whose node type declares its
  parameters in an order that is not alphabetical and not the binding order, so
  one case separates three readings that agree on any careless example.
- ``CREQ_SCHEDULER_NONE_READY``'s "never reported, because a produced instance
  stays offerable" is caught by ``TEST_SCHEDULER_PRODUCED_INSTANCE_NOT_OFFERED``,
  which files under readiness because that is where the mistake is made, and
  asserts quiescence because that is where it shows.
- ``CREQ_SCHEDULER_NONE_READY``'s "asked of the designated instance alone" is
  the run's question rather than the scheduler's, and is caught by
  ``TEST_RUN_IDLE_INSTANCE_DOES_NOT_MAKE_IT_STUCK``.
- ``CREQ_RUN_STOPS_AT_BUDGET``'s "counted per instance rather than per
  activation" cannot be distinguished while no instance activates twice
  (``DEC_ACTIVATION_ONCE_PER_RUN``). It is recorded as untestable here rather
  than given a case that would pass against both readings, and it becomes
  testable in the same change that makes a loop runnable.
- ``CREQ_RUN_ENDS_ON_FAILURE``'s "the failure rendered into a message" is
  asserted by every error-path case in this file, each of which matches on the
  ending as a value rather than on its text.
- ``CREQ_RUN_REFUSES_UNDECLARED_OUTPUT``'s "refused without saying why" is
  asserted by ``TEST_RUN_UNDECLARED_OUTPUT_IS_REFUSED``, which matches on the
  refusal's fields, and on which kind of refusal it is.
- ``CREQ_RUN_REFUSES_UNDECLARED_OUTPUT``'s "the output recorded before it is
  compared" is asserted by the same case, which checks that the refused output's
  consumer is not offered next.
- ``CREQ_RUN_REFUSES_HELD_IDENTIFIER``'s "refused without naming the
  identifier" is asserted by both of its passed-through cases, each of which
  matches on the identifier the refusal names.

``DEC_RUN_ENDS_ONE_WAY`` has no case of its own either, and it is a decision
rather than a failure mode, so it is not one of the nine. That a run reaches
exactly one ending is asserted inside each ending's case, which checks that the
other three were not reported, because a standalone property would have no
single requirement to verify.

.. test_case:: A bound optional parameter is waited for and arrives
   :id: TEST_SCHEDULER_BOUND_OPTIONAL_IS_AWAITED
   :verifies: CREQ_SCHEDULER_READY_WHEN_BOUND
   :test_kind: error_path
   :coverage: partial

   The measured defect (``EVD_RUN_OPTIONAL_BY_ORDER``). A consumer whose node
   type declares one required and one optional parameter, both bound, with the
   optional parameter's producer not yet activated: the consumer is not offered.
   Once that producer has produced, the consumer is offered and its activation
   carries both contexts.

   The second half is what stops a reader waiting and then discarding. Asserting
   only that the instance waited would pass against a scheduler that waits and
   then hands over the required parameter alone.

.. test_case:: An instance's inputs do not depend on where it is written
   :id: TEST_SCHEDULER_INPUTS_IGNORE_INSTANCE_ORDER
   :verifies: CREQ_SCHEDULER_READY_WHEN_BOUND
   :test_kind: property
   :coverage: partial

   For any definition and any set of produced outputs, permuting the order the
   definition carries its instances in changes neither which instances may be
   offered nor what each activation carries.

   This is the property form of the measured defect, and the strongest case in
   the file: the defect was visible only because one line moved. A scheduler
   that asks readiness of the required parameters alone fails it, because the
   instance found first differs between permutations.

.. test_case:: An unbound optional parameter does not hold an instance back
   :id: TEST_SCHEDULER_UNBOUND_OPTIONAL_IS_READY
   :verifies: CREQ_SCHEDULER_READY_WHEN_BOUND
   :test_kind: positive
   :coverage: partial

   The first half of the requirement's must-pass list. An instance whose node
   type declares an optional parameter that the definition does not bind is
   offered as soon as its bound parameters have contexts.

   The control against over-correcting the case above: a scheduler that waits
   for every parameter a type declares deadlocks here, and nothing else in this
   file would notice.

.. test_case:: An entry instance is ready before anything has run
   :id: TEST_SCHEDULER_ENTRY_IS_READY_AT_ONCE
   :verifies: CREQ_SCHEDULER_READY_WHEN_BOUND
   :test_kind: positive
   :coverage: partial

   The second half of the must-pass list. An entry instance's parameters come
   from the run's arguments rather than from wires, so it is offered on the
   first ask, with no instance having produced anything.

.. test_case:: An instance bound to a name that resolves to nothing is never ready
   :id: TEST_SCHEDULER_UNRESOLVED_SOURCE_NEVER_READY
   :verifies: CREQ_SCHEDULER_READY_WHEN_BOUND
   :test_kind: error_path
   :coverage: partial

   A binding whose source names no instance can never arrive, so the instance
   carrying it is never offered and the scheduler reports quiescence instead.

   A run would refuse such a definition before starting
   (``CREQ_RUN_REFUSES_DEFECTS``), which is exactly why the scheduler is asked
   directly here: the component has to be correct on input the run would not
   hand it, or the two are only ever right together.

.. test_case:: An instance that has produced is not offered again
   :id: TEST_SCHEDULER_PRODUCED_INSTANCE_NOT_OFFERED
   :verifies: CREQ_SCHEDULER_READY_WHEN_BOUND
   :test_kind: error_path
   :coverage: partial

   An instance that has produced an output holds a context for every parameter
   it binds, which is the readiness test passing for the wrong reason. It is not
   offered again, and with every instance produced the scheduler reports
   quiescence rather than cycling.

   Without this the run never ends by itself and the budget becomes the only
   thing that stops it, which reports a sound run as a runaway.

.. test_case:: An activation's inputs are in declared order
   :id: TEST_SCHEDULER_INPUTS_IN_DECLARED_ORDER
   :verifies: CREQ_SCHEDULER_ACTIVATION_CARRIES
   :test_kind: error_path
   :coverage: partial

   A node type declaring three parameters in an order that is neither
   alphabetical nor the order the definition binds them in, so that declared
   order, binding order and sorted order are three different answers and a case
   built carelessly would not tell them apart.

   The activation carries the contexts in the order the type declares, which is
   what lets a node assemble its own inputs (``DEC_DECLARED_PARAMETERS``).

.. test_case:: Each parameter is given the context bound to it
   :id: TEST_SCHEDULER_EACH_PARAMETER_GETS_ITS_OWN
   :verifies: CREQ_SCHEDULER_ACTIVATION_CARRIES
   :test_kind: error_path
   :coverage: partial

   Two parameters declared for one context type, bound to two different
   producers. Each receives the context of its own source.

   Both being of one type is the point: swapping them passes every check that
   compares declared types, so only the identity of the context catches it.

.. test_case:: A requested global context type is not given to a node
   :id: TEST_SCHEDULER_GLOBALS_ARE_NOT_GIVEN
   :verifies: CREQ_SCHEDULER_ACTIVATION_CARRIES
   :test_kind: error_path
   :coverage: partial

   A node type requesting a global context type is offered an activation
   carrying its bound parameters and nothing else. Nothing supplies globals yet,
   so a context arriving under that heading was invented rather than read.

.. test_case:: An activation carries exactly its instance's bindings
   :id: TEST_SCHEDULER_ACTIVATION_IS_EXACTLY_ITS_BINDINGS
   :verifies: CREQ_SCHEDULER_ACTIVATION_CARRIES
   :test_kind: property
   :coverage: partial

   For any definition and any set of produced outputs, every activation the
   scheduler offers carries one context per parameter its instance binds, each
   the output of that binding's source, with no parameter missing and none
   added.

.. test_case:: Quiescence is reported only when nothing may activate
   :id: TEST_SCHEDULER_NONE_READY_ONLY_WHEN_NONE
   :verifies: CREQ_SCHEDULER_NONE_READY
   :test_kind: property
   :coverage: partial

   For any definition and any set of produced outputs, the scheduler reports
   that no instance may activate exactly when it offers none. The two are one
   answer, and a scheduler reporting quiescence while an instance is offerable
   ends a run whose result was still reachable.

.. test_case:: A cycle whose instances hold some inputs is still quiescent
   :id: TEST_SCHEDULER_PARTIAL_INPUTS_STILL_QUIESCENT
   :verifies: CREQ_SCHEDULER_NONE_READY
   :test_kind: error_path
   :coverage: partial

   Three instances in a cycle, each of a node type requiring two parameters, one
   of which is bound to an entry instance that has produced. Every instance
   holds a context for one parameter and waits for the other.

   A reading that reports quiescence only when no instance has any context at
   all never fires here, and the run would wait forever on a graph that cannot
   move.

.. test_case:: A workflow with a wiring defect never starts
   :id: TEST_RUN_DEFECTIVE_WORKFLOW_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_DEFECTS
   :test_kind: error_path
   :coverage: partial

   A definition carrying one wiring defect is refused, and no activation is
   offered - which is asserted rather than assumed, because a refusal that comes
   after the first activation looks identical from the caller's side if the
   caller only reads the outcome.

   The refusal is a value of its own kind rather than one of the four endings,
   so a caller can tell a workflow it must fix from a run that happened
   (``DEC_RUN_REFUSED_BEFORE_IT_STARTS``).

.. test_case:: A refusal carries every defect the validator found
   :id: TEST_RUN_REFUSAL_CARRIES_EVERY_DEFECT
   :verifies: CREQ_RUN_REFUSAL_NAMES_EVERY_DEFECT
   :test_kind: error_path
   :coverage: full

   A definition carrying five defects of four classes, two of them on one
   instance. The refusal carries all five, and they equal what
   ``validate_wiring`` reports for the same definition - compared as a whole
   rather than counted, so a run that forwards the right number of the wrong
   defects fails.

   Comparing against the validator's own report rather than against a written-out
   list is what keeps this case from going stale the next time a defect class is
   added.

.. test_case:: A missing argument is refused rather than reported as stuck
   :id: TEST_RUN_MISSING_ARGUMENT_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNFILLED_SIGNATURE
   :test_kind: error_path
   :coverage: partial

   A run started without an argument for a required entry parameter is refused,
   naming that instance and parameter.

   The assertion that matters is which answer came back, not that one did:
   measured, this shape reads as a stuck run naming every waiting instance
   (``EVD_RUN_MISSING_ARGUMENT_QUIESCES``), and a case asserting only that the
   run did not complete would pass against that.

.. test_case:: An argument of the wrong context type is refused
   :id: TEST_RUN_ARGUMENT_OF_WRONG_TYPE_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNFILLED_SIGNATURE
   :test_kind: error_path
   :coverage: partial

   An argument whose context type is not the one the entry parameter is declared
   for is refused, naming the parameter and both types.

   No binding exists to compare, so the wiring validator cannot see this and the
   run is the only thing that can.

.. test_case:: An entry parameter that is also bound is refused
   :id: TEST_RUN_ENTRY_PARAMETER_ALSO_BOUND_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNFILLED_SIGNATURE
   :test_kind: error_path
   :coverage: partial

   An entry instance carrying a binding for a parameter the run also supplies is
   refused, naming the parameter with two sources. The definition's wiring is
   sound and the validator reports nothing, which is what makes this the run's
   question.

   Asserted in both arrangements, with and without an argument for that
   parameter, because a reader that silently prefers the argument passes the
   first and a reader that silently prefers the wire passes the second.

.. test_case:: An argument for a parameter the workflow does not have is refused
   :id: TEST_RUN_ARGUMENT_FOR_NO_PARAMETER_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNFILLED_SIGNATURE
   :test_kind: error_path
   :coverage: partial

   An argument addressed to an instance the workflow does not carry, and one
   addressed to a parameter that instance's type does not declare, are each
   refused naming the pair that matched nothing.

   Ignoring them is what makes a typo silent: the argument the caller believes
   it supplied is somewhere else, and the real parameter is unfilled.

.. test_case:: A filled signature starts, however its names collide
   :id: TEST_RUN_SIGNATURE_FILLED_EXACTLY_STARTS
   :verifies: CREQ_RUN_REFUSES_UNFILLED_SIGNATURE
   :test_kind: positive
   :coverage: partial

   The requirement's must-pass list. One argument per required entry parameter,
   each of the declared type, starts a run - including the shape measured wrong
   (``EVD_RUN_ENTRY_NAME_SHARED``): two entry instances of node types that each
   declare a parameter called the same thing, for two different context types,
   are two parameters and receive their own arguments.

   An entry instance's unsupplied optional parameter is here too, since refusing
   it would make a workflow unusable that is not wrong.

.. test_case:: A run stops at the budget rather than one past it
   :id: TEST_RUN_BUDGET_STOPS_AT_THE_LIMIT
   :verifies: CREQ_RUN_STOPS_AT_BUDGET
   :test_kind: error_path
   :coverage: partial

   A workflow needing three activations, run with a budget of two: exactly two
   activations are offered and the run ends reporting the budget.

   Counting the activations rather than reading the ending is the whole case. A
   run that checks after offering ends with the same outcome having spent three,
   which for an LLM node is one unbudgeted call on every run that hits the
   limit.

.. test_case:: A budget of nothing activates nothing
   :id: TEST_RUN_ZERO_BUDGET_ACTIVATES_NOTHING
   :verifies: CREQ_RUN_STOPS_AT_BUDGET
   :test_kind: error_path
   :coverage: partial

   A run with a budget of nought offers no activation and ends reporting the
   budget, on a workflow that would otherwise complete.

   The degenerate value, and the one a careless reading turns into its opposite
   by treating nought as no limit.

.. test_case:: A run finishing on its last permitted activation completes
   :id: TEST_RUN_LAST_PERMITTED_ACTIVATION_COMPLETES
   :verifies: CREQ_RUN_STOPS_AT_BUDGET
   :test_kind: positive
   :coverage: partial

   A workflow needing exactly as many activations as its budget allows completes
   and carries its result, rather than ending on the budget at the same moment.

   The boundary from the other side, and the control on the case above: a run
   that ends on the budget before checking whether the result exists reports a
   runaway on a workflow that finished.

.. test_case:: Activations never outnumber the budget
   :id: TEST_RUN_ACTIVATIONS_NEVER_EXCEED_BUDGET
   :verifies: CREQ_RUN_STOPS_AT_BUDGET
   :test_kind: property
   :coverage: partial

   For any definition, any arguments and any budget, a run driven to its ending
   offers no more activations than the budget allows, whichever of the four ways
   it ends.

.. test_case:: A cycle ends the run and names what was waiting
   :id: TEST_RUN_CYCLE_ENDS_QUIESCENT
   :verifies: CREQ_RUN_ENDS_QUIESCENT
   :test_kind: error_path
   :coverage: partial

   Two instances bound to each other's output, with the designated output among
   them. The run ends quiescent, naming both, and reports none of the other
   three endings.

   A cycle is legal to write (``DEC_BACK_EDGES_ALLOWED``) and cannot be run
   until explicit control edges exist, so this is the shape that reaches
   quiescence honestly rather than through a mistake.

.. test_case:: An instance that can never run does not make a run stuck
   :id: TEST_RUN_IDLE_INSTANCE_DOES_NOT_MAKE_IT_STUCK
   :verifies: CREQ_RUN_ENDS_QUIESCENT
   :test_kind: positive
   :coverage: partial

   The measured shape (``EVD_RUN_COMPLETES_WITH_IDLE``): a definition holding an
   instance bound to its own output, which can never become ready, beside a
   designated output that produces. The run completes.

   The control that keeps quiescence from firing on every workflow with a branch
   the result does not depend on, and the case that catches a scheduler's
   quiescence being read as the run's.

.. test_case:: A quiescent run names only the instances that produced nothing
   :id: TEST_RUN_QUIESCENT_NAMES_ONLY_UNPRODUCED
   :verifies: CREQ_RUN_ENDS_QUIESCENT
   :test_kind: error_path
   :coverage: partial

   A workflow whose entry instance produces and whose remaining instances form a
   cycle: the ending names the cycle's instances and not the entry instance,
   which did its work.

   Naming everything would point a reader at the whole graph, which is the same
   as naming nothing.

.. test_case:: A failed activation ends the run carrying its failure
   :id: TEST_RUN_FAILED_ACTIVATION_ENDS_THE_RUN
   :verifies: CREQ_RUN_ENDS_ON_FAILURE
   :test_kind: error_path
   :coverage: partial

   A failure reported for an outstanding activation ends the run carrying that
   failure and the instance it was reported for, with no further activation
   offered although another instance was ready.

   That another instance was ready is what makes the case bite: without it, a
   run that continues is indistinguishable from one that stops.

.. test_case:: A failure reported with nothing outstanding is refused
   :id: TEST_RUN_FAILURE_WITH_NO_ACTIVATION_IS_REFUSED
   :verifies: CREQ_RUN_ENDS_ON_FAILURE
   :test_kind: error_path
   :coverage: partial

   A failure reported when no activation is outstanding is refused rather than
   filed against an instance. There is no instance to name, and choosing one
   would attribute a failure to a node that never ran.

.. test_case:: A run's result is its designated output
   :id: TEST_RUN_COMPLETES_ON_DESIGNATED_OUTPUT
   :verifies: CREQ_RUN_COMPLETES
   :test_kind: positive
   :coverage: partial

   A linear workflow completes, and the result is the context the designated
   instance produced - the context itself, which the case reads content and
   declared type from, rather than an identifier a caller would have to hold the
   run to resolve. It is compared by identity as well, so that a different
   context holding the same bytes fails.

.. test_case:: A run ends without activating what the result does not need
   :id: TEST_RUN_COMPLETES_BEFORE_OFFERING_MORE
   :verifies: CREQ_RUN_COMPLETES
   :test_kind: error_path
   :coverage: partial

   A workflow whose designated instance is ready before a second, unrelated
   branch: the run ends as soon as that instance produces, and the branch is
   never offered.

   This is the failure no assertion on the result can catch, because a run that
   carries on arrives at the same result having spent the activations.

.. test_case:: The result is the designated context and not the last produced
   :id: TEST_RUN_RESULT_IS_THE_DESIGNATED_CONTEXT
   :verifies: CREQ_RUN_COMPLETES
   :test_kind: error_path
   :coverage: partial

   A workflow in which several instances have produced by the time it completes,
   so that returning any produced context other than the designated instance's
   gives a different answer.

   This case was first written as a workflow whose designated instance is not
   the last to produce, and that shape turned out to be unreachable: a run ends
   as soon as the designated instance produces
   (``CREQ_RUN_COMPLETES``), so that instance is always the last to have
   produced. What is reachable, and what a careless run does, is returning some
   other produced context - the first the run happens to hold, which in a linear
   workflow is indistinguishable from the right answer.

   Without it, every case in this file passes against a run that returns
   whatever produced context comes to hand, because everywhere else the
   designated one is the only candidate.

.. test_case:: Well-formed workflows run to completion
   :id: TEST_RUN_WELL_FORMED_RUNS_COMPLETE
   :verifies: FEAT_RUN_COMPLETES_ON_DESIGNATED
   :test_kind: property
   :coverage: partial

   For any well-formed definition with a generous budget and every entry
   parameter supplied, a driven run completes and its result is the context its
   designated instance produced.

   The counterweight to a file of refusals. Most requirements here say what must
   not happen, and a run that refuses everything satisfies them; this is what
   that run fails.

.. test_case:: An output of a type its node type does not declare is refused
   :id: TEST_RUN_UNDECLARED_OUTPUT_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNDECLARED_OUTPUT
   :test_kind: error_path
   :coverage: partial

   The measured defect (``EVD_RUN_ACCEPTS_UNDECLARED_OUTPUT``). An entry
   instance whose node type declares one output type is answered with a context
   of another. The refusal names that instance, the declared type and the
   reported type, as values, and is told apart from a refusal for a held
   identifier by which it is.

   The instance consuming that output is not offered next. That is what shows
   the output was refused before it was recorded rather than recorded and then
   complained about, since a recorded output would have made its consumer ready.

.. test_case:: The designated instance's output type is checked
   :id: TEST_RUN_DESIGNATED_UNDECLARED_OUTPUT_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNDECLARED_OUTPUT
   :test_kind: error_path
   :coverage: partial

   A workflow whose designated instance is answered with a context of a type its
   node type does not declare. Nothing consumes that output, so a run comparing
   outputs with the parameters they reach checks nothing here, and it would
   complete with a mistyped result.

.. test_case:: An output of the declared type is accepted whatever its inputs were
   :id: TEST_RUN_OUTPUT_OF_DECLARED_TYPE_IS_ACCEPTED
   :verifies: CREQ_RUN_REFUSES_UNDECLARED_OUTPUT
   :test_kind: positive
   :coverage: partial

   A node type whose output type differs from its input's is answered with a
   context of its declared output type, and then with a composition of that type
   whose part is of another. Both are accepted.

   The first control refuses a run that compares outputs with inputs, which
   every type-changing node would fail. The second refuses one that compares a
   composition's parts rather than the composition.

.. test_case:: An argument handed back as an output is refused
   :id: TEST_RUN_PASSED_THROUGH_ARGUMENT_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_HELD_IDENTIFIER
   :test_kind: error_path
   :coverage: partial

   Half of the measured defect (``EVD_RUN_ACCEPTS_HELD_IDENTIFIER``). An entry
   instance is answered with the argument it was given, whose type is the
   declared output type so that only the identifier is wrong. The refusal names
   the instance and the argument's identifier.

.. test_case:: Another instance's output handed back is refused
   :id: TEST_RUN_PASSED_THROUGH_OUTPUT_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_HELD_IDENTIFIER
   :test_kind: error_path
   :coverage: partial

   The other half of the measured defect, in its harder shape: the output handed
   back is that of an instance which is not wired to the one being answered, so
   a run comparing an output with the instance's own inputs finds nothing. The
   refusal names the instance and the identifier handed back.

.. test_case:: An output composing its input is accepted
   :id: TEST_RUN_OUTPUT_COMPOSING_ITS_INPUT_IS_ACCEPTED
   :verifies: CREQ_RUN_REFUSES_HELD_IDENTIFIER
   :test_kind: positive
   :coverage: partial

   An instance is answered with a composition holding the input it was given.
   The composition has an identifier of its own and holds its part by
   reference, and it is accepted - the one sanctioned way to pass an input on,
   and the control the measurement took.

.. test_case:: Every output a run accepts is new to it
   :id: TEST_RUN_ACCEPTED_OUTPUTS_ARE_ALL_NEW
   :verifies: CREQ_RUN_REFUSES_HELD_IDENTIFIER
   :test_kind: property
   :coverage: partial

   For any chain of instances and any caller choosing, for each activation, a
   new context, a composition of what the run holds, or any context the run
   holds handed back unchanged: an output is refused exactly when it was handed
   back, and no two contexts the run accepted or was started with share an
   identifier.

.. test_case:: A refused output leaves the same activation outstanding
   :id: TEST_RUN_REFUSED_OUTPUT_KEEPS_THE_ACTIVATION
   :verifies: CREQ_RUN_REFUSED_OUTPUT_OUTSTANDING
   :test_kind: error_path
   :coverage: partial

   An instance answered with a refused output. The run has not ended, and the
   next step offers that same instance again rather than the instance consuming
   its output. An accepted answer then lets the run go on to completion.

   The first half catches a run ended by the refusal, and the second a refused
   output recorded anyway, which would have made its consumer ready. What it
   cannot tell apart is an activation kept from one dropped and offered afresh:
   the scheduler offers the first ready instance in the definition's order
   either way, so both offer the same one.
   ``TEST_RUN_REFUSAL_DOES_NOT_SPEND_THE_BUDGET`` is the case that separates
   them, because only one of the two counts it again.

.. test_case:: A refusal does not spend the budget again
   :id: TEST_RUN_REFUSAL_DOES_NOT_SPEND_THE_BUDGET
   :verifies: CREQ_RUN_REFUSED_OUTPUT_OUTSTANDING
   :test_kind: error_path
   :coverage: partial

   A workflow run with a budget of exactly as many activations as it has
   instances, one answer refused and then corrected. The run completes rather
   than ending on its budget, which it would if the activation had been counted
   a second time.

.. test_case:: A refused activation can be failed
   :id: TEST_RUN_REFUSED_OUTPUT_CAN_BE_FAILED
   :verifies: CREQ_RUN_REFUSED_OUTPUT_OUTSTANDING
   :test_kind: error_path
   :coverage: partial

   A failure reported after a refused output ends the run carrying that failure
   and naming the instance, rather than being refused because nothing is
   outstanding. A caller that cannot produce an acceptable output has to be
   able to give up on the activation, and this is the only way it can.
