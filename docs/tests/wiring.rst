=================
Wiring test cases
=================

How each requirement in ``components/wiring`` is to be checked. Results are never
written here: they are imported from the test runner, and each case records only
what it asserts.

Every failure mode listed in ``components/wiring`` has a case that catches it,
and every "must pass unreported" list has a positive case naming its shapes. The
two lists pull in opposite directions on purpose - one is the validator failing
to report a defect, the other is the validator reporting one that is not there -
and a set of cases derived from only one of them would leave the other free.

That matters more here than in the feature before this one. Most of the
requirements say what must be refused and none of those says what must not be, so
a validator refusing everything satisfies all of them and no case built from them
would notice. The positive cases and ``TEST_WIRING_WELL_FORMED_DEFINITIONS_PASS``
are the half that does.

Four failure modes have no case of their own, because each is an interaction
between requirements and is asserted where that interaction happens. They are
named here so the derivation can be audited rather than taken on trust:

- ``CREQ_VALIDATOR_ONE_OUTPUT``'s "the check refuses the definition before the
  wiring is examined" and ``CREQ_VALIDATOR_TYPES_AGREE``'s "the check is skipped
  because something else about the wire was reported" are both what
  ``TEST_WIRING_ALL_FOUR_CLASSES_REPORTED`` is built to catch: its definition
  carries a signature defect and three wiring defects, two of them on one
  instance, so a walk that stops early anywhere reports three rather than four.
- ``CREQ_VALIDATOR_TYPES_AGREE``'s "a wire whose producer has no declaration is
  reported as a mismatch" is asserted by
  ``TEST_WIRING_INSTANCE_OF_MISSING_TYPE_IS_REPORTED``, which is about one
  definition and one report, and would be split in two by filing half of it
  under each requirement.
- ``CREQ_VALIDATOR_OUTPUT_RESOLVES``'s "a shared name is reported as naming
  nothing" is asserted by ``TEST_WIRING_SHARED_INSTANCE_NAME_IS_REPORTED``, whose
  definition designates the shared name as its output, for the same reason.

A case's id is the path of the Rust test that implements it, uppercased, with
``::`` written as ``_``. The paths here are ``<module>::<case>`` in
``agconflo-core``, with the modules ``wiring`` and ``defect`` - one per
component, as ``context``, ``id`` and ``lineage`` are for the feature before this
one. The ids hold as written only for tests that are bare test functions in those
modules, not inside a ``tests`` module (``EVD_NEXTEST_TEST_PATHS``).

That correspondence runs both ways, which is why this document has exactly as many
cases as those two modules will have tests. Every test in the crate is imported as
a ``test_run`` whose ``executes`` link names its case, so a test with no case here
is a dead link that fails the build rather than a result nobody noticed.

Two shapes are asserted on but not classified, and the distinction is the whole
reason ``TEST_WIRING_MALFORMED_DEFINITION_STILL_REPORTS`` reads the way it does:
two node types sharing one name within a definition, and - until every parameter
was required (``DEC_EVERY_INPUT_REQUIRED``) - one parameter declared as both
required and optional, which a declaration can no longer hold.
``CREQ_VALIDATOR_BINDING_RESOLVES`` records the first as still open, so which
defect it earns is unsettled, and a case asserting one would be inventing a
requirement rather than checking one. What is settled is that it may not stop
the walk, and that is what the case asserts. The three
shapes it held before them are classified now, each by cases of its own below.
The third, a binding into an entry node, is closed: there are no entry nodes
(``STKH_RUN_FROM_ANY_PARAMETER``).

.. test_case:: Globals and empty declarations pass
   :id: TEST_WIRING_DEGENERATE_DECLARATIONS_PASS
   :verifies: CREQ_VALIDATOR_ACCEPTS_WELL_FORMED
   :test_kind: positive
   :coverage: partial

   Nothing is reported for a node type declaring no parameters at all; a node
   requesting a global context type that no binding carries; and parameters
   nothing binds, on an instance with no bindings and on one with some, which
   are the workflow's inputs rather than broken wires
   (``STKH_RUN_FROM_ANY_PARAMETER``).

   A generator reaches a node type with an empty parameter list rarely, and the
   well-formed generator never leaves a parameter unbound, so none of these
   shapes is reached by ``TEST_WIRING_WELL_FORMED_DEFINITIONS_PASS``.

.. test_case:: A binding to an instance that is not there is reported
   :id: TEST_WIRING_BINDING_TO_MISSING_INSTANCE_IS_REPORTED
   :verifies: CREQ_VALIDATOR_BINDING_RESOLVES
   :test_kind: error_path
   :coverage: partial

   A binding naming an instance the definition does not carry is reported as one
   unresolved defect, naming the consuming instance and parameter and echoing the
   name that resolved to nothing.

   What must not also appear is the point of the case: one wire to nowhere is one
   defect, and nothing more is said about the parameter it fills, which is
   bound.

.. test_case:: Each wire to a missing name is reported
   :id: TEST_WIRING_EVERY_BROKEN_WIRE_IS_REPORTED
   :verifies: CREQ_VALIDATOR_BINDING_RESOLVES
   :test_kind: property
   :coverage: partial

   For any number of parameters bound to one instance name the definition does
   not carry, there is one defect per binding, each naming its own consumer.

   Collapsing them into a single defect about the name is what this rules out.
   Each binding is a separate wire to repoint, a defect is one thing wrong with
   one definition and carries one place, so a single defect could name only one
   of them and would leave every other broken wire unnamed.

.. test_case:: An instance of a type that was not supplied is reported
   :id: TEST_WIRING_INSTANCE_OF_MISSING_TYPE_IS_REPORTED
   :verifies: CREQ_VALIDATOR_BINDING_RESOLVES
   :test_kind: error_path
   :coverage: partial

   An instance whose node type the definition was not given is reported as one
   defect, naming that instance and echoing the type name.

   Nothing else about that instance is reported, and that is the half of the case
   that can regress quietly: its parameters are unknowable, so no defect is
   invented about its own bindings, and no type-agreement defect is invented for
   the wires it feeds. The rest of the definition is still walked,
   so another instance's defect appears in the same report.

.. test_case:: A self-binding and shared node types pass
   :id: TEST_WIRING_SELF_BINDING_AND_SHARED_TYPES_PASS
   :verifies: CREQ_VALIDATOR_BINDING_RESOLVES
   :test_kind: positive
   :coverage: partial

   Nothing is reported for a binding from an instance to itself, which is a cycle
   of length one and legal (``DEC_BACK_EDGES_ALLOWED``), nor for two instances of
   the same node type, which share a declaration and nothing else.

   Both resolve, and both are what a resolver written as if the definition were a
   tree of distinct nodes refuses by accident.

.. test_case:: A wire across two context types is reported
   :id: TEST_WIRING_TYPE_DISAGREEMENT_IS_REPORTED
   :verifies: CREQ_VALIDATOR_TYPES_AGREE
   :test_kind: error_path
   :coverage: partial

   A binding joining an output declared of one context type to a parameter
   declared of another is reported as one defect, naming the consuming instance
   and parameter and carrying both type names.

   Both names are asserted on, because what a reader has to act on is which type
   was expected and which arrived; a defect saying only that the two differ sends
   them back to the declarations to find out.

.. test_case:: Context type names are compared exactly
   :id: TEST_WIRING_TYPE_NAMES_COMPARE_EXACTLY
   :verifies: CREQ_VALIDATOR_TYPES_AGREE
   :test_kind: property
   :coverage: partial

   For any two context type names that differ as strings, a binding joining them
   is reported; for any one name, a binding joining it to itself is not.

   The generator must produce pairs differing only in case, pairs differing only
   by surrounding whitespace, and pairs where one is a prefix of the other. Those
   are the three ways the comparison gets loosened, and a pair of random names is
   unlikely to be any of them, so a generator left to itself would prove only
   that unrelated names differ.

.. test_case:: Agreeing wires and fan-out pass
   :id: TEST_WIRING_AGREEING_WIRES_PASS
   :verifies: CREQ_VALIDATOR_TYPES_AGREE
   :test_kind: positive
   :coverage: partial

   Nothing is reported for a binding whose output and parameter declare the same
   context type on two different node types, nor for one output bound by several
   parameters, which the model allows (``DEC_BINDING_BY_PORT``).

   Fan-out is a structural shape rather than a comparison, which is why it is
   here rather than left to the property above: a check recording one consumer per
   output would refuse it while comparing every name correctly.

.. test_case:: A signature without exactly one output is reported
   :id: TEST_WIRING_SIGNATURE_WITHOUT_ONE_OUTPUT_IS_REPORTED
   :verifies: CREQ_VALIDATOR_ONE_OUTPUT
   :test_kind: error_path
   :coverage: partial

   A definition designating no output, and one designating two, each report
   exactly one signature defect naming the definition. The wiring of both is
   otherwise sound, so the signature defect is the whole report.

   Both are asserted, separately. A check that counts designations and compares
   against one catches them both; a check that only tests for absence passes the
   definition designating two, and a workflow whose result is whichever output a
   caller happens to bind is the failure that requirement exists to rule out.

.. test_case:: Legal signatures pass
   :id: TEST_WIRING_LEGAL_SIGNATURES_PASS
   :verifies: CREQ_VALIDATOR_ONE_OUTPUT
   :test_kind: positive
   :coverage: partial

   Nothing is reported for a workflow whose every parameter is bound, which is
   legal,
   nor for a designated output whose node also feeds other nodes, which does not
   make it less terminal.

   The second is what a check written around "terminal" rather than around
   "designated" refuses, and it is the ordinary shape of a loop
   (``DEC_BACK_EDGES_ALLOWED``), where the designated output feeds the node that
   starts the next pass.

.. test_case:: A binding to an undeclared parameter is reported
   :id: TEST_WIRING_UNDECLARED_PARAMETER_IS_REPORTED
   :verifies: CREQ_VALIDATOR_PARAMETER_DECLARED
   :test_kind: error_path
   :coverage: partial

   On instances of a node type requiring one parameter and requesting a global
   type, each of these bindings is reported as one defect naming its instance
   and the parameter as it was written: a name no parameter has; a binding
   spelled like the requested global; a typo of
   the required parameter; and an undeclared parameter bound to an instance
   that is not there, reported beside that unresolved source.

   The first is the case the requirement exists for: nothing else about the
   instance is wrong. The last is what a walk moving on after one defect gets
   wrong, hiding a second fix behind the first. And no context
   type disagreement is reported for any of them, although the instance they are
   wired from produces a type no declared parameter has: an undeclared parameter
   has no type to compare, and comparing against a stand-in for one invents a
   disagreement.

.. test_case:: Every undeclared parameter is reported
   :id: TEST_WIRING_EVERY_UNDECLARED_PARAMETER_IS_REPORTED
   :verifies: CREQ_VALIDATOR_PARAMETER_DECLARED
   :test_kind: property
   :coverage: partial

   For any definition whose node types declare parameters and requested globals
   in any number, and whose instances bind names drawn from a pool holding every
   declared parameter, names spelled like requested globals and names nothing
   declares, the undeclared-parameter defects are exactly the bindings whose name
   no parameter has, computed
   independently by the test. Some instances are of a node type that was not
   supplied, and nothing about them is reported as undeclared.

   The pool puts declared and undeclared names side by side on one instance,
   because a check reading the wrong list passes every instance whose bindings
   are all of one kind.

.. test_case:: Bindings to declared parameters pass
   :id: TEST_WIRING_DECLARED_PARAMETERS_PASS
   :verifies: CREQ_VALIDATOR_PARAMETER_DECLARED
   :test_kind: positive
   :coverage: partial

   Nothing is reported for an instance binding both of the parameters its node
   type declares, each from a source of its type.

   The control against a check reporting a declared parameter: with two, one
   searching only the first entry of the list reports the second.

.. test_case:: An output naming no instance is reported
   :id: TEST_WIRING_OUTPUT_NAMING_NOTHING_IS_REPORTED
   :verifies: CREQ_VALIDATOR_OUTPUT_RESOLVES
   :test_kind: error_path
   :coverage: partial

   A definition whose wiring is sound and whose one designated output names no
   instance reports exactly one defect, naming the definition and echoing the
   designated name. A definition designating two names, one of which names
   nothing, and one designating two names of which neither names anything, each
   report exactly the defect for designating two, and nothing about either name.

   The first is the defect a count cannot see, and reporting it as designating
   nothing would pass a test of the count while losing the name. The other two
   are where resolving every designation reports the definition once per name.

.. test_case:: Outputs that resolve pass
   :id: TEST_WIRING_RESOLVING_OUTPUTS_PASS
   :verifies: CREQ_VALIDATOR_OUTPUT_RESOLVES
   :test_kind: positive
   :coverage: partial

   Nothing is reported for an output naming an instance with a parameter nothing
   binds, nor for one naming an
   instance whose name is not its node type's.

   The second is what an output resolved against the node types rather than the
   instances refuses, and such a check passes every definition whose instances
   happen to be named after their types - which is how short examples get written.

.. test_case:: A name several instances share is reported once
   :id: TEST_WIRING_SHARED_INSTANCE_NAME_IS_REPORTED
   :verifies: CREQ_VALIDATOR_INSTANCE_NAMED_ONCE
   :test_kind: error_path
   :coverage: partial

   Four instances sharing one name - of two node types producing different
   context types, of a type with a parameter, and of a type that was not
   supplied - are reported as exactly one defect naming that name. A binding from
   the name into a parameter declared for one of the two context types adds
   nothing, and nor does the designated output naming it; the rest of the
   definition is still walked, so another instance's wire to nowhere is in the
   same report. The definition is checked twice, with the two instances
   producing different types in either order and a different instance first each
   time - the second time, the one of the type that was not supplied - and the
   report is the same both times.

   Two orders, because a validator resolving the name to the first instance
   carrying it passes exactly one of them, and one walking only that first
   instance reports its missing type in the second. What is not reported is the
   rest of the case: the missing type, the disagreement and an unresolved output are each what checking one instance as
   if the name were its own produces, and each would name a node the author
   cannot find.

.. test_case:: Names of different kinds pass
   :id: TEST_WIRING_NAMES_OF_DIFFERENT_KINDS_PASS
   :verifies: CREQ_VALIDATOR_INSTANCE_NAMED_ONCE
   :test_kind: positive
   :coverage: partial

   Nothing is reported for an instance named like its own node type, one named
   like a parameter, and two instances of one node type under different names.

   A check gathering every name in a definition into one set before looking for
   repeats refuses the first two, and the first is how a definition with one
   instance of each type is naturally written.

.. test_case:: A parameter bound twice is reported once
   :id: TEST_WIRING_PARAMETER_BOUND_TWICE_IS_REPORTED
   :verifies: CREQ_VALIDATOR_PARAMETER_BOUND_ONCE
   :test_kind: error_path
   :coverage: partial

   Each of these is reported as exactly one defect naming its instance and
   parameter: a parameter bound to two instances that are not there; one bound
   three times to one sound source; and one bound to two sound sources whose
   context types disagree. An undeclared parameter bound twice is reported as
   bound twice and as undeclared, once each.

   Each shape catches a different wrong walk. Checking each binding reports the
   first twice over and the third as a disagreement, and passes the second;
   checking the first binding reports an unresolved source for the first;
   reporting each binding beyond the first reports the second twice. The last
   shape holds the one check that still runs on such a parameter, and a walk
   skipping everything about a parameter bound twice loses it.

.. test_case:: A parameter bound once on each instance passes
   :id: TEST_WIRING_SINGLY_BOUND_PARAMETERS_PASS
   :verifies: CREQ_VALIDATOR_PARAMETER_BOUND_ONCE
   :test_kind: positive
   :coverage: partial

   Nothing is reported for one parameter name bound once on each of several
   instances, nor for one output bound by two parameters of one instance.

   A check counting parameter names across the definition refuses the first, and
   one counting sources within an instance refuses the second - and each is a
   graph drawn every day.

.. test_case:: A definition with four defect classes reports all of them
   :id: TEST_WIRING_ALL_FOUR_CLASSES_REPORTED
   :verifies: CREQ_VALIDATOR_EVERY_DEFECT
   :test_kind: error_path
   :coverage: partial

   A definition carrying one defect of each of four classes - a binding to a
   parameter its type does not declare, a binding to a name that is not there, a
   wire across two context types, and no designated output - reports exactly four
   defects, one of each class, each once.

   Where the four sit is as much of the case as the count. The undeclared
   parameter and the mismatched wire are on the same instance, so a walk that moves on once
   an instance has a defect reports three; and the signature is malformed, so a
   validator that refuses the definition for that before examining any wiring
   reports one. Both are failure modes their own requirements enumerate, and
   neither is visible in a definition whose defects are one per instance.

   The author learns about the rest only after a second submission in either
   case, which is the round trip ``FEAT_WIRING_ALL_DEFECTS`` exists to prevent.

.. test_case:: The name defects are reported with everything else
   :id: TEST_WIRING_NAME_DEFECTS_REPORTED_TOGETHER
   :verifies: CREQ_VALIDATOR_EVERY_DEFECT
   :test_kind: error_path
   :coverage: partial

   A definition carrying a name two instances share, an undeclared parameter
   bound twice on one instance, and an output naming no instance reports exactly
   four defects, each once: the shared name, the undeclared parameter, the
   parameter bound twice, and the output.

   The rules for a name that picks out two things each stop part of the walk on
   purpose, and this is the case for a stop placed one step too wide. Two of the
   four sit on one instance, so a walk leaving an instance once a binding repeats
   reports fewer, and the shared name comes first, so one leaving the definition
   once a name is shared reports one.

.. test_case:: No defect is reported twice
   :id: TEST_WIRING_NO_DEFECT_IS_REPORTED_TWICE
   :verifies: CREQ_VALIDATOR_EVERY_DEFECT
   :test_kind: property
   :coverage: partial

   For any definition, defective or not, no two reported defects are equal, and
   no place is named twice by defects of the same class.

   The second half is what catches a duplicate carrying a different message for
   the same wire - reported once per direction of a binding, or once per check
   that touched it. Equality alone would let those through, and a report that
   grows without saying more is what an author paying per round trip reads.

   The generator gives some instances a shared name and binds some parameters
   more than once. Those are the two shapes in which checking each thing a name
   picks out on its own reports it twice, and both were measured doing so before
   the validator answered them.

.. test_case:: A malformed definition is reported rather than fatal
   :id: TEST_WIRING_MALFORMED_DEFINITION_STILL_REPORTS
   :verifies: CREQ_VALIDATOR_EVERY_DEFECT
   :test_kind: error_path
   :coverage: partial

   Each of these returns a report rather than ending the run: two node types
   sharing one name; a binding naming an empty instance name; a definition declaring node types it has
   no instances of.

   Not ending the run is the whole assertion, and it is deliberately the whole of
   it. An index out of range, or an unwrap on a declaration that is not there,
   stops the walk at the first defect wearing different clothes, and a process
   that aborts reports nothing at all - so ``CREQ_VALIDATOR_EVERY_DEFECT`` is
   what these shapes are held to. Which defect, if any, the first earns is
   recorded as still open under ``CREQ_VALIDATOR_BINDING_RESOLVES``, and
   asserting a class here would pin behaviour no requirement asks for and make
   the next slice's answer a regression.

   The report is read rather than discarded, so the test fails if the call
   returns something unreadable, but nothing is asserted about its contents. That
   is a weaker case than the others in this document, and knowingly so: the
   alternative is inventing the classification it is missing.

.. test_case:: A well-formed definition is reported as nothing
   :id: TEST_WIRING_WELL_FORMED_DEFINITIONS_PASS
   :verifies: CREQ_VALIDATOR_ACCEPTS_WELL_FORMED
   :test_kind: property
   :coverage: partial

   For any definition generated well formed by construction - every required
   parameter bound, every name resolving, every wire agreeing on its context
   type, exactly one designated output - the report is empty.

   The generator carries this case, and it must reach the shapes a validator
   written to be strict refuses by accident: a cycle, a node nothing
   reaches, a declared global that no binding carries, and an output bound by several parameters. A generator producing only
   trees would pass against a validator that refuses every one of them, which
   would make this the weakest case in the document rather than the control every
   other case here is measured against.

.. test_case:: An empty definition is refused for its signature alone
   :id: TEST_WIRING_EMPTY_DEFINITION_IS_REFUSED_FOR_ITS_SIGNATURE
   :verifies: CREQ_VALIDATOR_ACCEPTS_WELL_FORMED
   :test_kind: error_path
   :coverage: partial

   A definition with no instances and no node types reports exactly one defect,
   the signature defect, and no wiring defect of any class.

   It is the case where two requirements could quietly both fire, since an empty
   definition has nothing to wire and nothing to designate. Asserting the count
   and the class is what separates "refused for the right reason" from "refused",
   and only one of those two readings survives adding an instance to it.

.. test_case:: A binding defect names its instance and parameter
   :id: TEST_DEFECT_NAMES_INSTANCE_AND_PARAMETER
   :verifies: CREQ_DEFECT_NAMES_PLACE
   :test_kind: property
   :coverage: partial

   For any definition, every reported defect that concerns a wire - a required
   parameter carrying no binding, a binding that resolves to nothing, a binding
   whose two ends declare different context types, a binding to a parameter its
   type does not declare, and a parameter bound more than once - carries the
   consuming instance and the parameter as values read from the defect itself,
   and each names something the definition carries.

   The five are named here rather than counted off the requirements, because
   ``CREQ_VALIDATOR_BINDING_RESOLVES`` covers two shapes and only one of them is
   a wire, and two of the classes about names are not wires either: a name
   several instances share concerns an instance, and an output naming nothing
   concerns the definition. An instance of a node type that was not supplied
   concerns that instance and no parameter at all: its declaration is what is
   missing, so its parameter list is unknowable, and a defect per parameter would
   contradict the single defect ``TEST_WIRING_INSTANCE_OF_MISSING_TYPE_IS_REPORTED``
   asks for.
   Filling the field anyway is the failure mode ``CREQ_DEFECT_NAMES_PLACE``
   forbids in its other form, where a signature defect invents a parameter and
   sends the author to a node that is not wrong. That case is where an instance
   defect's place is checked.

   Read from the defect rather than from its rendering, deliberately. A place
   that can be recovered only by parsing a message is a place an agent correcting
   its own workflow cannot use, and the two are indistinguishable to a human
   reading the output - which is how this requirement would be lost without
   anyone noticing.

.. test_case:: A signature defect names the definition and no node
   :id: TEST_DEFECT_SIGNATURE_NAMES_THE_DEFINITION
   :verifies: CREQ_DEFECT_NAMES_PLACE
   :test_kind: error_path
   :coverage: partial

   The defect reported for a definition designating no output names the
   definition, and carries no node instance and no parameter; and so does the
   one reported for an output naming no instance.

   The absence is the assertion. A defect shaped so that every one of them has an
   instance forces a signature defect to name some node, and the author is then
   sent to a node that is not wrong.

.. test_case:: An unresolved name is echoed back
   :id: TEST_DEFECT_ECHOES_AN_UNRESOLVED_NAME
   :verifies: CREQ_DEFECT_NAMES_PLACE
   :test_kind: error_path
   :coverage: partial

   A defect reported for a binding to an instance that is not there carries that
   name as it was written, and so does one reported for an instance of a node
   type that was not supplied, and one reported for an output naming no
   instance.

   The name resolves to nothing, which is exactly why it has to be echoed: it is
   the only thing tying the defect to what the author typed. A defect reporting
   the place without it says a wire is broken without saying what it was meant to
   point at.
